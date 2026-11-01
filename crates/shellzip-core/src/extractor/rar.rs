//! RAR 4 and RAR 5 listing, testing, and extraction.
//!
//! The `unrar` crate binds UnRAR. That licence allows extraction only, so this
//! module never writes a RAR archive. Split volumes (`.part1.rar`) are listed
//! but not extracted: UnRAR's volume-change callback AVs on Windows. Header
//! encryption fails with exit 11 until a password is supplied, and a wrong
//! password is the same exit code. UnRAR does not expose host OS or the
//! dictionary size, so those listing fields stay empty. Solid and encrypted
//! flags are reported.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;
use unrar::error::{Code, UnrarError};
use unrar::{Archive, CursorBeforeFile, CursorBeforeHeader, List, OpenArchive, Process};

/// UnRAR's C++ library (`RAR_SMP`) is process-global and not thread-safe.
/// `cargo test` runs integration tests in parallel; two opens at once have
/// crashed Windows CI with `STATUS_ACCESS_VIOLATION`.
fn unrar_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Decode through UnRAR's test callback instead of `RAR_EXTRACT`.
///
/// `extract_to` asks UnRAR to create the file. That path has AVed on
/// Windows Server runners. Members above this size still use `extract_to`.
const DECODE_IN_MEMORY: u64 = 32 * 1024 * 1024;

use super::policy::{
    Budget, Destination, destination, mark_extracted, restore_entry_time, stage_path,
};
use crate::crypto::Password;
use crate::entry::{Entry, EntryKind};
use crate::error::Error;
use crate::hash::{Blake2spWriter, CrcWriter};
use crate::options::{ConflictDecider, ExtractOpts, ExtractReport, TestReport};
use crate::progress::ProgressSink;

/// List RAR members. `hash` decodes payloads for BLAKE2sp.
pub fn list_rar(path: &Path, hash: bool, password: Option<&Password>) -> Result<Vec<Entry>, Error> {
    let _lock = unrar_lock();
    if hash {
        return list_hashed(path, password);
    }
    let archive = open_listing(path, password)?;
    let solid = archive.is_solid();
    let headers_encrypted = archive.has_encrypted_headers();
    let mut entries = Vec::new();
    for item in archive {
        let header = item.map_err(|err| map_unrar(err, password.is_some()))?;
        entries.push(entry_from(&header, solid, headers_encrypted)?);
    }
    Ok(entries)
}

/// Decode every member. Encrypted members without a password are reported, not unpacked.
pub fn test_rar(
    path: &Path,
    progress: &dyn ProgressSink,
    password: Option<&Password>,
    limits: &crate::options::ReadLimits,
) -> Result<TestReport, Error> {
    let _lock = unrar_lock();
    refuse_multipart(path)?;
    let mut archive = open_process(path, password)?;
    progress.begin(&format!("test {}", path.display()), 0);
    let mut report = TestReport::default();
    let mut budget = Budget::for_archive(path, limits.max_bytes, limits.max_ratio);
    loop {
        let Some(header) = archive
            .read_header()
            .map_err(|err| map_unrar(err, password.is_some()))?
        else {
            break;
        };
        let name = header_name(header.entry())?;
        report.entries += 1;
        if header.entry().is_directory() {
            progress.begin_entry(&name, 0);
            progress.end_entry();
            archive = header
                .skip()
                .map_err(|err| map_unrar(err, password.is_some()))?;
            continue;
        }
        if header.entry().is_encrypted() && password.is_none() {
            report.encrypted.push(name);
            archive = header
                .skip()
                .map_err(|err| map_unrar(err, password.is_some()))?;
            continue;
        }
        let size = header.entry().unpacked_size;
        budget.reserve(size, None)?;
        progress.begin_entry(&name, size);
        archive = match header.test() {
            Ok(next) => next,
            Err(err) if err.code == Code::BadData => {
                return Err(Error::BadChecksum { entry: name });
            }
            Err(err) => return Err(map_unrar(err, password.is_some())),
        };
        budget.charge(size)?;
        report.bytes += size;
        progress.end_entry();
    }
    Ok(report)
}

/// Extract selected members. Names are checked before anything is created.
pub fn extract_rar(
    path: &Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    selected: &dyn Fn(&str) -> bool,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    let _lock = unrar_lock();
    refuse_multipart(path)?;
    let mut archive = open_process(path, opts.password.as_ref())?;
    progress.begin(&format!("extract {}", path.display()), 0);
    let mut report = ExtractReport::default();
    let mut budget = Budget::for_archive(path, opts.max_bytes, opts.max_ratio);
    let had_password = opts.password.is_some();
    loop {
        let Some(header) = archive
            .read_header()
            .map_err(|err| map_unrar(err, had_password))?
        else {
            break;
        };
        let name = header_name(header.entry())?;
        if name.is_empty() || !selected(&name) {
            archive = header.skip().map_err(|err| map_unrar(err, had_password))?;
            continue;
        }
        let size = header.entry().unpacked_size;
        budget.reserve(size, None)?;
        progress.begin_entry(&name, size);
        if header.entry().is_directory() {
            if !opts.flat {
                match destination(&name, opts, false, true, None, decide)? {
                    Destination::Skip => report.skipped += 1,
                    Destination::Write(_) => report.extracted += 1,
                }
            } else {
                report.extracted += 1;
            }
            archive = header.skip().map_err(|err| map_unrar(err, had_password))?;
            progress.end_entry();
            continue;
        }
        if header.entry().is_encrypted() && opts.password.is_none() {
            if opts.skip_encrypted {
                report.skipped += 1;
                report
                    .warnings
                    .push(format!("skipped encrypted entry `{name}`"));
                archive = header.skip().map_err(|err| map_unrar(err, had_password))?;
                progress.end_entry();
                continue;
            }
            return Err(Error::Encrypted(name));
        }
        let modified = dos_time(header.entry().file_time);
        match destination(&name, opts, opts.flat, false, modified, decide)? {
            Destination::Skip => {
                report.skipped += 1;
                archive = header.skip().map_err(|err| map_unrar(err, had_password))?;
            }
            Destination::Write(dest) => {
                let staged = stage_path(&dest);
                archive = match take_member(header, &staged, size, had_password) {
                    Ok(next) => next,
                    Err(err) => {
                        if !opts.keep_broken {
                            let _ = std::fs::remove_file(&staged);
                        }
                        return Err(err);
                    }
                };
                finish_staged(path, &staged, &dest, &name, opts, modified, &mut report)?;
                budget.charge(size)?;
            }
        }
        progress.end_entry();
    }
    Ok(report)
}

/// Copy member `index` into `sink`.
pub fn copy_rar_index(
    path: &Path,
    index: usize,
    sink: &mut dyn Write,
    password: Option<&Password>,
) -> Result<(u64, u32), Error> {
    let _lock = unrar_lock();
    refuse_multipart(path)?;
    let mut archive = open_process(path, password)?;
    let mut cursor = 0usize;
    loop {
        let Some(header) = archive
            .read_header()
            .map_err(|err| map_unrar(err, password.is_some()))?
        else {
            return Err(Error::Usage(format!("entry {index} is out of range")));
        };
        if cursor != index {
            cursor += 1;
            archive = header
                .skip()
                .map_err(|err| map_unrar(err, password.is_some()))?;
            continue;
        }
        if header.entry().is_directory() {
            return Ok((0, 0));
        }
        let name = header_name(header.entry())?;
        if header.entry().is_encrypted() && password.is_none() {
            return Err(Error::Encrypted(name));
        }
        let budget = Budget::for_archive(
            path,
            crate::options::DEFAULT_MAX_BYTES,
            crate::options::DEFAULT_MAX_RATIO,
        );
        budget.reserve(header.entry().unpacked_size, None)?;
        let size = header.entry().unpacked_size;
        let staged =
            std::env::temp_dir().join(format!("szip-rar-{}-{cursor}.bin", std::process::id()));
        let extracted = take_member(header, &staged, size, password.is_some());
        let archive_result = match extracted {
            Ok(next) => next,
            Err(err) => {
                let _ = std::fs::remove_file(&staged);
                return Err(err);
            }
        };
        drop(archive_result);
        let outcome = copy_staged(&staged, sink, &name);
        let _ = std::fs::remove_file(&staged);
        return outcome;
    }
}

fn list_hashed(path: &Path, password: Option<&Password>) -> Result<Vec<Entry>, Error> {
    let mut archive = open_process(path, password)?;
    let solid = archive.is_solid();
    let headers_encrypted = archive.has_encrypted_headers();
    let mut entries = Vec::new();
    loop {
        let Some(header) = archive
            .read_header()
            .map_err(|err| map_unrar(err, password.is_some()))?
        else {
            break;
        };
        let mut entry = entry_from(header.entry(), solid, headers_encrypted)?;
        if entry.kind != EntryKind::File || (entry.encrypted && password.is_none()) {
            archive = header
                .skip()
                .map_err(|err| map_unrar(err, password.is_some()))?;
            entries.push(entry);
            continue;
        }
        let size = header.entry().unpacked_size;
        let staged = std::env::temp_dir().join(format!(
            "szip-rar-hash-{}-{}.bin",
            std::process::id(),
            entries.len()
        ));
        archive = match take_member(header, &staged, size, password.is_some()) {
            Ok(next) => next,
            Err(err) => {
                let _ = std::fs::remove_file(&staged);
                return Err(err);
            }
        };
        if let Ok(hash) = hash_file(&staged) {
            entry.blake2sp = Some(hash);
        }
        let _ = std::fs::remove_file(&staged);
        entries.push(entry);
    }
    Ok(entries)
}

fn finish_staged(
    archive: &Path,
    staged: &Path,
    dest: &Path,
    name: &str,
    opts: &ExtractOpts,
    modified: Option<SystemTime>,
    report: &mut ExtractReport,
) -> Result<(), Error> {
    let meta = std::fs::symlink_metadata(staged)
        .map_err(|err| Error::io(format!("stat {}", staged.display()), err))?;
    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(staged)
            .map_err(|err| Error::io(format!("read link {}", staged.display()), err))?;
        let _ = std::fs::remove_file(staged);
        let target = target.to_str().ok_or_else(|| {
            Error::UnsafePath(format!("symlink `{name}` has a non-Unicode target"))
        })?;
        if !opts.allow_unsafe_links {
            crate::fs::safe_path::symlink_stays_inside(&opts.output, dest, target)?;
        }
        match crate::fs::metadata::create_symlink(target, dest) {
            Ok(()) => report.extracted += 1,
            Err(err) => report.warnings.push(err.to_string()),
        }
        return Ok(());
    }
    if dest.exists() {
        std::fs::remove_file(dest)
            .map_err(|err| Error::io(format!("replace {}", dest.display()), err))?;
    }
    std::fs::rename(staged, dest)
        .map_err(|err| Error::io(format!("rename {}", dest.display()), err))?;
    if let Err(err) = restore_entry_time(dest, modified, opts) {
        report.warnings.push(err.to_string());
    }
    if let Err(err) = mark_extracted(archive, dest, opts) {
        report.warnings.push(err.to_string());
    }
    report.extracted += 1;
    Ok(())
}

fn copy_staged(staged: &Path, sink: &mut dyn Write, name: &str) -> Result<(u64, u32), Error> {
    let mut file =
        File::open(staged).map_err(|err| Error::io(format!("open {}", staged.display()), err))?;
    let mut hasher = CrcWriter::new();
    let mut tee = Tee {
        left: &mut hasher,
        right: sink,
    };
    std::io::copy(&mut file, &mut tee).map_err(|err| Error::io(name.to_string(), err))?;
    Ok((hasher.bytes, hasher.crc()))
}

fn hash_file(path: &Path) -> Result<[u8; 32], Error> {
    let mut file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let mut hasher = Blake2spWriter::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?;
    Ok(hasher.finish())
}

fn open_listing(
    path: &Path,
    password: Option<&Password>,
) -> Result<OpenArchive<List, CursorBeforeHeader>, Error> {
    let owned = first_volume(path);
    archive_at(&owned, password)
        .open_for_listing()
        .map_err(|err| map_unrar(err, password.is_some()))
}

fn open_process(
    path: &Path,
    password: Option<&Password>,
) -> Result<OpenArchive<Process, CursorBeforeHeader>, Error> {
    let owned = first_volume(path);
    archive_at(&owned, password)
        .open_for_processing()
        .map_err(|err| map_unrar(err, password.is_some()))
}

fn archive_at<'a>(path: &'a Path, password: Option<&'a Password>) -> Archive<'a> {
    match password {
        Some(password) => Archive::with_password(path, password.expose().as_bytes()),
        None => Archive::new(path),
    }
}

/// UnRAR's volume-change callback in unrar 0.5.8 reads a 2048-wchar buffer
/// and is compiled with `RAR_SMP`. That combination AVs on Windows.
fn refuse_multipart(path: &Path) -> Result<(), Error> {
    let first = first_volume(path);
    if Archive::new(&first).is_multipart() {
        return Err(Error::Unsupported {
            format: "RAR".to_string(),
            detail: "extracting a split (multivolume) RAR is not supported".to_string(),
        });
    }
    Ok(())
}

/// Write `header` to `dest`. Small members go through `read()` so UnRAR never
/// creates the file; large members still use `extract_to`.
fn take_member(
    header: OpenArchive<Process, CursorBeforeFile>,
    dest: &Path,
    size: u64,
    had_password: bool,
) -> Result<OpenArchive<Process, CursorBeforeHeader>, Error> {
    if size > DECODE_IN_MEMORY {
        return header
            .extract_to(dest)
            .map_err(|err| map_unrar(err, had_password));
    }
    let (bytes, next) = header.read().map_err(|err| map_unrar(err, had_password))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > DECODE_IN_MEMORY {
        return Err(Error::Usage(format!(
            "refusing to decode {} bytes of a RAR member into memory",
            bytes.len()
        )));
    }
    std::fs::write(dest, bytes)
        .map_err(|err| Error::io(format!("write {}", dest.display()), err))?;
    Ok(next)
}

/// First volume that exists on disk.
///
/// `unrar` rewrites `.part12.rar` to `.part01.rar`. WinRAR `-v` writes
/// `.part1.rar`. Prefer a name that is actually present.
fn first_volume(path: &Path) -> PathBuf {
    let mut candidates = Vec::new();
    candidates.push(Archive::new(path).first_part());
    if let Some(unpadded) = unpadded_first(path) {
        candidates.push(unpadded);
    }
    candidates.push(path.to_path_buf());
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| path.to_path_buf())
}

fn unpadded_first(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_str()?;
    let marker = ".part";
    let idx = name.rfind(marker)?;
    let rest = &name[idx + marker.len()..];
    let dot = rest.find('.')?;
    if rest[..dot].is_empty() || !rest[..dot].chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let mut rewritten = name[..idx].to_string();
    rewritten.push_str(".part1");
    rewritten.push_str(&rest[dot..]);
    Some(path.with_file_name(rewritten))
}

fn entry_from(
    header: &unrar::FileHeader,
    solid: bool,
    headers_encrypted: bool,
) -> Result<Entry, Error> {
    let kind = if header.is_directory() {
        EntryKind::Dir
    } else if header.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    };
    Ok(Entry {
        path: header_name(header)?,
        kind,
        size: Some(header.unpacked_size),
        packed_size: None,
        modified: dos_time(header.file_time),
        mode: None,
        crc32: Some(header.file_crc),
        method: Some(rar_method(header.method)),
        host_os: None,
        blake2sp: None,
        link_target: None,
        encrypted: header.is_encrypted() || headers_encrypted,
        solid,
        dictionary: None,
    })
}

fn header_name(header: &unrar::FileHeader) -> Result<String, Error> {
    let text = header.filename.to_str().ok_or_else(|| {
        Error::UnsafePath(format!(
            "non-Unicode RAR name {}",
            header.filename.display()
        ))
    })?;
    Ok(text.replace('\\', "/").trim_end_matches('/').to_string())
}

fn rar_method(method: u32) -> String {
    match method {
        0x30 => "store".to_string(),
        0x31..=0x35 => format!("rar-m{}", method - 0x30),
        other => format!("rar-{other}"),
    }
}

fn dos_time(stamp: u32) -> Option<SystemTime> {
    if stamp == 0 {
        return None;
    }
    let time = stamp & 0xFFFF;
    let date = stamp >> 16;
    let second = (time & 0x1F) * 2;
    let minute = (time >> 5) & 0x3F;
    let hour = (time >> 11) & 0x1F;
    let day = date & 0x1F;
    let month = (date >> 5) & 0x0F;
    let year = 1980u32 + (date >> 9);
    let naive = chrono::NaiveDate::from_ymd_opt(i32::try_from(year).ok()?, month, day)?
        .and_hms_opt(hour, minute, second)?;
    let local = chrono::Local.from_local_datetime(&naive).single()?;
    let secs = local.timestamp();
    if secs < 0 {
        return None;
    }
    Some(UNIX_EPOCH + std::time::Duration::from_secs(u64::try_from(secs).ok()?))
}

fn map_unrar(err: UnrarError, had_password: bool) -> Error {
    match err.code {
        Code::MissingPassword => {
            if had_password {
                Error::BadPassword
            } else {
                Error::Encrypted("archive headers".to_string())
            }
        }
        Code::BadPassword => Error::BadPassword,
        Code::BadData => Error::BadChecksum {
            entry: err.to_string(),
        },
        Code::EOpen | Code::ERead | Code::ECreate | Code::EWrite | Code::EClose => {
            Error::io(err.to_string(), std::io::Error::other(err.to_string()))
        }
        _ => Error::Archive(err.to_string()),
    }
}

struct Tee<'a> {
    left: &'a mut CrcWriter,
    right: &'a mut dyn Write,
}

impl Write for Tee<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.left.write_all(buf)?;
        self.right.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.left.flush()?;
        self.right.flush()
    }
}
