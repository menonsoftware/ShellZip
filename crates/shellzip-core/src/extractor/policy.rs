//! Overwrite policy and atomic file replacement.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::fs::safe_path::{
    ensure_dir, extend_long_path, safe_join, safe_rel, symlink_stays_inside,
};
use crate::options::{ConflictChoice, ConflictDecider, ExtractOpts, ExtractUpdate, Overwrite};

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

/// Resolve `raw` inside `opts.output`, applying update mode and the overwrite policy.
///
/// `modified` is the archive copy's modification time. Update and freshen use it
/// to decide whether an existing file is older.
pub fn destination(
    raw: &str,
    opts: &ExtractOpts,
    flat: bool,
    is_dir: bool,
    modified: Option<std::time::SystemTime>,
    decide: Option<&ConflictDecider<'_>>,
) -> Result<Destination, Error> {
    let raw = strip_prefix(raw, &opts.strip_prefix);
    if raw.is_empty() {
        if is_dir && admit_existing(opts.update, &opts.output, modified, true) {
            ensure_dir(&opts.output)?;
            return Ok(Destination::Write(opts.output.clone()));
        }
        return Ok(Destination::Skip);
    }
    let rel = if flat && !is_dir {
        let name = raw.rsplit(['/', '\\']).next().unwrap_or(raw.as_str());
        safe_rel(name)?
    } else {
        safe_rel(&raw)?
    };
    let path = safe_join(&opts.output, &rel)?;
    if is_dir {
        if !admit_existing(opts.update, &path, modified, true) {
            return Ok(Destination::Skip);
        }
        ensure_dir(&path)?;
        return Ok(Destination::Write(path));
    }
    if !admit_existing(opts.update, &path, modified, false) {
        return Ok(Destination::Skip);
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

/// Write the archive modification time when the extract options ask for it.
pub fn restore_entry_time(
    path: &Path,
    time: Option<std::time::SystemTime>,
    opts: &ExtractOpts,
) -> Result<(), Error> {
    if !opts.restore_mtime {
        return Ok(());
    }
    let Some(time) = time else {
        return Ok(());
    };
    crate::fs::metadata::restore_mtime(path, time)
}

/// Validate a symlink target and return the link path when it is safe.
pub fn symlink_destination(
    raw_name: &str,
    target: &str,
    opts: &ExtractOpts,
) -> Result<PathBuf, Error> {
    let raw_name = strip_prefix(raw_name, &opts.strip_prefix);
    let raw_name = if opts.flat {
        raw_name
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(raw_name.as_str())
            .to_string()
    } else {
        raw_name
    };
    if raw_name.is_empty() {
        return Err(Error::UnsafePath("symlink has an empty name".to_string()));
    }
    let rel = safe_rel(&raw_name)?;
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

fn strip_prefix(raw: &str, prefix: &str) -> String {
    let raw_trim = raw.trim_matches(['/', '\\']);
    let prefix = prefix.trim_matches(['/', '\\']);
    if prefix.is_empty() {
        return raw_trim.to_string();
    }
    let raw_norm = raw_trim.replace('\\', "/");
    let prefix_norm = prefix.replace('\\', "/");
    if raw_norm.eq_ignore_ascii_case(&prefix_norm) {
        return String::new();
    }
    let head = format!("{prefix_norm}/");
    if raw_norm.len() >= head.len() && raw_norm[..head.len()].eq_ignore_ascii_case(&head) {
        return raw_norm[head.len()..].to_string();
    }
    raw_trim.to_string()
}

/// Freshen skips a path that is not already on disk. Update and freshen skip an
/// existing file unless the archive copy is strictly newer. Directories that
/// already exist are kept.
fn admit_existing(
    update: ExtractUpdate,
    path: &Path,
    modified: Option<std::time::SystemTime>,
    is_dir: bool,
) -> bool {
    if !exists(path) {
        return !matches!(update, ExtractUpdate::Freshen);
    }
    if is_dir {
        return true;
    }
    match update {
        ExtractUpdate::Replace => true,
        ExtractUpdate::Update | ExtractUpdate::Freshen => archive_is_newer(modified, path),
    }
}

fn archive_is_newer(archive: Option<std::time::SystemTime>, path: &Path) -> bool {
    let Some(archive) = archive else {
        return false;
    };
    let Some(disk) = disk_mtime(path) else {
        return false;
    };
    archive > disk
}

fn disk_mtime(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::symlink_metadata(path).ok()?.modified().ok()
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

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use filetime::{FileTime, set_file_mtime};

    use super::*;

    struct Remove(PathBuf);

    impl Drop for Remove {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch() -> (Remove, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "shellzip-policy-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        (Remove(path.clone()), path)
    }

    fn stamp(path: &Path, secs: u64) {
        set_file_mtime(
            path,
            FileTime::from_system_time(UNIX_EPOCH + Duration::from_secs(secs)),
        )
        .unwrap();
    }

    fn touch(path: &Path) {
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(b"x").unwrap();
    }

    #[test]
    fn freshen_skips_a_missing_file() {
        let (guard, root) = scratch();
        let mut opts = ExtractOpts::new(&root);
        opts.update = ExtractUpdate::Freshen;
        let dest = destination("docs/a.txt", &opts, false, false, Some(UNIX_EPOCH), None).unwrap();
        assert!(matches!(dest, Destination::Skip));
        assert!(!root.join("docs").exists());
        drop(guard);
    }

    #[test]
    fn update_skips_an_older_archive_copy() {
        let (guard, root) = scratch();
        let file = root.join("a.txt");
        touch(&file);
        stamp(&file, 200);
        let mut opts = ExtractOpts::new(&root);
        opts.update = ExtractUpdate::Update;
        opts.overwrite = Overwrite::Always;
        let dest = destination(
            "a.txt",
            &opts,
            false,
            false,
            Some(UNIX_EPOCH + Duration::from_secs(100)),
            None,
        )
        .unwrap();
        assert!(matches!(dest, Destination::Skip));
        drop(guard);
    }

    #[test]
    fn freshen_replaces_an_older_file_and_replace_still_honors_overwrite() {
        let (guard, root) = scratch();
        let file = root.join("a.txt");
        touch(&file);
        stamp(&file, 50);
        let newer = Some(UNIX_EPOCH + Duration::from_secs(80));
        let mut freshen = ExtractOpts::new(&root);
        freshen.update = ExtractUpdate::Freshen;
        freshen.overwrite = Overwrite::Always;
        let dest = destination("a.txt", &freshen, false, false, newer, None).unwrap();
        assert!(matches!(dest, Destination::Write(path) if path == file));

        let mut replace = ExtractOpts::new(&root);
        replace.overwrite = Overwrite::Never;
        let dest = destination("a.txt", &replace, false, false, newer, None).unwrap();
        assert!(matches!(dest, Destination::Skip));
        drop(guard);
    }

    #[test]
    fn relative_prefix_is_removed_before_the_name_is_joined() {
        let (guard, root) = scratch();
        let mut opts = ExtractOpts::new(&root);
        opts.strip_prefix = "Backup/MyData".to_string();
        let dest = destination(
            "Backup/MyData/Images/a.txt",
            &opts,
            false,
            false,
            None,
            None,
        )
        .unwrap();
        let Destination::Write(path) = dest else {
            panic!("expected a write");
        };
        assert_eq!(path, root.join("Images").join("a.txt"));
        drop(guard);
    }
}
