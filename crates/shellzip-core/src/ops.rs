//! Operations shared by the command line and the desktop window.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::archiver;
use crate::detect::{Format, detect_file, format_from_extension};
use crate::entry::{EntryKind, EntryMeta, member_key, to_archive_path};
use crate::error::Error;
use crate::extractor::{ArchiveReader, Opened};
use crate::fs::walk::{self, Planned};
use crate::options::{
    AddReport, CompressOpts, ConflictDecider, DeleteReport, ExtractOpts, ExtractReport, ListOpts,
    TestReport, UpdateMode,
};
use crate::progress::{ProgressReader, ProgressSink};

/// Pack `inputs` into `dest`. The format comes from the extension.
///
/// An existing ZIP keeps members this call does not replace, unless `opts.force` is set.
pub fn add(
    dest: &Path,
    inputs: &[PathBuf],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<AddReport, Error> {
    if opts.level > 5 {
        return Err(Error::Usage(
            "compression level must be 0 through 5".to_string(),
        ));
    }
    let format = format_from_extension(dest)?;
    if opts.update == UpdateMode::Freshen && !dest.exists() {
        return Err(Error::io(
            format!("open {}", dest.display()),
            std::io::Error::new(std::io::ErrorKind::NotFound, "archive does not exist"),
        ));
    }
    if opts.update != UpdateMode::Add && dest.exists() && !matches!(format, Format::Zip) {
        return Err(Error::Usage(
            "update and freshen only rewrite ZIP archives".to_string(),
        ));
    }
    if opts.password.is_some() && !matches!(format, Format::Zip) {
        return Err(Error::Usage(
            "passwords encrypt ZIP file contents only; names stay visible, and other formats are not encrypted"
                .to_string(),
        ));
    }
    if opts.password.is_some() && dest.is_file() && !opts.force {
        return Err(Error::Usage(
            "pass -y to replace a ZIP when encrypting; an update would leave older members unchanged"
                .to_string(),
        ));
    }
    let mut planned = walk::plan(inputs, opts, Some(dest))?;
    if opts.update != UpdateMode::Add && !opts.force && dest.is_file() {
        let index = zip_mtimes(dest)?;
        planned.retain(|item| keep_member(item, opts.update, &index));
        if planned.is_empty() {
            let packed = std::fs::metadata(dest).map(|meta| meta.len()).unwrap_or(0);
            return Ok(AddReport {
                path: dest.to_path_buf(),
                entries: 0,
                bytes: 0,
                packed_bytes: packed,
            });
        }
    }
    let total = planned.iter().map(|item| item.meta.size).sum();
    let mut opts = opts.clone();
    opts.input_bytes = Some(total);
    progress.begin(&format!("add {}", dest.display()), total);
    let packed_bytes = if matches!(format, Format::Zip)
        && opts.password.is_none()
        && opts.thread_count() > 1
        && planned.iter().any(|item| item.kind == EntryKind::File)
    {
        archiver::add_zip(dest, &planned, &opts, progress)?
    } else {
        write_sequential(dest, &planned, &opts, progress)?
    };
    let entries = planned
        .iter()
        .filter(|item| item.kind != EntryKind::Other)
        .count() as u64;
    let bytes = planned
        .iter()
        .filter(|item| item.kind == EntryKind::File)
        .map(|item| item.meta.size)
        .sum();
    tracing::debug!(entries, bytes, packed_bytes, "archive written");
    progress.message(&format!("wrote {} ({} entries)", dest.display(), entries));
    Ok(AddReport {
        path: dest.to_path_buf(),
        entries,
        bytes,
        packed_bytes,
    })
}

fn write_sequential(
    dest: &Path,
    planned: &[Planned],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<u64, Error> {
    let mut writer = archiver::create_from_path(dest, opts)?;
    let replacing: Vec<String> = planned.iter().map(|item| item.rel.clone()).collect();
    writer.retain_existing(&replacing)?;
    for item in planned {
        progress.begin_entry(&item.rel, item.meta.size);
        let rel = Path::new(&item.rel);
        match item.kind {
            EntryKind::Dir => writer.add_dir(rel, &item.meta)?,
            EntryKind::Symlink => {
                let target = item.link_target.as_deref().unwrap_or("");
                writer.add_symlink(rel, Path::new(target), &item.meta)?;
            }
            EntryKind::File => {
                let source = item
                    .source
                    .as_ref()
                    .ok_or_else(|| Error::Archive(format!("missing source for `{}`", item.rel)))?;
                let file = File::open(source)
                    .map_err(|err| Error::io(format!("open {}", source.display()), err))?;
                let mut reader = ProgressReader::new(file, progress);
                writer.add_file(rel, &mut reader, &item.meta)?;
            }
            EntryKind::Other => {
                progress.end_entry();
                continue;
            }
        }
        progress.end_entry();
    }
    writer.finish()
}

/// Sniff, list, and return the format label's source archive.
pub fn list(path: &Path) -> Result<(Opened, Vec<crate::entry::Entry>), Error> {
    list_with(path, &ListOpts::default())
}

/// List `path`. [`ListOpts::blake2sp`] reads every file payload.
pub fn list_with(
    path: &Path,
    opts: &ListOpts,
) -> Result<(Opened, Vec<crate::entry::Entry>), Error> {
    let mut opened = Opened::open_with(path, opts.password.clone())?;
    let entries = opened.read_entries(opts.blake2sp)?;
    Ok((opened, entries))
}

/// Write one member to `sink`. `name` is required when the archive has several files.
pub fn write_member(
    path: &Path,
    name: Option<&str>,
    sink: &mut dyn Write,
    password: Option<&crate::crypto::Password>,
) -> Result<u64, Error> {
    let mut opened = Opened::open_with(path, password.cloned())?;
    let entries = opened.entries()?;
    let index = match name {
        Some(name) => find_member(&entries, name)?,
        None => only_file(&entries)?,
    };
    let verify = opened.extract_entry(index, sink, password)?;
    Ok(verify.bytes)
}

/// Delete ZIP members whose names match `masks`, rewriting via a temporary file.
pub fn delete(
    path: &Path,
    masks: &[String],
    progress: &dyn ProgressSink,
) -> Result<DeleteReport, Error> {
    if masks.is_empty() {
        return Err(Error::NoFiles);
    }
    let format = detect_file(path)?;
    if !matches!(format, Format::Zip) {
        return Err(Error::Usage(
            "delete only rewrites ZIP archives".to_string(),
        ));
    }
    let (_opened, entries) = list(path)?;
    let globs = build_globs(masks)?;
    let doomed: Vec<String> = entries
        .iter()
        .filter(|entry| mask_hit(&globs, &entry.path))
        .map(|entry| member_key(&entry.path))
        .collect();
    if doomed.is_empty() {
        return Err(Error::NoFiles);
    }
    let kept = u64::try_from(entries.len()).unwrap_or(u64::MAX) - doomed.len() as u64;
    progress.begin(&format!("delete {}", path.display()), 0);
    let mut writer = archiver::create_from_path(path, &CompressOpts::default())?;
    writer.retain_existing(&doomed)?;
    let packed_bytes = writer.finish()?;
    progress.message(&format!("removed {} from {}", doomed.len(), path.display()));
    Ok(DeleteReport {
        path: path.to_path_buf(),
        removed: doomed.len() as u64,
        kept,
        packed_bytes,
    })
}

/// Pack one member read from `input`. `.gz` and `.zst` are streamed.
///
/// Tar copies `input` to a temporary file first because the header stores the
/// size. ZIP is written to a seekable temporary file for the same reason the
/// local headers store sizes; a pipe is not a ZIP destination.
pub fn add_stdin(
    dest: &Path,
    name: &str,
    input: &mut dyn Read,
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<AddReport, Error> {
    if opts.level > 5 {
        return Err(Error::Usage(
            "compression level must be 0 through 5".to_string(),
        ));
    }
    let rel = crate::fs::safe_path::safe_rel(name)?;
    let format = format_from_extension(dest)?;
    if opts.update != UpdateMode::Add && !matches!(format, Format::Zip) {
        return Err(Error::Usage(
            "update and freshen only rewrite ZIP archives".to_string(),
        ));
    }
    match format {
        Format::Tar(_) => {
            let spooled = spool_stdin(input)?;
            let mut file = File::open(&spooled.path)
                .map_err(|err| Error::io(format!("open {}", spooled.path.display()), err))?;
            stream_one(dest, &rel, &mut file, Some(spooled.len), opts, progress)
        }
        Format::Zip | Format::Gzip | Format::Zstd => {
            stream_one(dest, &rel, input, None, opts, progress)
        }
        other => Err(Error::Unsupported {
            format: other.label(),
            detail: "stdin packing supports .zip, .tar, .tar.gz, .tar.zst, .gz, and .zst"
                .to_string(),
        }),
    }
}

/// Extract `path` using `opts`.
pub fn extract(
    path: &Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    #[cfg(not(windows))]
    if opts.mark_of_the_web {
        return Err(Error::Usage(
            "mark-of-the-web (-om) is only written on Windows".to_string(),
        ));
    }
    let opened = Opened::open_with(path, opts.password.clone())?;
    let filter = name_filter(opts)?;
    let report = opened.extract_to(opts, decide, &filter, progress)?;
    if report.extracted == 0 && report.skipped == 0 && filter_is_restrictive(opts) {
        return Err(Error::NoFiles);
    }
    Ok(report)
}

/// Test every member of `path`.
pub fn test(
    path: &Path,
    password: Option<&crate::crypto::Password>,
    progress: &dyn ProgressSink,
) -> Result<TestReport, Error> {
    let mut opened = Opened::open_with(path, password.cloned())?;
    opened.test(progress, password)
}

/// Open an archive for indexed reads.
pub fn open(path: &Path) -> Result<Box<dyn ArchiveReader>, Error> {
    Ok(Box::new(Opened::open(path)?))
}

fn filter_is_restrictive(opts: &ExtractOpts) -> bool {
    !opts.includes.is_empty() || opts.exact_paths.is_some()
}

fn name_filter(opts: &ExtractOpts) -> Result<impl Fn(&str) -> bool, Error> {
    let globs = if opts.includes.is_empty() {
        None
    } else {
        Some(build_globs(&opts.includes)?)
    };
    let exact = opts.exact_paths.clone().map(|paths| {
        paths
            .into_iter()
            .map(|path| path.trim_end_matches(['/', '\\']).replace('\\', "/"))
            .collect::<std::collections::HashSet<_>>()
    });
    Ok(move |name: &str| {
        let trimmed = name.trim_end_matches(['/', '\\']).replace('\\', "/");
        if let Some(exact) = &exact
            && !exact.contains(&trimmed)
        {
            return false;
        }
        match &globs {
            None => true,
            Some(globs) => {
                let base = trimmed.rsplit('/').next().unwrap_or(trimmed.as_str());
                globs.is_match(&trimmed) || globs.is_match(base)
            }
        }
    })
}

fn build_globs(patterns: &[String]) -> Result<GlobSet, Error> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob = GlobBuilder::new(pattern)
            .case_insensitive(cfg!(windows))
            .literal_separator(true)
            .build()
            .map_err(|err| Error::Usage(format!("invalid mask `{pattern}`: {err}")))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|err| Error::Usage(format!("invalid mask: {err}")))
}

fn zip_mtimes(path: &Path) -> Result<HashMap<String, Option<SystemTime>>, Error> {
    let (_opened, entries) = list(path)?;
    let mut index = HashMap::with_capacity(entries.len());
    for entry in entries {
        index.insert(member_key(&entry.path), entry.modified);
    }
    Ok(index)
}

fn keep_member(
    item: &Planned,
    mode: UpdateMode,
    existing: &HashMap<String, Option<SystemTime>>,
) -> bool {
    let key = member_key(&item.rel);
    match (mode, existing.get(&key)) {
        (UpdateMode::Add, _) => true,
        (UpdateMode::Update, None) => true,
        (UpdateMode::Update, Some(mtime)) => newer(item.meta.modified, *mtime),
        (UpdateMode::Freshen, Some(mtime)) => newer(item.meta.modified, *mtime),
        (UpdateMode::Freshen, None) => false,
    }
}

/// Source is newer than the archived copy, past the 2-second ZIP timestamp bucket.
fn newer(source: Option<SystemTime>, archived: Option<SystemTime>) -> bool {
    match (source, archived) {
        (Some(source), Some(archived)) => {
            let source = source
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let archived = archived
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            source > archived.saturating_add(1)
        }
        (Some(_), None) => true,
        (None, _) => false,
    }
}

fn mask_hit(globs: &GlobSet, name: &str) -> bool {
    let trimmed = member_key(name);
    let base = trimmed.rsplit('/').next().unwrap_or(trimmed.as_str());
    globs.is_match(&trimmed) || globs.is_match(base)
}

fn find_member(entries: &[crate::entry::Entry], name: &str) -> Result<usize, Error> {
    let key = member_key(name);
    let mut found = None;
    for (index, entry) in entries.iter().enumerate() {
        if member_key(&entry.path) == key {
            if found.is_some() {
                return Err(Error::Usage(format!(
                    "`{name}` matches more than one member"
                )));
            }
            found = Some(index);
        }
    }
    found.ok_or(Error::NoFiles)
}

fn only_file(entries: &[crate::entry::Entry]) -> Result<usize, Error> {
    let mut found = None;
    for (index, entry) in entries.iter().enumerate() {
        if entry.kind != EntryKind::File {
            continue;
        }
        if found.is_some() {
            return Err(Error::Usage(
                "name the member to print; the archive has more than one file".to_string(),
            ));
        }
        found = Some(index);
    }
    found.ok_or(Error::NoFiles)
}

struct Spooled {
    path: PathBuf,
    len: u64,
}

impl Drop for Spooled {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn spool_stdin(input: &mut dyn Read) -> Result<Spooled, Error> {
    let path = std::env::temp_dir().join(format!(
        "szip-stdin-{}-{}.bin",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    let mut file =
        File::create(&path).map_err(|err| Error::io(format!("create {}", path.display()), err))?;
    let mut buf = Vec::new();
    let len = match crate::fs::copy_chunked(input, &mut file, &mut buf) {
        Ok(len) => len,
        Err(err) => {
            drop(file);
            let _ = std::fs::remove_file(&path);
            return Err(Error::io("read stdin", err));
        }
    };
    file.flush()
        .map_err(|err| Error::io(format!("flush {}", path.display()), err))?;
    Ok(Spooled { path, len })
}

fn stream_one(
    dest: &Path,
    rel: &Path,
    input: &mut dyn Read,
    known: Option<u64>,
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<AddReport, Error> {
    let rel_text = to_archive_path(rel);
    if opts.update == UpdateMode::Freshen {
        if !dest.is_file() {
            return Err(Error::io(
                format!("open {}", dest.display()),
                std::io::Error::new(std::io::ErrorKind::NotFound, "archive does not exist"),
            ));
        }
        let index = zip_mtimes(dest)?;
        if !index.contains_key(&member_key(&rel_text)) {
            return Err(Error::NoFiles);
        }
    }
    let mut opts = opts.clone();
    opts.input_bytes = known.or(opts.input_bytes);
    let mut writer = archiver::create_from_path(dest, &opts)?;
    writer.retain_existing(std::slice::from_ref(&rel_text))?;
    let meta = EntryMeta {
        size: known.unwrap_or(0),
        modified: Some(SystemTime::now()),
        mode: 0o644,
        size_known: known.is_some(),
    };
    progress.begin(&format!("add {}", dest.display()), known.unwrap_or(0));
    progress.begin_entry(&rel_text, meta.size);
    let mut reader = ProgressReader::new(input, progress);
    let mut counting = Counting {
        inner: &mut reader,
        n: 0,
    };
    writer.add_file(rel, &mut counting, &meta)?;
    progress.end_entry();
    let packed_bytes = writer.finish()?;
    let bytes = known.unwrap_or(counting.n);
    Ok(AddReport {
        path: dest.to_path_buf(),
        entries: 1,
        bytes,
        packed_bytes,
    })
}

struct Counting<'a, R> {
    inner: &'a mut R,
    n: u64,
}

impl<R: Read> Read for Counting<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.n += n as u64;
        Ok(n)
    }
}
