//! Operations shared by the command line and the desktop window.

use std::collections::{HashMap, HashSet};
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
    AddReport, CompressOpts, ConflictChoice, ConflictDecider, DeleteReport, ExtractOpts,
    ExtractReport, ListOpts, TestReport, UpdateMode, UpdatePrompt,
};
use crate::progress::{ProgressReader, ProgressSink, ensure_running};

/// Pack `inputs` into `dest`. The format comes from the extension.
///
/// An existing ZIP or 7z keeps members this call does not replace, unless `opts.force` is set.
/// A password on that update is accepted for 7z, which recompresses the members it keeps.
/// A ZIP update copies old members unchanged, so a password there is rejected unless `opts.force` replaces the archive.
pub fn add(
    dest: &Path,
    inputs: &[PathBuf],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<AddReport, Error> {
    add_inner(dest, Some(dest), inputs, opts, progress, None)
}

/// Pack `inputs` into `dest`, asking `prompt` when [`UpdateMode::Ask`] hits an existing member.
pub fn add_prompted(
    dest: &Path,
    inputs: &[PathBuf],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
    prompt: &UpdatePrompt<'_>,
) -> Result<AddReport, Error> {
    add_inner(dest, Some(dest), inputs, opts, progress, Some(prompt))
}

/// UTF-8 comment stored in a ZIP archive. An empty comment is an empty string.
pub fn zip_comment(path: &Path) -> Result<String, Error> {
    let format = detect_file(path)?;
    if !matches!(format, Format::Zip) {
        return Err(Error::Usage(
            "only ZIP archives store a comment".to_string(),
        ));
    }
    let file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let archive = zip::ZipArchive::new(file)?;
    String::from_utf8(archive.comment().to_vec())
        .map_err(|_| Error::Archive("ZIP comment is not UTF-8".to_string()))
}

/// Pack `inputs` into a temporary ZIP and wrap it as a self-extractor at `dest`.
///
/// The walk skips `dest`, so an existing package in the input tree is not stored.
///
/// # Errors
///
/// Returns [`Error::Usage`] when `opts.update` is not [`UpdateMode::Add`].
/// Other failures match [`add`].
pub fn add_sfx(
    dest: &Path,
    stub: &Path,
    inputs: &[PathBuf],
    opts: &CompressOpts,
    payload: crate::sfx::SfxPayload,
    progress: &dyn ProgressSink,
) -> Result<AddReport, Error> {
    if opts.update != UpdateMode::Add {
        return Err(Error::Usage(
            "--sfx is only valid when creating an archive".to_string(),
        ));
    }
    let scratch = scratch_archive(dest, payload);
    let _ = std::fs::remove_file(&scratch);
    let mut opts = opts.clone();
    opts.force = true;
    if payload == crate::sfx::SfxPayload::SevenZ && opts.level > 0 {
        // The stub is built with LZMA2 and AES only.
        opts.sevenz.method = crate::options::SevenZMethod::Lzma2;
    }
    let result = (|| {
        let report = add_inner(&scratch, Some(dest), inputs, &opts, progress, None)?;
        let packed_bytes = crate::sfx::build(dest, stub, &scratch, payload, progress)?;
        Ok(AddReport {
            path: dest.to_path_buf(),
            entries: report.entries,
            bytes: report.bytes,
            packed_bytes,
        })
    })();
    let _ = std::fs::remove_file(&scratch);
    result
}

fn scratch_archive(dest: &Path, payload: crate::sfx::SfxPayload) -> PathBuf {
    // Parallel `szip a --sfx` calls must not share one temp ZIP. The counter
    // only makes the file name unique.
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = dest
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_string());
    std::env::temp_dir().join(format!(
        ".{name}.{}.{id}.szip-sfx.{}",
        std::process::id(),
        payload.extension()
    ))
}

fn add_inner(
    dest: &Path,
    skip: Option<&Path>,
    inputs: &[PathBuf],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
    prompt: Option<&UpdatePrompt<'_>>,
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
    if opts.update == UpdateMode::Ask && prompt.is_none() {
        return Err(Error::Usage(
            "ask before overwrite needs a prompt".to_string(),
        ));
    }
    if let Some(comment) = &opts.comment {
        if matches!(format, Format::Zip) && comment.len() > usize::from(u16::MAX) {
            return Err(Error::Usage(
                "ZIP comment is longer than 65535 bytes".to_string(),
            ));
        }
        if !comment.is_empty() && !matches!(format, Format::Zip | Format::SevenZ) {
            return Err(Error::Usage(
                "archive comments are stored in ZIP and 7z archives".to_string(),
            ));
        }
    }
    if opts.sevenz.encrypt_headers && !matches!(format, Format::SevenZ) {
        return Err(Error::Usage(
            "encrypting file names is a 7z option; ZIP names stay visible".to_string(),
        ));
    }
    if opts.update != UpdateMode::Add
        && dest.exists()
        && !matches!(format, Format::Zip | Format::SevenZ)
    {
        return Err(Error::Usage(
            "only ZIP and 7z archives can be updated in place".to_string(),
        ));
    }
    if opts.password.is_some() && !matches!(format, Format::Zip | Format::SevenZ) {
        return Err(Error::Usage(
            "passwords encrypt ZIP and 7z archives; other formats are not encrypted".to_string(),
        ));
    }
    // A 7z update recompresses kept members, so the password that opened the
    // archive has to travel with the rewrite. A ZIP update copies old members
    // unchanged, and a new password would leave them on the old key.
    if opts.password.is_some() && dest.is_file() && !opts.force && !matches!(format, Format::SevenZ)
    {
        return Err(Error::Usage(
            "pass -y to replace a ZIP when encrypting; an update would leave older members unchanged"
                .to_string(),
        ));
    }
    let mut planned = walk::plan(inputs, opts, skip)?;
    let source_names: Vec<String> = planned.iter().map(|item| item.rel.clone()).collect();
    let mut opts = opts.clone();
    if opts.update != UpdateMode::Add && !opts.force && dest.is_file() {
        let index = member_times(dest, opts.password.as_ref())?;
        planned = select_members(planned, opts.update, &index, prompt)?;
        let extras = opts.update == UpdateMode::Sync
            && index
                .keys()
                .any(|key| source_names.iter().all(|name| &member_key(name) != key));
        if planned.is_empty() && !extras {
            let packed = std::fs::metadata(dest).map(|meta| meta.len()).unwrap_or(0);
            return Ok(AddReport {
                path: dest.to_path_buf(),
                entries: 0,
                bytes: 0,
                packed_bytes: packed,
            });
        }
        if opts.update == UpdateMode::Sync {
            opts.sync_keep = Some(source_names);
        }
    }
    let total = planned.iter().map(|item| item.meta.size).sum();
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
        crate::progress::ensure_running(progress)?;
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

/// Delete ZIP or 7z members whose names match `masks`, rewriting via a temporary file.
///
/// `password` unlocks an encrypted 7z header and re-encrypts the members that stay.
pub fn delete(
    path: &Path,
    masks: &[String],
    password: Option<&crate::crypto::Password>,
    progress: &dyn ProgressSink,
) -> Result<DeleteReport, Error> {
    if masks.is_empty() {
        return Err(Error::NoFiles);
    }
    let format = detect_file(path)?;
    if !matches!(format, Format::Zip | Format::SevenZ) {
        return Err(Error::Usage(
            "delete only rewrites ZIP and 7z archives".to_string(),
        ));
    }
    let (_opened, entries) = list_with(
        path,
        &ListOpts {
            password: password.cloned(),
            ..ListOpts::default()
        },
    )?;
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
    // A header-encrypted 7z cannot be listed without the password. Keep that
    // property; data-only encryption leaves names visible and stays that way.
    let encrypt_headers = password.is_some()
        && matches!(format, Format::SevenZ)
        && matches!(list(path), Err(Error::Encrypted(_)));
    let kept_opts = CompressOpts {
        password: password.cloned(),
        sevenz: crate::options::SevenZOpts {
            encrypt_headers,
            ..crate::options::SevenZOpts::default()
        },
        ..CompressOpts::default()
    };
    let mut writer = archiver::create_from_path(path, &kept_opts)?;
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

/// Rename `from` to `to` inside a ZIP or 7z archive.
///
/// See [`crate::rename`] for the report fields and the name rules.
///
/// # Errors
///
/// [`Error::Usage`] when the container is not ZIP or 7z, or when `to` names a
/// different member that is already stored. [`Error::UnsafePath`] for an empty
/// name, a leading slash, a drive letter, `..`, or a control character.
/// [`Error::NoFiles`] when `from` is not in the archive. [`Error::Interrupted`]
/// when `progress` is cancelled; the original file is left in place.
pub fn rename(
    path: &Path,
    from: &str,
    to: &str,
    password: Option<&crate::crypto::Password>,
    progress: &dyn ProgressSink,
) -> Result<DeleteReport, Error> {
    let format = detect_file(path)?;
    if !matches!(format, Format::Zip | Format::SevenZ) {
        return Err(Error::Usage(
            "rename only rewrites ZIP and 7z archives".to_string(),
        ));
    }
    let from_key = checked_member_name(from)?;
    let to_key = checked_member_name(to)?;
    let (_opened, entries) = list_with(
        path,
        &ListOpts {
            password: password.cloned(),
            ..ListOpts::default()
        },
    )?;
    let map = rename_map(&entries, &from_key, &to_key)?;
    reject_collisions(&entries, &map)?;
    let changed: HashMap<String, String> =
        map.into_iter().filter(|(src, dest)| src != dest).collect();
    let changed_count = changed.len();
    progress.begin(&format!("rename {}", path.display()), 0);
    ensure_running(progress)?;
    let (kept, packed_bytes) = if changed.is_empty() {
        let packed_bytes = std::fs::metadata(path)
            .map_err(|err| Error::io(format!("stat {}", path.display()), err))?
            .len();
        (member_count(&entries)?, packed_bytes)
    } else {
        let encrypt_headers = password.is_some()
            && matches!(format, Format::SevenZ)
            && matches!(list(path), Err(Error::Encrypted(_)));
        let opts = CompressOpts {
            password: password.cloned(),
            sevenz: crate::options::SevenZOpts {
                encrypt_headers,
                ..crate::options::SevenZOpts::default()
            },
            ..CompressOpts::default()
        };
        archiver::rewrite_renamed(path, &format, &opts, &changed, progress)?
    };
    progress.message(&format!("renamed {changed_count} in {}", path.display()));
    Ok(DeleteReport {
        path: path.to_path_buf(),
        removed: 0,
        kept,
        packed_bytes,
    })
}

fn checked_member_name(raw: &str) -> Result<String, Error> {
    let rel = crate::fs::safe_path::safe_rel(raw)?;
    let mut out = String::new();
    for component in rel.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(Error::UnsafePath(raw.to_string()));
        };
        let Some(text) = part.to_str() else {
            return Err(Error::UnsafePath(raw.to_string()));
        };
        if text.is_empty() || text.contains('/') || text.contains('\\') {
            return Err(Error::UnsafePath(raw.to_string()));
        }
        if !out.is_empty() {
            out.push('/');
        }
        out.push_str(text);
    }
    if out.is_empty() {
        return Err(Error::UnsafePath(raw.to_string()));
    }
    Ok(member_key(&out))
}

fn rename_map(
    entries: &[crate::entry::Entry],
    from_key: &str,
    to_key: &str,
) -> Result<HashMap<String, String>, Error> {
    let prefix = format!("{from_key}/");
    let mut matched = Vec::new();
    for entry in entries {
        let key = member_key(&entry.path);
        if key.is_empty() {
            continue;
        }
        if key == from_key || key.starts_with(&prefix) {
            matched.push((key, entry.kind));
        }
    }
    if matched.is_empty() {
        return Err(Error::NoFiles);
    }
    let directory = matched.iter().any(|(key, kind)| {
        key.starts_with(&prefix) || (*kind == EntryKind::Dir && key == from_key)
    });
    let mut map = HashMap::new();
    for (key, _) in matched {
        if !directory && key != from_key {
            continue;
        }
        let dest = if key == from_key {
            to_key.to_string()
        } else if let Some(rest) = key.strip_prefix(&prefix).filter(|rest| !rest.is_empty()) {
            format!("{to_key}/{rest}")
        } else {
            continue;
        };
        let dest = checked_member_name(&dest)?;
        map.insert(key, dest);
    }
    if map.is_empty() {
        return Err(Error::NoFiles);
    }
    Ok(map)
}

fn reject_collisions(
    entries: &[crate::entry::Entry],
    map: &HashMap<String, String>,
) -> Result<(), Error> {
    let existing: HashSet<String> = entries
        .iter()
        .map(|entry| member_key(&entry.path))
        .filter(|key| !key.is_empty())
        .collect();
    let mut destinations = HashSet::new();
    for (src, dest) in map {
        if !destinations.insert(dest.clone()) {
            return Err(Error::Usage(format!("destination `{dest}` already exists")));
        }
        if dest != src && existing.contains(dest) {
            return Err(Error::Usage(format!("destination `{dest}` already exists")));
        }
    }
    Ok(())
}

fn member_count(entries: &[crate::entry::Entry]) -> Result<u64, Error> {
    let mut seen = HashSet::new();
    let mut count = 0u64;
    for entry in entries {
        let key = member_key(&entry.path);
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| Error::Archive("entry count overflow".to_string()))?;
    }
    Ok(count)
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
    if opts.update == UpdateMode::Ask {
        return Err(Error::Usage(
            "ask before overwrite needs a prompt and is not available for stdin".to_string(),
        ));
    }
    if opts.update != UpdateMode::Add && !matches!(format, Format::Zip | Format::SevenZ) {
        return Err(Error::Usage(
            "only ZIP and 7z archives can be updated in place".to_string(),
        ));
    }
    match format {
        Format::Tar(_) => {
            let spooled = spool_stdin(input)?;
            let mut file = File::open(&spooled.path)
                .map_err(|err| Error::io(format!("open {}", spooled.path.display()), err))?;
            stream_one(dest, &rel, &mut file, Some(spooled.len), opts, progress)
        }
        Format::Zip | Format::Gzip | Format::Zstd | Format::SevenZ => {
            stream_one(dest, &rel, input, None, opts, progress)
        }
        other => Err(Error::Unsupported {
            format: other.label(),
            detail: "stdin packing supports .zip, .7z, .tar, .tar.gz, .tar.zst, .gz, and .zst"
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
    limits: &crate::options::ReadLimits,
) -> Result<TestReport, Error> {
    let mut opened = Opened::open_with(path, password.cloned())?;
    opened.test(progress, password, limits)
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

fn member_times(
    path: &Path,
    password: Option<&crate::crypto::Password>,
) -> Result<HashMap<String, Option<SystemTime>>, Error> {
    let (_opened, entries) = list_with(
        path,
        &ListOpts {
            password: password.cloned(),
            ..ListOpts::default()
        },
    )?;
    let mut index = HashMap::with_capacity(entries.len());
    for entry in entries {
        index.insert(member_key(&entry.path), entry.modified);
    }
    Ok(index)
}

enum MemberAction {
    Write,
    Drop,
    Rename(String),
}

fn select_members(
    planned: Vec<Planned>,
    mode: UpdateMode,
    existing: &HashMap<String, Option<SystemTime>>,
    prompt: Option<&UpdatePrompt<'_>>,
) -> Result<Vec<Planned>, Error> {
    let mut taken: HashSet<String> = existing.keys().cloned().collect();
    for item in &planned {
        taken.insert(member_key(&item.rel));
    }
    let mut kept = Vec::with_capacity(planned.len());
    for mut item in planned {
        let key = member_key(&item.rel);
        let present = existing.get(&key);
        let action = match mode {
            UpdateMode::Add => MemberAction::Write,
            UpdateMode::Update => match present {
                None => MemberAction::Write,
                Some(mtime) if newer(item.meta.modified, *mtime) => MemberAction::Write,
                Some(_) => MemberAction::Drop,
            },
            UpdateMode::Freshen => match present {
                Some(mtime) if newer(item.meta.modified, *mtime) => MemberAction::Write,
                _ => MemberAction::Drop,
            },
            UpdateMode::Skip => {
                if present.is_some() {
                    MemberAction::Drop
                } else {
                    MemberAction::Write
                }
            }
            UpdateMode::Sync => match present {
                None => MemberAction::Write,
                Some(mtime) if newer(item.meta.modified, *mtime) => MemberAction::Write,
                Some(_) => MemberAction::Drop,
            },
            UpdateMode::Ask => {
                if present.is_none() {
                    MemberAction::Write
                } else {
                    let prompt = prompt.ok_or_else(|| {
                        Error::Usage("ask before overwrite needs a prompt".to_string())
                    })?;
                    match prompt(&item.rel)? {
                        ConflictChoice::Overwrite => MemberAction::Write,
                        ConflictChoice::Skip => MemberAction::Drop,
                        ConflictChoice::Rename => {
                            let rel = fresh_name(&item.rel, &taken);
                            taken.insert(member_key(&rel));
                            MemberAction::Rename(rel)
                        }
                    }
                }
            }
        };
        match action {
            MemberAction::Drop => {}
            MemberAction::Write => kept.push(item),
            MemberAction::Rename(rel) => {
                item.rel = rel;
                kept.push(item);
            }
        }
    }
    Ok(kept)
}

/// `notes.txt` becomes `notes (2).txt`, then `(3)`, until the name is free.
fn fresh_name(rel: &str, taken: &HashSet<String>) -> String {
    let (dir, file) = match rel.rsplit_once('/') {
        Some((dir, file)) => (Some(dir), file),
        None => (None, rel),
    };
    let (stem, ext) = match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => (stem, Some(ext)),
        _ => (file, None),
    };
    for index in 2..10_000 {
        let name = match ext {
            Some(ext) => format!("{stem} ({index}).{ext}"),
            None => format!("{stem} ({index})"),
        };
        let full = match dir {
            Some(dir) => format!("{dir}/{name}"),
            None => name,
        };
        if !taken.contains(&member_key(&full)) {
            return full;
        }
    }
    format!("{rel}.new")
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
    if opts.update == UpdateMode::Freshen && !dest.is_file() {
        return Err(Error::io(
            format!("open {}", dest.display()),
            std::io::Error::new(std::io::ErrorKind::NotFound, "archive does not exist"),
        ));
    }
    let mut opts = opts.clone();
    if dest.is_file() && opts.update != UpdateMode::Add && !opts.force {
        let index = member_times(dest, opts.password.as_ref())?;
        let key = member_key(&rel_text);
        if opts.update == UpdateMode::Freshen && !index.contains_key(&key) {
            return Err(Error::NoFiles);
        }
        if opts.update == UpdateMode::Skip && index.contains_key(&key) {
            let packed = std::fs::metadata(dest).map(|meta| meta.len()).unwrap_or(0);
            return Ok(AddReport {
                path: dest.to_path_buf(),
                entries: 0,
                bytes: 0,
                packed_bytes: packed,
            });
        }
        if opts.update == UpdateMode::Sync {
            opts.sync_keep = Some(vec![rel_text.clone()]);
        }
    }
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
