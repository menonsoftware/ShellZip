//! Overwrite policy and atomic file replacement.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::fs::safe_path::{
    ensure_dir, extend_long_path, safe_join, safe_rel, symlink_stays_inside,
};
use crate::options::{ConflictChoice, ConflictDecider, ExtractOpts, Overwrite};

/// Stop an extract that would expand past the configured cap.
#[derive(Debug)]
pub struct Budget {
    written: u64,
    max: u64,
}

impl Budget {
    /// Budget that allows `max` uncompressed bytes.
    pub fn new(max: u64) -> Self {
        Self { written: 0, max }
    }

    /// Reject a member whose declared size cannot fit.
    pub fn reserve(&self, declared: u64) -> Result<(), Error> {
        if declared > self.max.saturating_sub(self.written) {
            return Err(Error::Usage(format!(
                "refusing to extract {declared} bytes; {remaining} bytes remain of the {max} byte cap",
                remaining = self.max.saturating_sub(self.written),
                max = self.max
            )));
        }
        Ok(())
    }

    /// Count bytes actually written.
    pub fn charge(&mut self, bytes: u64) -> Result<(), Error> {
        let next = self.written.saturating_add(bytes);
        if next > self.max {
            return Err(Error::Usage(format!(
                "refusing to extract more than {} bytes",
                self.max
            )));
        }
        self.written = next;
        Ok(())
    }
}

/// Where a file should be written, or that it should be left alone.
pub enum Destination {
    /// Do not write this member.
    Skip,
    /// Write to this path.
    Write(PathBuf),
}

/// Resolve `raw` inside `opts.output`, applying the overwrite policy.
pub fn destination(
    raw: &str,
    opts: &ExtractOpts,
    flat: bool,
    is_dir: bool,
    decide: Option<&ConflictDecider<'_>>,
) -> Result<Destination, Error> {
    let rel = if flat && !is_dir {
        let name = raw
            .trim_end_matches(['/', '\\'])
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(raw);
        safe_rel(name)?
    } else {
        safe_rel(raw)?
    };
    let path = safe_join(&opts.output, &rel)?;
    if is_dir {
        ensure_dir(&path)?;
        return Ok(Destination::Write(path));
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    if !exists(&path) {
        return Ok(Destination::Write(path));
    }
    match opts.overwrite {
        Overwrite::Always => Ok(Destination::Write(path)),
        Overwrite::Never => Ok(Destination::Skip),
        Overwrite::Rename => Ok(Destination::Write(renamed(&path))),
        Overwrite::Ask => match decide {
            Some(decide) => match decide(&path)? {
                ConflictChoice::Overwrite => Ok(Destination::Write(path)),
                ConflictChoice::Skip => Ok(Destination::Skip),
                ConflictChoice::Rename => Ok(Destination::Write(renamed(&path))),
            },
            None => Ok(Destination::Skip),
        },
    }
}

/// Validate a symlink target and return the link path when it is safe.
pub fn symlink_destination(
    raw_name: &str,
    target: &str,
    opts: &ExtractOpts,
) -> Result<PathBuf, Error> {
    let rel = safe_rel(raw_name)?;
    let path = safe_join(&opts.output, &rel)?;
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    if opts.allow_unsafe_links {
        if target.is_empty() || target.chars().any(char::is_control) {
            return Err(Error::UnsafePath(target.to_string()));
        }
    } else {
        symlink_stays_inside(&opts.output, &path, target)?;
    }
    Ok(path)
}

/// Copy the archive zone onto a file that was just written, when `-om` is set.
pub fn mark_extracted(archive: &Path, dest: &Path, opts: &ExtractOpts) -> Result<(), Error> {
    if opts.mark_of_the_web {
        crate::fs::motw::propagate(archive, dest)?;
    }
    Ok(())
}

/// Write `write_body` to a sibling temp file and rename it into place.
pub fn write_atomic(
    final_path: &Path,
    keep_broken: bool,
    write_body: impl FnOnce(&mut File) -> Result<(), Error>,
) -> Result<(), Error> {
    let tmp = partial_path(final_path);
    let mut file = File::create(extend_long_path(tmp.clone()))
        .map_err(|err| Error::io(format!("create {}", tmp.display()), err))?;
    let result = write_body(&mut file);
    if let Err(err) = file.flush() {
        if !keep_broken {
            let _ = std::fs::remove_file(&tmp);
        }
        return Err(Error::io(format!("flush {}", tmp.display()), err));
    }
    drop(file);
    if let Err(err) = result {
        if !keep_broken {
            let _ = std::fs::remove_file(&tmp);
        }
        return Err(err);
    }
    if final_path.exists() {
        std::fs::remove_file(final_path)
            .map_err(|err| Error::io(format!("replace {}", final_path.display()), err))?;
    }
    std::fs::rename(&tmp, final_path)
        .map_err(|err| Error::io(format!("rename {}", final_path.display()), err))?;
    Ok(())
}

fn exists(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

fn renamed(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    for index in 1..10_000 {
        let candidate = parent.join(format!("{stem} ({index}){ext}"));
        if !exists(&candidate) {
            return candidate;
        }
    }
    parent.join(format!("{stem} ({}){ext}", std::process::id()))
}

#[cfg(feature = "rar")]
pub(crate) fn stage_path(final_path: &Path) -> PathBuf {
    partial_path(final_path)
}

fn partial_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "entry".to_string());
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!(".{name}.{}.szip-partial", std::process::id()))
}
