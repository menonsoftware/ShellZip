//! Operations shared by the command line and the desktop window.

use std::fs::File;
use std::path::{Path, PathBuf};

use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

use crate::archiver;
use crate::entry::EntryKind;
use crate::error::Error;
use crate::extractor::{ArchiveReader, Opened};
use crate::fs::walk;
use crate::options::{
    AddReport, CompressOpts, ConflictDecider, ExtractOpts, ExtractReport, TestReport,
};
use crate::progress::{ProgressReader, ProgressSink};

/// Pack `inputs` into a new archive at `dest`. The format comes from the extension.
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
    let planned = walk::plan(inputs, opts, Some(dest))?;
    let total = planned.iter().map(|item| item.meta.size).sum();
    progress.begin(&format!("add {}", dest.display()), total);
    let mut writer = archiver::create_from_path(dest, opts)?;
    let mut entries = 0u64;
    let mut bytes = 0u64;
    for item in &planned {
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
                bytes += item.meta.size;
            }
            EntryKind::Other => {
                progress.end_entry();
                continue;
            }
        }
        entries += 1;
        progress.end_entry();
    }
    let packed_bytes = writer.finish()?;
    tracing::debug!(entries, bytes, packed_bytes, "archive written");
    progress.message(&format!("wrote {} ({} entries)", dest.display(), entries));
    Ok(AddReport {
        path: dest.to_path_buf(),
        entries,
        bytes,
        packed_bytes,
    })
}

/// Sniff, list, and return the format label's source archive.
pub fn list(path: &Path) -> Result<(Opened, Vec<crate::entry::Entry>), Error> {
    let mut opened = Opened::open(path)?;
    let entries = opened.entries()?;
    Ok((opened, entries))
}

/// Extract `path` using `opts`.
pub fn extract(
    path: &Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport, Error> {
    let opened = Opened::open(path)?;
    let filter = name_filter(opts)?;
    let report = opened.extract_to(opts, decide, &filter, progress)?;
    if report.extracted == 0 && report.skipped == 0 && filter_is_restrictive(opts) {
        return Err(Error::NoFiles);
    }
    Ok(report)
}

/// Test every member of `path`.
pub fn test(path: &Path, progress: &dyn ProgressSink) -> Result<TestReport, Error> {
    let mut opened = Opened::open(path)?;
    opened.test(progress, None)
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
