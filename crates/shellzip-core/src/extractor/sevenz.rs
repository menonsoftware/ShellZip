//! 7z listing, testing, and extraction.
//!
//! LZMA, LZMA2, BZip2, Deflate, and AES are provided by `sevenz-rust2`. A
//! missing password on an encrypted archive is exit 11, and so is a wrong one.
//! Solid blocks are decoded in order; unselected members are discarded after
//! they are read so the next member stays aligned.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::SystemTime;

use sevenz_rust2::{ArchiveEntry, ArchiveReader, EncoderMethod, Password as SevenPassword};

use super::policy::{Budget, Destination, destination, mark_extracted, symlink_destination};
use crate::crypto::Password;
use crate::entry::{Entry, EntryKind};
use crate::error::Error;
use crate::hash::{Blake2spWriter, CrcWriter};
use crate::options::{ConflictDecider, ExtractOpts, ExtractReport, TestReport};
use crate::progress::{ProgressReader, ProgressSink};

const REPARSE_POINT: u32 = 0x400;

/// List 7z members. `hash` decodes payloads for BLAKE2sp.
pub fn list_sevenz(
    path: &Path,
    hash: bool,
    password: Option<&Password>,
) -> Result<Vec<Entry>, Error> {
    let mut reader = open_reader(path, password)?;
    let solid = reader.archive().is_solid;
    let files = reader.archive().files.clone();
    let mut entries = Vec::with_capacity(files.len());
    for file in &files {
        let (method, encrypted) = methods(&reader, file.name());
        entries.push(entry_from(file, &method, encrypted, solid));
    }
    if !hash {
        return Ok(entries);
    }
    let encrypted_paths: Vec<String> = entries
        .iter()
        .filter(|entry| entry.encrypted)
        .map(|entry| entry.path.clone())
        .collect();
    let mut hashes = HashMap::new();
    let mut failure = None;
    let decoded = reader.for_each_entries(|file, data| {
        if failure.is_some() {
            return Ok(false);
        }
        let name = file.name.trim_end_matches('/').to_string();
        if file.is_directory || !file.has_stream || encrypted_paths.iter().any(|item| item == &name)
        {
            return drain(data).map(|()| true).map_err(io_as_sevenz);
        }
        let mut hasher = Blake2spWriter::new();
        match std::io::copy(data, &mut hasher) {
            Ok(_) => {
                hashes.insert(name, hasher.finish());
                Ok(true)
            }
            Err(err) => {
                failure = Some(Error::io(file.name.clone(), err));
                Ok(false)
            }
        }
    });
    if let Some(err) = failure {
        return Err(err);
    }
    decoded.map_err(|err| map_sevenz(err, password.is_some()))?;
    for entry in &mut entries {
        if let Some(hash) = hashes.get(&entry.path) {
            entry.blake2sp = Some(*hash);
        }
    }
    Ok(entries)
}

/// Decode every member and compare a stored CRC32 when the header has one.
pub fn test_sevenz(
    path: &Path,
    progress: &dyn ProgressSink,
    password: Option<&Password>,
) -> Result<TestReport, Error> {
    let mut reader = open_reader(path, password)?;
    let encrypted = encrypted_names(&reader);
    progress.begin(&format!("test {}", path.display()), 0);
    let mut report = TestReport::default();
    if password.is_none() && !encrypted.is_empty() {
        report.entries = u64::try_from(reader.archive().files.len()).unwrap_or(u64::MAX);
        report.encrypted = encrypted;
        return Ok(report);
    }
    let mut failure = None;
    let decoded = reader.for_each_entries(|file, data| {
        if failure.is_some() {
            return Ok(false);
        }
        let name = file.name().trim_end_matches('/').to_string();
        report.entries += 1;
        if file.is_directory() {
            progress.begin_entry(&name, 0);
            progress.end_entry();
            return Ok(true);
        }
        progress.begin_entry(&name, file.size());
        let mut hasher = CrcWriter::new();
        let mut reader = ProgressReader::new(data, progress);
        if let Err(err) = std::io::copy(&mut reader, &mut hasher) {
            if is_checksum(&err) {
                report.checksums.push(name);
                progress.end_entry();
                return Ok(false);
            }
            failure = Some(Error::io(name, err));
            return Ok(false);
        }
        report.bytes += hasher.bytes;
        if file.has_crc && u64::from(hasher.crc()) != file.crc {
            report.checksums.push(name);
        }
        progress.end_entry();
        Ok(true)
    });
    if let Some(err) = failure {
        return Err(err);
    }
    decoded.map_err(|err| map_sevenz(err, password.is_some()))?;
    Ok(report)
}

/// Extract selected members.
pub fn extract_sevenz(
    path: &Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    selected: &dyn Fn(&str) -> bool,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    let mut reader = open_reader(path, opts.password.as_ref())?;
    let encrypted = encrypted_names(&reader);
    let mut report = ExtractReport::default();
    if opts.password.is_none() {
        let files = reader.archive().files.clone();
        let mut plain = false;
        for file in &files {
            let name = file.name.trim_end_matches('/').to_string();
            if name.is_empty() || !selected(&name) || !file.has_stream {
                continue;
            }
            if encrypted.iter().any(|item| item == &name) {
                if !opts.skip_encrypted {
                    return Err(Error::Encrypted(name));
                }
            } else {
                plain = true;
            }
        }
        if !plain {
            for file in &files {
                let name = file.name.trim_end_matches('/').to_string();
                if name.is_empty() || !selected(&name) {
                    continue;
                }
                if encrypted.iter().any(|item| item == &name) {
                    report.skipped += 1;
                    report
                        .warnings
                        .push(format!("skipped encrypted entry `{name}`"));
                } else if file.is_directory && !opts.flat {
                    destination(&name, opts, false, true, decide)?;
                    report.extracted += 1;
                }
            }
            return Ok(report);
        }
    }
    progress.begin(&format!("extract {}", path.display()), 0);
    let mut budget = Budget::new(opts.max_bytes);
    let mut failure = None;
    let had_password = opts.password.is_some();
    let decoded = reader.for_each_entries(|file, data| {
        if failure.is_some() {
            return Ok(false);
        }
        let ctx = MemberWrite {
            archive: path,
            opts,
            decide,
            selected,
            encrypted: &encrypted,
            progress,
        };
        match write_member(&ctx, file, data, &mut budget, &mut report) {
            Ok(()) => Ok(true),
            Err(err) => {
                failure = Some(err);
                Ok(false)
            }
        }
    });
    if let Some(err) = failure {
        return Err(err);
    }
    decoded.map_err(|err| map_sevenz(err, had_password))?;
    Ok(report)
}

/// Copy the member at `index` (in header order) into `sink`.
pub fn copy_sevenz_index(
    path: &Path,
    index: usize,
    sink: &mut dyn Write,
    password: Option<&Password>,
) -> Result<(u64, u32), Error> {
    let mut reader = open_reader(path, password)?;
    let name = reader
        .archive()
        .files
        .get(index)
        .map(|file| file.name.clone())
        .ok_or_else(|| Error::Usage(format!("entry {index} is out of range")))?;
    let target_is_dir = reader
        .archive()
        .files
        .get(index)
        .is_some_and(|file| file.is_directory);
    if target_is_dir {
        return Ok((0, 0));
    }
    let mut found = None;
    let mut failure = None;
    let decoded = reader.for_each_entries(|file, data| {
        if failure.is_some() {
            return Ok(false);
        }
        if file.name != name {
            return drain(data).map(|()| true).map_err(io_as_sevenz);
        }
        let mut hasher = CrcWriter::new();
        let mut tee = Tee {
            left: &mut hasher,
            right: sink,
        };
        match std::io::copy(data, &mut tee) {
            Ok(_) => {
                found = Some((hasher.bytes, hasher.crc()));
                Ok(false)
            }
            Err(err) => {
                failure = Some(Error::io(name.clone(), err));
                Ok(false)
            }
        }
    });
    if let Some(err) = failure {
        return Err(err);
    }
    decoded.map_err(|err| map_sevenz(err, password.is_some()))?;
    found.ok_or_else(|| Error::Usage(format!("entry {index} is out of range")))
}

struct MemberWrite<'a> {
    archive: &'a Path,
    opts: &'a ExtractOpts,
    decide: Option<&'a ConflictDecider<'a>>,
    selected: &'a dyn Fn(&str) -> bool,
    encrypted: &'a [String],
    progress: &'a dyn ProgressSink,
}

fn write_member(
    ctx: &MemberWrite<'_>,
    file: &ArchiveEntry,
    data: &mut dyn Read,
    budget: &mut Budget,
    report: &mut ExtractReport,
) -> Result<(), Error> {
    let name = file.name.trim_end_matches('/').to_string();
    if name.is_empty() || !(ctx.selected)(&name) {
        drain(data)?;
        return Ok(());
    }
    if ctx.encrypted.iter().any(|item| item == &name) && ctx.opts.skip_encrypted {
        drain(data)?;
        report.skipped += 1;
        report
            .warnings
            .push(format!("skipped encrypted entry `{name}`"));
        return Ok(());
    }
    budget.reserve(file.size)?;
    ctx.progress.begin_entry(&name, file.size);
    if file.is_directory {
        if !ctx.opts.flat {
            destination(&name, ctx.opts, false, true, ctx.decide)?;
        }
        report.extracted += 1;
        ctx.progress.end_entry();
        return Ok(());
    }
    let reparse = file.has_windows_attributes && (file.windows_attributes & REPARSE_POINT) != 0;
    if reparse {
        let mut target = String::new();
        data.read_to_string(&mut target)
            .map_err(|err| Error::io(name.clone(), err))?;
        budget.charge(file.size)?;
        let target = target.trim_end_matches('\0').to_string();
        let link = symlink_destination(&name, &target, ctx.opts)?;
        match crate::fs::metadata::create_symlink(&target, &link) {
            Ok(()) => report.extracted += 1,
            Err(err) => report.warnings.push(err.to_string()),
        }
        ctx.progress.end_entry();
        return Ok(());
    }
    match destination(&name, ctx.opts, ctx.opts.flat, false, ctx.decide)? {
        Destination::Skip => {
            drain(data)?;
            report.skipped += 1;
        }
        Destination::Write(dest) => {
            let modified = file
                .has_last_modified_date
                .then(|| SystemTime::from(file.last_modified_date));
            super::policy::write_atomic(&dest, ctx.opts.keep_broken, |out| {
                let mut reader = ProgressReader::new(data, ctx.progress);
                let mut buf = Vec::new();
                let copied = crate::fs::copy_chunked(&mut reader, out, &mut buf)
                    .map_err(|err| Error::io(name.clone(), err))?;
                budget.charge(copied)?;
                Ok(())
            })?;
            if let Some(modified) = modified
                && let Err(err) = crate::fs::metadata::restore_mtime(&dest, modified)
            {
                report.warnings.push(err.to_string());
            }
            if let Err(err) = mark_extracted(ctx.archive, &dest, ctx.opts) {
                report.warnings.push(err.to_string());
            }
            report.extracted += 1;
        }
    }
    ctx.progress.end_entry();
    Ok(())
}

fn open_reader(path: &Path, password: Option<&Password>) -> Result<ArchiveReader<File>, Error> {
    let seven = match password {
        Some(password) => SevenPassword::from(password.expose()),
        None => SevenPassword::empty(),
    };
    ArchiveReader::open(path, seven).map_err(|err| map_sevenz(err, password.is_some()))
}

fn methods(reader: &ArchiveReader<File>, name: &str) -> (String, bool) {
    let mut found = Vec::new();
    if reader.file_compression_methods(name, &mut found).is_err() {
        return ("-".to_string(), false);
    }
    let encrypted = found.iter().any(|method| method_is_aes(*method));
    if found.is_empty() {
        return ("-".to_string(), encrypted);
    }
    let label = found
        .iter()
        .map(|method| method_label(*method))
        .collect::<Vec<_>>()
        .join("+");
    (label, encrypted)
}

fn method_is_aes(method: EncoderMethod) -> bool {
    method_label(method).to_ascii_lowercase().contains("aes")
}

fn method_label(method: EncoderMethod) -> String {
    let raw = format!("{method:?}");
    let Some(start) = raw.find('"') else {
        return raw;
    };
    let rest = &raw[start + 1..];
    let Some(end) = rest.find('"') else {
        return raw;
    };
    rest[..end].to_string()
}

fn encrypted_names(reader: &ArchiveReader<File>) -> Vec<String> {
    reader
        .archive()
        .files
        .iter()
        .filter(|file| methods(reader, file.name()).1)
        .map(|file| file.name.trim_end_matches('/').to_string())
        .collect()
}

fn entry_from(file: &ArchiveEntry, method: &str, encrypted: bool, solid: bool) -> Entry {
    Entry {
        path: file.name.trim_end_matches('/').to_string(),
        kind: if file.is_directory {
            EntryKind::Dir
        } else {
            EntryKind::File
        },
        size: Some(file.size),
        packed_size: Some(file.compressed_size),
        modified: file
            .has_last_modified_date
            .then(|| SystemTime::from(file.last_modified_date)),
        mode: None,
        crc32: file.has_crc.then(|| u32::try_from(file.crc).ok()).flatten(),
        method: Some(method.to_string()),
        host_os: file.has_windows_attributes.then(|| "Windows".to_string()),
        blake2sp: None,
        link_target: None,
        encrypted,
        solid,
        dictionary: None,
    }
}

fn drain(data: &mut dyn Read) -> Result<(), std::io::Error> {
    std::io::copy(data, &mut std::io::sink())?;
    Ok(())
}

fn is_checksum(err: &std::io::Error) -> bool {
    err.get_ref()
        .and_then(|inner| inner.downcast_ref::<sevenz_rust2::Error>())
        .is_some_and(|inner| matches!(inner, sevenz_rust2::Error::ChecksumVerificationFailed))
        || err.to_string().to_ascii_lowercase().contains("checksum")
}

fn io_as_sevenz(err: std::io::Error) -> sevenz_rust2::Error {
    sevenz_rust2::Error::Other(std::borrow::Cow::Owned(err.to_string()))
}

fn map_sevenz(err: sevenz_rust2::Error, had_password: bool) -> Error {
    match err {
        sevenz_rust2::Error::PasswordRequired | sevenz_rust2::Error::MaybeBadPassword(_) => {
            if had_password {
                Error::BadPassword
            } else {
                Error::Encrypted("archive headers".to_string())
            }
        }
        sevenz_rust2::Error::ChecksumVerificationFailed
        | sevenz_rust2::Error::NextHeaderCrcMismatch => Error::BadChecksum {
            entry: err.to_string(),
        },
        sevenz_rust2::Error::FileNotFound => Error::NoFiles,
        other => Error::Archive(other.to_string()),
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
