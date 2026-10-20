//! Tar, tar.gz, tar.zst, and single-file gzip/zstd streams.

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use flate2::read::GzDecoder;
use tar::{Archive, EntryType};

use super::policy::{Budget, Destination, destination, symlink_destination, write_atomic};
use crate::detect::{Compression, prefix_is_tar};
use crate::entry::{Entry, EntryKind};
use crate::error::Error;
use crate::hash::CrcWriter;
use crate::options::{ConflictDecider, ExtractOpts, ExtractReport, TestReport};
use crate::progress::{ProgressReader, ProgressSink};

struct Input(Box<dyn Read + Send>);

impl Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

/// True when the gzip or zstd payload begins with a tar header.
pub fn payload_is_tar(path: &Path, compression: Compression) -> Result<bool, Error> {
    let mut input = open_input(path, compression)?;
    let mut prefix = [0u8; 512];
    let mut filled = 0;
    while filled < prefix.len() {
        match input.read(&mut prefix[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(err) => return Err(Error::io(format!("read {}", path.display()), err)),
        }
    }
    Ok(prefix_is_tar(&prefix[..filled]))
}

/// List a tar stream.
pub fn list_tar(path: &Path, compression: Compression) -> Result<Vec<Entry>, Error> {
    let mut archive = Archive::new(open_input(path, compression)?);
    let mut entries = Vec::new();
    for entry in archive
        .entries()
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?
    {
        let mut entry = entry.map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        entries.push(tar_entry(&mut entry)?);
    }
    Ok(entries)
}

/// Test a tar stream by reading every payload and checking declared sizes.
pub fn test_tar(
    path: &Path,
    compression: Compression,
    progress: &dyn ProgressSink,
) -> Result<TestReport, Error> {
    progress.begin(&format!("test {}", path.display()), 0);
    let mut archive = Archive::new(open_input(path, compression)?);
    let mut report = TestReport::default();
    for entry in archive
        .entries()
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?
    {
        let mut entry = entry.map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        let name = entry_name(&entry)?;
        let declared = entry.size();
        report.entries += 1;
        if entry.header().entry_type().is_dir() {
            progress.begin_entry(&name, 0);
            progress.end_entry();
            continue;
        }
        progress.begin_entry(&name, declared);
        let mut hasher = CrcWriter::new();
        let mut reader = ProgressReader::new(&mut entry, progress);
        std::io::copy(&mut reader, &mut hasher).map_err(|err| Error::io(name.clone(), err))?;
        if hasher.bytes != declared {
            report.checksums.push(format!(
                "{name} (read {} bytes, header says {declared})",
                hasher.bytes
            ));
        }
        report.bytes += hasher.bytes;
        progress.end_entry();
    }
    Ok(report)
}

/// Extract a tar stream.
pub fn extract_tar(
    path: &Path,
    compression: Compression,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    selected: &dyn Fn(&str) -> bool,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    progress.begin(&format!("extract {}", path.display()), 0);
    let mut archive = Archive::new(open_input(path, compression)?);
    let mut report = ExtractReport::default();
    let mut budget = Budget::new(opts.max_bytes);
    for entry in archive
        .entries()
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?
    {
        let mut entry = entry.map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        let name = entry_name(&entry)?;
        if name.is_empty() || !selected(&name) {
            std::io::copy(&mut entry, &mut std::io::sink()).map_err(|err| Error::io(name, err))?;
            continue;
        }
        let kind = entry.header().entry_type();
        let declared = entry.size();
        progress.begin_entry(&name, declared);
        budget.reserve(declared)?;
        match kind {
            EntryType::Directory => {
                if !opts.flat {
                    destination(&name, opts, false, true, decide)?;
                    if let Ok(mtime) = entry.header().mtime()
                        && let Some(path) = dir_path(&name, opts)
                    {
                        let _ = crate::fs::metadata::restore_mtime(
                            &path,
                            UNIX_EPOCH + std::time::Duration::from_secs(mtime),
                        );
                    }
                }
                report.extracted += 1;
            }
            EntryType::Symlink => {
                let target = entry
                    .link_name()
                    .map_err(|err| Error::io(format!("link {name}"), err))?
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                std::io::copy(&mut entry, &mut std::io::sink())
                    .map_err(|err| Error::io(name.clone(), err))?;
                let link = symlink_destination(&name, &target, opts)?;
                match crate::fs::metadata::create_symlink(&target, &link) {
                    Ok(()) => report.extracted += 1,
                    Err(err) => report.warnings.push(err.to_string()),
                }
            }
            EntryType::Regular | EntryType::Continuous => {
                extract_regular(
                    &mut entry,
                    &name,
                    opts,
                    decide,
                    &mut budget,
                    &mut report,
                    progress,
                )?;
            }
            EntryType::Link => {
                std::io::copy(&mut entry, &mut std::io::sink())
                    .map_err(|err| Error::io(name.clone(), err))?;
                report.warnings.push(format!("skipping hard link `{name}`"));
            }
            _ => {
                std::io::copy(&mut entry, &mut std::io::sink())
                    .map_err(|err| Error::io(name.clone(), err))?;
                report
                    .warnings
                    .push(format!("skipping unsupported member `{name}`"));
            }
        }
        progress.end_entry();
    }
    Ok(report)
}

/// List one gzip or zstd member.
pub fn list_single(path: &Path, compression: Compression) -> Result<Vec<Entry>, Error> {
    let name = single_name(path, compression)?;
    let size = match compression {
        Compression::Gzip => gzip_isize(path),
        Compression::Zstd => zstd_content_size(path),
        Compression::None => None,
    };
    Ok(vec![Entry {
        path: name,
        kind: EntryKind::File,
        size,
        packed_size: std::fs::metadata(path).ok().map(|meta| meta.len()),
        modified: None,
        mode: None,
        crc32: None,
        method: Some(
            match compression {
                Compression::Gzip => "gzip",
                Compression::Zstd => "zstd",
                Compression::None => "store",
            }
            .to_string(),
        ),
        link_target: None,
        encrypted: false,
    }])
}

/// Test one gzip or zstd member by decoding it fully.
pub fn test_single(
    path: &Path,
    compression: Compression,
    progress: &dyn ProgressSink,
) -> Result<TestReport, Error> {
    let name = single_name(path, compression)?;
    progress.begin(&format!("test {}", path.display()), 0);
    progress.begin_entry(&name, 0);
    let mut input = open_input(path, compression)?;
    let mut hasher = CrcWriter::new();
    let mut reader = ProgressReader::new(&mut input, progress);
    std::io::copy(&mut reader, &mut hasher).map_err(|err| Error::io(name, err))?;
    progress.end_entry();
    Ok(TestReport {
        entries: 1,
        bytes: hasher.bytes,
        ..TestReport::default()
    })
}

/// Extract one gzip or zstd member.
pub fn extract_single(
    path: &Path,
    compression: Compression,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    selected: &dyn Fn(&str) -> bool,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    let name = single_name(path, compression)?;
    if !selected(&name) {
        return Ok(ExtractReport::default());
    }
    progress.begin(&format!("extract {}", path.display()), 0);
    progress.begin_entry(&name, 0);
    let mut report = ExtractReport::default();
    let mut budget = Budget::new(opts.max_bytes);
    let mut input = open_input(path, compression)?;
    match destination(&name, opts, opts.flat, false, decide)? {
        Destination::Skip => report.skipped += 1,
        Destination::Write(dest) => {
            write_atomic(&dest, opts.keep_broken, |out| {
                let mut reader = ProgressReader::new(&mut input, progress);
                let mut buf = Vec::new();
                let copied = crate::fs::copy_chunked(&mut reader, out, &mut buf)
                    .map_err(|err| Error::io(name.clone(), err))?;
                budget.charge(copied)?;
                Ok(())
            })?;
            report.extracted += 1;
        }
    }
    progress.end_entry();
    Ok(report)
}

fn extract_regular(
    entry: &mut tar::Entry<'_, Input>,
    name: &str,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    budget: &mut Budget,
    report: &mut ExtractReport,
    progress: &dyn ProgressSink,
) -> Result<(), Error> {
    let declared = entry.size();
    match destination(name, opts, opts.flat, false, decide)? {
        Destination::Skip => {
            std::io::copy(entry, &mut std::io::sink())
                .map_err(|err| Error::io(name.to_string(), err))?;
            report.skipped += 1;
        }
        Destination::Write(dest) => {
            let modified = entry
                .header()
                .mtime()
                .ok()
                .map(|secs| UNIX_EPOCH + std::time::Duration::from_secs(secs));
            let mode = entry.header().mode().ok();
            write_atomic(&dest, opts.keep_broken, |out| {
                let mut reader = ProgressReader::new(entry, progress);
                let mut buf = Vec::new();
                let copied = crate::fs::copy_chunked(&mut reader, out, &mut buf)
                    .map_err(|err| Error::io(name.to_string(), err))?;
                if copied != declared {
                    return Err(Error::BadChecksum {
                        entry: format!("{name} (read {copied} bytes, header says {declared})"),
                    });
                }
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
    Ok(())
}

fn tar_entry(entry: &mut tar::Entry<'_, Input>) -> Result<Entry, Error> {
    let path = entry_name(entry)?;
    let kind = match entry.header().entry_type() {
        EntryType::Directory => EntryKind::Dir,
        EntryType::Symlink => EntryKind::Symlink,
        EntryType::Regular | EntryType::Continuous => EntryKind::File,
        _ => EntryKind::Other,
    };
    let link_target = entry
        .link_name()
        .ok()
        .flatten()
        .map(|p| p.to_string_lossy().into_owned());
    Ok(Entry {
        path,
        kind,
        size: Some(entry.size()),
        packed_size: None,
        modified: entry
            .header()
            .mtime()
            .ok()
            .map(|secs| UNIX_EPOCH + std::time::Duration::from_secs(secs)),
        mode: entry.header().mode().ok(),
        crc32: None,
        method: None,
        link_target,
        encrypted: false,
    })
}

fn entry_name(entry: &tar::Entry<'_, Input>) -> Result<String, Error> {
    let path = entry.path().map_err(|err| Error::io("tar path", err))?;
    let mut text = path.to_string_lossy().replace('\\', "/");
    while let Some(rest) = text.strip_prefix("./") {
        text = rest.to_string();
    }
    Ok(text.trim_end_matches('/').to_string())
}

fn dir_path(name: &str, opts: &ExtractOpts) -> Option<std::path::PathBuf> {
    let rel = crate::fs::safe_path::safe_rel(name).ok()?;
    crate::fs::safe_path::safe_join(&opts.output, &rel).ok()
}

fn open_input(path: &Path, compression: Compression) -> Result<Input, Error> {
    let file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let reader = BufReader::with_capacity(256 * 1024, file);
    let boxed: Box<dyn Read + Send> = match compression {
        Compression::None => Box::new(reader),
        Compression::Gzip => Box::new(GzDecoder::new(reader)),
        Compression::Zstd => Box::new(
            zstd::stream::read::Decoder::new(reader)
                .map_err(|err| Error::io(format!("zstd {}", path.display()), err))?,
        ),
    };
    Ok(Input(boxed))
}

fn single_name(path: &Path, compression: Compression) -> Result<String, Error> {
    if compression == Compression::Gzip
        && let Some(name) = gzip_filename(path)?
        && crate::fs::safe_path::safe_rel(&name).is_ok()
        && !name.contains(['/', '\\'])
    {
        return Ok(name);
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    if stem.is_empty() {
        Ok("output".to_string())
    } else {
        Ok(stem)
    }
}

fn gzip_filename(path: &Path) -> Result<Option<String>, Error> {
    let bytes = read_head(path, 64 * 1024)?;
    if bytes.len() < 10 || bytes[0] != 0x1f || bytes[1] != 0x8b {
        return Ok(None);
    }
    let flags = bytes[3];
    let mut index = 10usize;
    if flags & 0x04 != 0 {
        if index + 2 > bytes.len() {
            return Ok(None);
        }
        let extra = u16::from_le_bytes([bytes[index], bytes[index + 1]]) as usize;
        index += 2 + extra;
    }
    if flags & 0x08 == 0 {
        return Ok(None);
    }
    let start = index;
    while index < bytes.len() && bytes[index] != 0 {
        index += 1;
    }
    if index >= bytes.len() {
        return Ok(None);
    }
    Ok(String::from_utf8(bytes[start..index].to_vec()).ok())
}

fn gzip_isize(path: &Path) -> Option<u64> {
    let file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len < 4 {
        return None;
    }
    use std::io::{Seek, SeekFrom};
    let mut file = file;
    file.seek(SeekFrom::End(-4)).ok()?;
    let mut buf = [0u8; 4];
    std::io::Read::read_exact(&mut file, &mut buf).ok()?;
    Some(u64::from(u32::from_le_bytes(buf)))
}

fn zstd_content_size(path: &Path) -> Option<u64> {
    let bytes = read_head(path, 32).ok()?;
    if bytes.len() < 6 || bytes[0..4] != [0x28, 0xB5, 0x2F, 0xFD] {
        return None;
    }
    let descriptor = bytes[4];
    let fcs_flag = descriptor >> 6;
    let single = descriptor & 0x20 != 0;
    let dict = descriptor & 0x03;
    let mut index = 5usize;
    if !single {
        index += 1;
    }
    index += match dict {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 4,
        _ => return None,
    };
    let fcs_len = match (fcs_flag, single) {
        (0, true) => 1,
        (0, false) => 0,
        (1, _) => 2,
        (2, _) => 4,
        (3, _) => 8,
        _ => return None,
    };
    if fcs_len == 0 || index + fcs_len > bytes.len() {
        return None;
    }
    let mut raw = [0u8; 8];
    raw[..fcs_len].copy_from_slice(&bytes[index..index + fcs_len]);
    let mut value = u64::from_le_bytes(raw);
    if fcs_flag == 1 {
        value += 256;
    }
    Some(value)
}

fn read_head(path: &Path, limit: usize) -> Result<Vec<u8>, Error> {
    let mut file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let mut buf = vec![0u8; limit];
    let n = file
        .read(&mut buf)
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?;
    buf.truncate(n);
    Ok(buf)
}

/// Copy tar member `idx` into `sink`.
pub fn copy_tar_index(
    path: &Path,
    compression: Compression,
    idx: usize,
    sink: &mut dyn Write,
) -> Result<(u64, u32), Error> {
    let mut archive = Archive::new(open_input(path, compression)?);
    for (index, entry) in archive
        .entries()
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?
        .enumerate()
    {
        let mut entry = entry.map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        if index != idx {
            std::io::copy(&mut entry, &mut std::io::sink())
                .map_err(|err| Error::io(format!("skip {index}"), err))?;
            continue;
        }
        let mut hasher = CrcWriter::new();
        {
            let mut fan = Fanout {
                left: &mut hasher,
                right: sink,
            };
            std::io::copy(&mut entry, &mut fan)
                .map_err(|err| Error::io(format!("entry {idx}"), err))?;
        }
        return Ok((hasher.bytes, hasher.crc()));
    }
    Err(Error::Usage(format!("entry {idx} is out of range")))
}

/// Decode a single-file gzip or zstd stream into `sink`.
pub fn copy_single(
    path: &Path,
    compression: Compression,
    sink: &mut dyn Write,
) -> Result<(u64, u32), Error> {
    let mut input = open_input(path, compression)?;
    let mut hasher = CrcWriter::new();
    {
        let mut fan = Fanout {
            left: &mut hasher,
            right: sink,
        };
        std::io::copy(&mut input, &mut fan)
            .map_err(|err| Error::io(format!("read {}", path.display()), err))?;
    }
    Ok((hasher.bytes, hasher.crc()))
}

struct Fanout<'a> {
    left: &'a mut CrcWriter,
    right: &'a mut dyn Write,
}

impl Write for Fanout<'_> {
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
