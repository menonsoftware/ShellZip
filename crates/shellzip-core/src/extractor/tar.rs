//! Tar, tar.gz, tar.zst, and single-file gzip/zstd streams.

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use flate2::read::GzDecoder;
use tar::{Archive, EntryType};

use super::policy::{
    Budget, Destination, copy_limited, destination, mark_extracted, restore_entry_time,
    symlink_destination, write_atomic,
};
use crate::detect::{Compression, prefix_is_tar};
use crate::entry::{Entry, EntryKind};
use crate::error::Error;
use crate::hash::{Blake2spWriter, CrcWriter};
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

/// List a tar stream. `hash` computes CRC32 and BLAKE2sp while reading each file.
pub fn list_tar(path: &Path, compression: Compression, hash: bool) -> Result<Vec<Entry>, Error> {
    let mut archive = Archive::new(open_input(path, compression)?);
    let mut entries = Vec::new();
    for entry in archive
        .entries()
        .map_err(|err| Error::io(format!("read {}", path.display()), err))?
    {
        let mut entry = entry.map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        let mut listed = tar_entry(&mut entry)?;
        if hash && listed.kind == EntryKind::File {
            let mut crc = CrcWriter::new();
            let mut blake = Blake2spWriter::new();
            {
                let mut fan = Fanout {
                    left: &mut crc,
                    right: &mut blake,
                };
                std::io::copy(&mut entry, &mut fan)
                    .map_err(|err| Error::io(listed.path.clone(), err))?;
            }
            listed.crc32 = Some(crc.crc());
            listed.blake2sp = Some(blake.finish());
        }
        entries.push(listed);
    }
    Ok(entries)
}

/// Test a tar stream by reading every payload and checking declared sizes.
pub fn test_tar(
    path: &Path,
    compression: Compression,
    progress: &dyn ProgressSink,
    limits: &crate::options::ReadLimits,
) -> Result<TestReport, Error> {
    progress.begin(&format!("test {}", path.display()), 0);
    let mut archive = Archive::new(open_input(path, compression)?);
    let mut report = TestReport::default();
    let mut budget = Budget::for_archive(path, limits.max_bytes, limits.max_ratio);
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
        budget.reserve(declared, None)?;
        progress.begin_entry(&name, declared);
        let mut hasher = CrcWriter::new();
        let mut reader = ProgressReader::new(&mut entry, progress);
        copy_limited(&mut reader, &mut hasher, &mut budget, &name)?;
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
    let mut budget = Budget::for_archive(path, opts.max_bytes, opts.max_ratio);
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
        budget.reserve(declared, None)?;
        match kind {
            EntryType::Directory => {
                if !opts.flat {
                    let modified = entry
                        .header()
                        .mtime()
                        .ok()
                        .map(|secs| UNIX_EPOCH + std::time::Duration::from_secs(secs));
                    match destination(&name, opts, false, true, modified, decide)? {
                        Destination::Skip => report.skipped += 1,
                        Destination::Write(path) => {
                            let _ = restore_entry_time(&path, modified, opts);
                            report.extracted += 1;
                        }
                    }
                } else {
                    report.extracted += 1;
                }
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
                match crate::fs::metadata::create_symlink(&link.target, &link.path) {
                    Ok(()) => report.extracted += 1,
                    Err(err) => report.warnings.push(err.to_string()),
                }
            }
            EntryType::Regular | EntryType::Continuous => {
                if let Some(dest) = extract_regular(
                    &mut entry,
                    &name,
                    opts,
                    decide,
                    &mut budget,
                    &mut report,
                    progress,
                )? && let Err(err) = mark_extracted(path, &dest, opts)
                {
                    report.warnings.push(err.to_string());
                }
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

/// List one compressed stream. `hash` decodes it to fill CRC32 and BLAKE2sp.
pub fn list_single(path: &Path, compression: Compression, hash: bool) -> Result<Vec<Entry>, Error> {
    let name = single_name(path, compression)?;
    let (size, crc32, blake2sp) = if hash {
        let mut input = open_input(path, compression)?;
        let mut crc = CrcWriter::new();
        let mut blake = Blake2spWriter::new();
        {
            let mut fan = Fanout {
                left: &mut crc,
                right: &mut blake,
            };
            std::io::copy(&mut input, &mut fan).map_err(|err| Error::io(name.clone(), err))?;
        }
        (Some(crc.bytes), Some(crc.crc()), Some(blake.finish()))
    } else {
        let size = match compression {
            Compression::Gzip => gzip_isize(path),
            Compression::Zstd => zstd_content_size(path),
            Compression::None | Compression::Xz | Compression::Bzip2 => None,
        };
        let crc32 = if compression == Compression::Gzip {
            gzip_crc(path)
        } else {
            None
        };
        (size, crc32, None)
    };
    Ok(vec![Entry {
        path: name,
        kind: EntryKind::File,
        size,
        packed_size: std::fs::metadata(path).ok().map(|meta| meta.len()),
        modified: None,
        mode: None,
        crc32,
        method: Some(method_name(compression).to_string()),
        host_os: None,
        blake2sp,
        link_target: None,
        encrypted: false,
        solid: false,
        dictionary: None,
    }])
}

/// Test one gzip or zstd member by decoding it fully.
pub fn test_single(
    path: &Path,
    compression: Compression,
    progress: &dyn ProgressSink,
    limits: &crate::options::ReadLimits,
) -> Result<TestReport, Error> {
    let name = single_name(path, compression)?;
    progress.begin(&format!("test {}", path.display()), 0);
    progress.begin_entry(&name, 0);
    let mut input = open_input(path, compression)?;
    let mut hasher = CrcWriter::new();
    let mut reader = ProgressReader::new(&mut input, progress);
    let mut budget = Budget::for_archive(path, limits.max_bytes, limits.max_ratio);
    copy_limited(&mut reader, &mut hasher, &mut budget, &name)?;
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
    let mut budget = Budget::for_archive(path, opts.max_bytes, opts.max_ratio);
    let mut input = open_input(path, compression)?;
    match destination(&name, opts, opts.flat, false, None, decide)? {
        Destination::Skip => report.skipped += 1,
        Destination::Write(dest) => {
            write_atomic(&dest, opts.keep_broken, |out| {
                let mut reader = ProgressReader::new(&mut input, progress);
                copy_limited(&mut reader, out, &mut budget, &name)?;
                Ok(())
            })?;
            if let Err(err) = mark_extracted(path, &dest, opts) {
                report.warnings.push(err.to_string());
            }
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
) -> Result<Option<std::path::PathBuf>, Error> {
    let declared = entry.size();
    let modified = entry
        .header()
        .mtime()
        .ok()
        .map(|secs| UNIX_EPOCH + std::time::Duration::from_secs(secs));
    match destination(name, opts, opts.flat, false, modified, decide)? {
        Destination::Skip => {
            std::io::copy(entry, &mut std::io::sink())
                .map_err(|err| Error::io(name.to_string(), err))?;
            report.skipped += 1;
            Ok(None)
        }
        Destination::Write(dest) => {
            let mode = entry.header().mode().ok();
            write_atomic(&dest, opts.keep_broken, |out| {
                let mut reader = ProgressReader::new(entry, progress);
                let copied = copy_limited(&mut reader, out, budget, name)?;
                if copied != declared {
                    return Err(Error::BadChecksum {
                        entry: format!("{name} (read {copied} bytes, header says {declared})"),
                    });
                }
                Ok(())
            })?;
            if let Err(err) = restore_entry_time(&dest, modified, opts) {
                report.warnings.push(err.to_string());
            }
            if let Some(mode) = mode
                && let Err(err) = crate::fs::metadata::restore_mode(&dest, mode)
            {
                report.warnings.push(err.to_string());
            }
            report.extracted += 1;
            Ok(Some(dest))
        }
    }
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
        host_os: Some("Unix".to_string()),
        blake2sp: None,
        link_target,
        encrypted: false,
        solid: false,
        dictionary: None,
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
        Compression::Xz => Box::new(xz2::read::XzDecoder::new(reader)),
        Compression::Bzip2 => Box::new(bzip2::read::BzDecoder::new(reader)),
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

fn method_name(compression: Compression) -> &'static str {
    match compression {
        Compression::Gzip => "gzip",
        Compression::Zstd => "zstd",
        Compression::Xz => "xz",
        Compression::Bzip2 => "bzip2",
        Compression::None => "store",
    }
}

fn gzip_crc(path: &Path) -> Option<u32> {
    use std::io::{Seek, SeekFrom};
    let mut file = File::open(path).ok()?;
    if file.metadata().ok()?.len() < 8 {
        return None;
    }
    file.seek(SeekFrom::End(-8)).ok()?;
    let mut buf = [0u8; 4];
    std::io::Read::read_exact(&mut file, &mut buf).ok()?;
    Some(u32::from_le_bytes(buf))
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
            let mut skip = Budget::for_archive(
                path,
                crate::options::DEFAULT_MAX_BYTES,
                crate::options::DEFAULT_MAX_RATIO,
            );
            copy_limited(
                &mut entry,
                &mut std::io::sink(),
                &mut skip,
                &format!("skip {index}"),
            )?;
            continue;
        }
        let declared = entry.size();
        let mut budget = Budget::for_archive(
            path,
            crate::options::DEFAULT_MAX_BYTES,
            crate::options::DEFAULT_MAX_RATIO,
        );
        budget.reserve(declared, None)?;
        let mut hasher = CrcWriter::new();
        {
            let mut fan = Fanout {
                left: &mut hasher,
                right: sink,
            };
            copy_limited(&mut entry, &mut fan, &mut budget, &format!("entry {idx}"))?;
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
    let mut budget = Budget::for_archive(
        path,
        crate::options::DEFAULT_MAX_BYTES,
        crate::options::DEFAULT_MAX_RATIO,
    );
    {
        let mut fan = Fanout {
            left: &mut hasher,
            right: sink,
        };
        copy_limited(
            &mut input,
            &mut fan,
            &mut budget,
            &format!("read {}", path.display()),
        )?;
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn extracts_xz_bzip2_and_tar_xz() {
        let tmp = tempfile::tempdir().unwrap();
        let xz_path = tmp.path().join("hello.xz");
        {
            let file = File::create(&xz_path).unwrap();
            let mut enc = xz2::write::XzEncoder::new(file, 6);
            enc.write_all(b"hello xz").unwrap();
            enc.finish().unwrap();
        }
        let bz_path = tmp.path().join("hello.bz2");
        {
            let file = File::create(&bz_path).unwrap();
            let mut enc = bzip2::write::BzEncoder::new(file, bzip2::Compression::fast());
            enc.write_all(b"hello bz2").unwrap();
            enc.finish().unwrap();
        }
        let tar_xz = tmp.path().join("bundle.tar.xz");
        {
            let mut plain = Vec::new();
            {
                let mut builder = tar::Builder::new(&mut plain);
                let mut header = tar::Header::new_gnu();
                header.set_size(5);
                header.set_mode(0o644);
                header.set_cksum();
                builder
                    .append_data(&mut header, "note.txt", &b"hello"[..])
                    .unwrap();
                builder.finish().unwrap();
            }
            let file = File::create(&tar_xz).unwrap();
            let mut enc = xz2::write::XzEncoder::new(file, 6);
            enc.write_all(&plain).unwrap();
            enc.finish().unwrap();
        }

        let xz_out = tmp.path().join("xz");
        let bz_out = tmp.path().join("bz");
        let tar_out = tmp.path().join("tar");
        for (archive, dir, rel, bytes) in [
            (&xz_path, &xz_out, "hello", &b"hello xz"[..]),
            (&bz_path, &bz_out, "hello", &b"hello bz2"[..]),
            (&tar_xz, &tar_out, "note.txt", &b"hello"[..]),
        ] {
            std::fs::create_dir(dir).unwrap();
            let mut opts = crate::ExtractOpts::new(dir);
            opts.overwrite = crate::Overwrite::Always;
            crate::extract(archive, &opts, None, &crate::NopProgress).unwrap();
            assert_eq!(std::fs::read(dir.join(rel)).unwrap(), bytes, "{rel}");
            crate::test(archive, None, &crate::NopProgress).unwrap();
        }
    }
}
