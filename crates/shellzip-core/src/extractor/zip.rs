//! ZIP listing, testing, and extraction.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::TimeZone;
use zip::ZipArchive;

use super::policy::{Budget, Destination, destination, symlink_destination, write_atomic};
use crate::entry::{Entry, EntryKind};
use crate::error::Error;
use crate::hash::{Blake2spWriter, CrcWriter};
use crate::options::{ConflictDecider, ExtractOpts, ExtractReport, TestReport};
use crate::progress::{ProgressReader, ProgressSink};
use zip::HasZipMetadata;

/// List every ZIP member. Symlink targets are read when they are small.
///
/// `hash` streams each file payload through BLAKE2sp.
pub fn list_zip(path: &Path, hash: bool) -> Result<Vec<Entry>, Error> {
    let mut archive = open_zip(path)?;
    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|err| annotate(err, path))?;
        let mut entry = entry_from(&mut file)?;
        if hash && entry.kind == EntryKind::File && !entry.encrypted {
            let mut blake = Blake2spWriter::new();
            std::io::copy(&mut file, &mut blake)
                .map_err(|err| Error::io(entry.path.clone(), err))?;
            entry.blake2sp = Some(blake.finish());
        }
        entries.push(entry);
    }
    Ok(entries)
}

/// Decode every member and compare CRC32 values.
pub fn test_zip(path: &Path, progress: &dyn ProgressSink) -> Result<TestReport, Error> {
    let mut archive = open_zip(path)?;
    let total: u64 = (0..archive.len())
        .map(|index| archive.by_index(index).map(|file| file.size()).unwrap_or(0))
        .sum();
    progress.begin(&format!("test {}", path.display()), total);
    let mut report = TestReport::default();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|err| annotate(err, path))?;
        let name = file.name().to_string();
        report.entries += 1;
        if file.is_dir() {
            progress.begin_entry(&name, 0);
            progress.end_entry();
            continue;
        }
        if file.encrypted() {
            report.encrypted.push(name);
            continue;
        }
        progress.begin_entry(&name, file.size());
        let expected = file.crc32();
        let mut hasher = CrcWriter::new();
        let mut reader = ProgressReader::new(&mut file, progress);
        if let Err(err) = std::io::copy(&mut reader, &mut hasher) {
            let mapped = Error::io(name.clone(), err);
            match mapped {
                Error::BadChecksum { entry } => report.checksums.push(entry),
                other => return Err(other),
            }
            continue;
        }
        report.bytes += hasher.bytes;
        if hasher.crc() != expected {
            report.checksums.push(name);
        }
        progress.end_entry();
    }
    Ok(report)
}

/// Extract selected members. `selected` receives the raw archive name.
pub fn extract_zip(
    path: &Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    selected: &dyn Fn(&str) -> bool,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    let mut archive = open_zip(path)?;
    let chosen: Vec<usize> = (0..archive.len())
        .filter(|&index| {
            archive
                .by_index(index)
                .map(|file| selected(file.name()))
                .unwrap_or(false)
        })
        .collect();
    let total: u64 = chosen
        .iter()
        .filter_map(|&index| archive.by_index(index).ok().map(|file| file.size()))
        .sum();
    progress.begin(&format!("extract {}", path.display()), total);
    let mut report = ExtractReport::default();
    let mut budget = Budget::new(opts.max_bytes);
    for index in chosen {
        let mut file = archive.by_index(index).map_err(|err| annotate(err, path))?;
        let name = file.name().to_string();
        if file.encrypted() {
            return Err(Error::Encrypted(name));
        }
        progress.begin_entry(&name, file.size());
        budget.reserve(file.size())?;
        if file.is_dir() {
            if !opts.flat {
                destination(&name, opts, false, true, decide)?;
            }
            report.extracted += 1;
            progress.end_entry();
            continue;
        }
        if file.is_symlink() {
            let mut target = String::new();
            let mut reader = ProgressReader::new(&mut file, progress);
            reader
                .read_to_string(&mut target)
                .map_err(|err| Error::io(name.clone(), err))?;
            budget.charge(target.len() as u64)?;
            let link = symlink_destination(&name, target.trim_end_matches('\0'), opts)?;
            match crate::fs::metadata::create_symlink(target.trim_end_matches('\0'), &link) {
                Ok(()) => report.extracted += 1,
                Err(err) => report.warnings.push(format!("{err}")),
            }
            progress.end_entry();
            continue;
        }
        match destination(&name, opts, opts.flat, false, decide)? {
            Destination::Skip => report.skipped += 1,
            Destination::Write(dest) => {
                let modified = zip_time(file.last_modified());
                let mode = file.unix_mode();
                write_atomic(&dest, opts.keep_broken, |out| {
                    let mut reader = ProgressReader::new(&mut file, progress);
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
                if let Some(mode) = mode
                    && let Err(err) = crate::fs::metadata::restore_mode(&dest, mode)
                {
                    report.warnings.push(err.to_string());
                }
                report.extracted += 1;
            }
        }
        progress.end_entry();
    }
    Ok(report)
}

/// Copy one member, by index, into `sink`.
pub fn extract_zip_index(
    path: &Path,
    index: usize,
    sink: &mut dyn Write,
    progress: &dyn ProgressSink,
) -> Result<(u64, u32), Error> {
    let mut archive = open_zip(path)?;
    let mut file = archive.by_index(index).map_err(|err| annotate(err, path))?;
    if file.encrypted() {
        return Err(Error::Encrypted(file.name().to_string()));
    }
    let name = file.name().to_string();
    progress.begin_entry(&name, file.size());
    let mut hasher = CrcWriter::new();
    {
        let mut reader = ProgressReader::new(&mut file, progress);
        let mut tee = Tee {
            left: &mut hasher,
            right: sink,
        };
        std::io::copy(&mut reader, &mut tee).map_err(|err| Error::io(name.clone(), err))?;
    }
    progress.end_entry();
    if hasher.crc() != file.crc32() {
        return Err(Error::BadChecksum { entry: name });
    }
    Ok((hasher.bytes, hasher.crc()))
}

fn open_zip(path: &Path) -> Result<ZipArchive<File>, Error> {
    let file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    ZipArchive::new(file).map_err(|err| annotate(err, path))
}

fn annotate(err: zip::result::ZipError, path: &Path) -> Error {
    match Error::from(err) {
        Error::Archive(message) => Error::Archive(format!("{message} ({})", path.display())),
        other => other,
    }
}

fn entry_from(file: &mut zip::read::ZipFile<'_, File>) -> Result<Entry, Error> {
    let path = file.name().trim_end_matches('/').to_string();
    let kind = if file.is_symlink() {
        EntryKind::Symlink
    } else if file.is_dir() {
        EntryKind::Dir
    } else {
        EntryKind::File
    };
    let link_target = if kind == EntryKind::Symlink && !file.encrypted() && file.size() <= 8192 {
        let mut text = String::new();
        file.read_to_string(&mut text).ok();
        Some(text.trim_end_matches('\0').to_string())
    } else {
        None
    };
    Ok(Entry {
        path,
        kind,
        size: Some(file.size()),
        packed_size: Some(file.compressed_size()),
        modified: zip_time(file.last_modified()),
        mode: file.unix_mode(),
        crc32: Some(file.crc32()),
        method: Some(method_name(file.compression())),
        host_os: Some(host_label(&format!("{:?}", file.get_metadata().system))),
        blake2sp: None,
        link_target,
        encrypted: file.encrypted(),
    })
}

fn host_label(system: &str) -> String {
    match system {
        "Dos" => "DOS".to_string(),
        "Unix" => "Unix".to_string(),
        other => other.to_string(),
    }
}

fn method_name(method: zip::CompressionMethod) -> String {
    if method == zip::CompressionMethod::Stored {
        "store".to_string()
    } else if method == zip::CompressionMethod::Deflated {
        "deflate".to_string()
    } else if method == zip::CompressionMethod::Zstd {
        "zstd".to_string()
    } else {
        format!("{method:?}")
    }
}

fn zip_time(stamp: Option<zip::DateTime>) -> Option<SystemTime> {
    let stamp = stamp?;
    let naive = chrono::NaiveDate::from_ymd_opt(
        i32::from(stamp.year()),
        u32::from(stamp.month()),
        u32::from(stamp.day()),
    )?
    .and_hms_opt(
        u32::from(stamp.hour()),
        u32::from(stamp.minute()),
        u32::from(stamp.second()),
    )?;
    let local = chrono::Local.from_local_datetime(&naive).single()?;
    let secs = local.timestamp();
    if secs < 0 {
        return None;
    }
    Some(UNIX_EPOCH + std::time::Duration::from_secs(secs as u64))
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
