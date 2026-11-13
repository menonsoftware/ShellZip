//! Overwrite policy and atomic file replacement.

use std::fs::File;
use std::io::{Read, Write};
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
    max_ratio: u64,
    /// Packed size of the whole archive, when the file length is known.
    archive_packed: Option<u64>,
}

impl Budget {
    /// Budget for `path`, using its packed length as the ratio baseline.
    ///
    /// A self-extracting stub is not counted: the trailer records the payload length.
    pub fn for_archive(path: &Path, max_bytes: u64, max_ratio: u64) -> Self {
        let archive_packed = archive_packed_len(path);
        Self {
            written: 0,
            max: max_bytes,
            max_ratio,
            archive_packed,
        }
    }

    /// Reject a member whose declared size cannot fit in the byte cap or the ratio cap.
    ///
    /// `member_packed` is that member's compressed size. `None` skips the
    /// per-member ratio and still checks the archive-wide ratio.
    pub fn reserve(&self, declared: u64, member_packed: Option<u64>) -> Result<(), Error> {
        if declared > self.max.saturating_sub(self.written) {
            return Err(Error::Usage(format!(
                "refusing to extract {declared} bytes; {remaining} bytes remain of the {max} byte cap",
                remaining = self.max.saturating_sub(self.written),
                max = self.max
            )));
        }
        if self.max_ratio > 0 {
            if let Some(packed) = member_packed {
                let allowed = packed.saturating_mul(self.max_ratio);
                if declared > allowed {
                    return Err(ratio_error(declared, packed, self.max_ratio));
                }
            }
            if let Some(packed) = self.archive_packed.filter(|n| *n > 0) {
                let used = self.written.saturating_add(declared);
                let allowed = packed.saturating_mul(self.max_ratio);
                if used > allowed {
                    return Err(ratio_error(used, packed, self.max_ratio));
                }
            }
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

    /// Error for a reader that still has bytes after [`Self::room`] hit zero.
    fn blocked(&self) -> Error {
        if self.max_ratio > 0
            && let Some(packed) = self.archive_packed.filter(|n| *n > 0)
        {
            let allowed = packed.saturating_mul(self.max_ratio);
            if self.written >= allowed && self.written < self.max {
                return ratio_error(self.written, packed, self.max_ratio);
            }
        }
        Error::Usage(format!("refusing to extract more than {} bytes", self.max))
    }

    /// Uncompressed bytes still allowed by the tighter of the two caps.
    fn room(&self) -> u64 {
        let by_bytes = self.max.saturating_sub(self.written);
        if self.max_ratio == 0 {
            return by_bytes;
        }
        let Some(packed) = self.archive_packed.filter(|n| *n > 0) else {
            return by_bytes;
        };
        let by_ratio = packed
            .saturating_mul(self.max_ratio)
            .saturating_sub(self.written);
        by_bytes.min(by_ratio)
    }
}

fn ratio_error(uncompressed: u64, packed: u64, max_ratio: u64) -> Error {
    Error::Usage(format!(
        "refusing {uncompressed} uncompressed bytes from {packed} packed bytes; ratio cap is {max_ratio}:1"
    ))
}

fn archive_packed_len(path: &Path) -> Option<u64> {
    if let Ok(Some(trailer)) = crate::sfx::locate(path) {
        return Some(trailer.archive_len);
    }
    std::fs::metadata(path).ok().map(|meta| meta.len())
}

/// Copy `src` to `dst`, stopping when `budget` is exhausted and more bytes remain.
pub fn copy_limited(
    src: &mut dyn Read,
    dst: &mut dyn Write,
    budget: &mut Budget,
    context: &str,
) -> Result<u64, Error> {
    let mut buf = vec![0u8; 256 * 1024];
    let mut total = 0u64;
    loop {
        let room = budget.room();
        if room == 0 {
            let mut probe = [0u8; 1];
            let extra = src
                .read(&mut probe)
                .map_err(|err| Error::io(context.to_string(), err))?;
            if extra == 0 {
                break;
            }
            return Err(budget.blocked());
        }
        let max = usize::try_from(room).unwrap_or(usize::MAX).min(buf.len());
        let n = src
            .read(&mut buf[..max])
            .map_err(|err| Error::io(context.to_string(), err))?;
        if n == 0 {
            break;
        }
        dst.write_all(&buf[..n])
            .map_err(|err| Error::io(context.to_string(), err))?;
        let bytes = u64::try_from(n).map_err(|_| {
            Error::Archive(format!("read size does not fit in u64 while {context}"))
        })?;
        budget.charge(bytes)?;
        total = total.saturating_add(bytes);
    }
    Ok(total)
}

/// `Write` adapter that charges `budget` and stops at the cap.
#[cfg(feature = "sevenz")]
pub struct BudgetWriter<'a, W> {
    /// Destination for accepted bytes.
    pub inner: W,
    /// Running extract cap.
    pub budget: &'a mut Budget,
}

#[cfg(feature = "sevenz")]
impl<W: Write> Write for BudgetWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let room = self.budget.room();
        if room == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::QuotaExceeded,
                self.budget.blocked().to_string(),
            ));
        }
        let max = usize::try_from(room).unwrap_or(usize::MAX).min(buf.len());
        let n = self.inner.write(&buf[..max])?;
        let bytes = u64::try_from(n).unwrap_or(u64::MAX);
        self.budget.charge(bytes).map_err(|err| {
            std::io::Error::new(std::io::ErrorKind::QuotaExceeded, err.to_string())
        })?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
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

/// A symlink that is safe to create inside the destination.
#[derive(Debug)]
pub struct SymlinkExtract {
    /// Path of the link inside the destination.
    pub path: PathBuf,
    /// Target text to store. An absolute target that names another member of
    /// the same tree is rewritten relative to [`Self::path`].
    pub target: String,
}

/// Validate a symlink and return where to create it.
///
/// Absolute targets are not written as absolute paths. When the target names
/// another member of this archive, it is stored as a relative link under the
/// destination the caller chose. Targets that still leave that directory are
/// refused unless `allow_unsafe_links` is set.
///
/// # Errors
///
/// [`Error::UnsafePath`] when the link name is empty or the target would leave
/// the destination.
pub fn symlink_destination(
    raw_name: &str,
    target: &str,
    opts: &ExtractOpts,
) -> Result<SymlinkExtract, Error> {
    let archive_name = raw_name.trim_matches(['/', '\\']).to_string();
    let placed = strip_prefix(&archive_name, &opts.strip_prefix);
    let placed = if opts.flat {
        placed
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(placed.as_str())
            .to_string()
    } else {
        placed
    };
    if placed.is_empty() {
        return Err(Error::UnsafePath("symlink has an empty name".to_string()));
    }
    let rel = safe_rel(&placed)?;
    let path = safe_join(&opts.output, &rel)?;
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let target = symlink_target_text(&archive_name, &path, target, opts)?;
    Ok(SymlinkExtract { path, target })
}

/// Target text to store for a link already placed at `link_path`.
///
/// `archive_name` is the member name before strip or flat placement.
pub(crate) fn symlink_target_text(
    archive_name: &str,
    link_path: &Path,
    target: &str,
    opts: &ExtractOpts,
) -> Result<String, Error> {
    if target.is_empty() || target.chars().any(char::is_control) {
        return Err(Error::UnsafePath(target.to_string()));
    }
    if opts.allow_unsafe_links {
        return Ok(target.to_string());
    }
    if let Some(relative) =
        rebase_absolute_symlink(archive_name, target, &opts.strip_prefix, opts.flat)
    {
        symlink_stays_inside(&opts.output, link_path, &relative)?;
        return Ok(relative);
    }
    symlink_stays_inside(&opts.output, link_path, target)?;
    Ok(target.to_string())
}

/// Rewrite an absolute link onto the archive tree, then onto the stripped destination.
fn rebase_absolute_symlink(
    archive_name: &str,
    target: &str,
    prefix: &str,
    flat: bool,
) -> Option<String> {
    if flat || !is_absolute_link_target(target) {
        return None;
    }
    let archive_target = infer_archive_target(archive_name, target)?;
    let stripped_link = strip_prefix(archive_name, prefix);
    let stripped_target = strip_prefix(&archive_target, prefix);
    if stripped_link.is_empty() || stripped_target.is_empty() {
        return None;
    }
    relative_link(&stripped_link, &stripped_target)
}

fn is_absolute_link_target(target: &str) -> bool {
    let bytes = target.as_bytes();
    target.starts_with('/')
        || target.starts_with('\\')
        || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
        || Path::new(target).is_absolute()
}

/// Archive-relative path of `absolute`, using `link_rel`'s directory as the anchor.
fn infer_archive_target(link_rel: &str, absolute: &str) -> Option<String> {
    let link_parts = lexical_parts(link_rel)?;
    if link_parts.len() < 2 {
        return None;
    }
    let target_parts = lexical_parts(absolute)?;
    if target_parts.is_empty() {
        return None;
    }
    let link_dir = &link_parts[..link_parts.len() - 1];
    let target_dir = &target_parts[..target_parts.len() - 1];
    let (_matched, start) = find_anchor(link_dir, target_dir)?;
    Some(target_parts[start..].join("/"))
}

/// Longest prefix of `link_dir` that occurs in `target_dir`, preferring the rightmost hit.
fn find_anchor(link_dir: &[String], target_dir: &[String]) -> Option<(usize, usize)> {
    let max = link_dir.len().min(target_dir.len());
    for len in (1..=max).rev() {
        let last = target_dir.len() - len;
        for start in (0..=last).rev() {
            if components_eq(&target_dir[start..start + len], &link_dir[..len]) {
                return Some((len, start));
            }
        }
    }
    None
}

fn relative_link(from_file: &str, to_file: &str) -> Option<String> {
    let from = lexical_parts(from_file)?;
    let to = lexical_parts(to_file)?;
    if from.is_empty() || to.is_empty() {
        return None;
    }
    let from_dir = &from[..from.len() - 1];
    let to_dir = &to[..to.len() - 1];
    let mut common = 0;
    while common < from_dir.len()
        && common < to_dir.len()
        && component_eq(&from_dir[common], &to_dir[common])
    {
        common += 1;
    }
    let mut out = Vec::new();
    for _ in 0..(from_dir.len() - common) {
        out.push("..".to_string());
    }
    out.extend(to_dir[common..].iter().cloned());
    out.push(to[to.len() - 1].clone());
    Some(out.join("/"))
}

fn lexical_parts(raw: &str) -> Option<Vec<String>> {
    if raw.is_empty() || raw.chars().any(char::is_control) {
        return None;
    }
    let mut norm = raw.replace('\\', "/");
    let bytes = norm.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        norm = norm[2..].to_string();
    }
    let mut parts = Vec::new();
    for component in norm.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            parts.pop()?;
            continue;
        }
        if component.contains(':') {
            return None;
        }
        parts.push(component.to_string());
    }
    Some(parts)
}

fn components_eq(left: &[String], right: &[String]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| component_eq(a, b))
}

fn component_eq(left: &str, right: &str) -> bool {
    if cfg!(windows) {
        left.eq_ignore_ascii_case(right)
    } else {
        left == right
    }
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

    #[test]
    fn ratio_cap_rejects_a_member_that_expands_too_far() {
        let budget = Budget::for_archive(Path::new("missing-budget-archive"), 1_000_000, 10);
        let err = budget.reserve(100, Some(5)).unwrap_err();
        assert!(err.to_string().contains("ratio cap"), "{err}");
        budget.reserve(50, Some(5)).unwrap();
        budget.reserve(1_000, None).unwrap();
    }

    #[test]
    fn zero_ratio_disables_the_check() {
        let budget = Budget::for_archive(Path::new("missing-budget-archive"), 1_000, 0);
        budget.reserve(500, Some(1)).unwrap();
    }

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

    #[test]
    fn absolute_symlink_inside_the_archive_is_rewritten_into_the_destination() {
        let (guard, root) = scratch();
        let opts = ExtractOpts::new(&root);
        let link = symlink_destination(
            "code/rea/ghidra-release",
            "/home/ajaymenon/code/rea/ghidra_12.1.4_PUBLIC",
            &opts,
        )
        .unwrap();
        assert_eq!(link.target, "ghidra_12.1.4_PUBLIC");
        assert_eq!(
            link.path,
            root.join("code").join("rea").join("ghidra-release")
        );
        drop(guard);
    }

    #[test]
    fn stripped_absolute_symlink_stays_beside_its_target() {
        let (guard, root) = scratch();
        let mut opts = ExtractOpts::new(&root);
        opts.strip_prefix = "code/rea".to_string();
        let link = symlink_destination(
            "code/rea/ghidra-release",
            "/home/ajaymenon/code/rea/ghidra_12.1.4_PUBLIC",
            &opts,
        )
        .unwrap();
        assert_eq!(link.target, "ghidra_12.1.4_PUBLIC");
        assert_eq!(link.path, root.join("ghidra-release"));
        drop(guard);
    }

    #[test]
    fn absolute_symlink_deeper_than_its_target_uses_parent_segments() {
        let (guard, root) = scratch();
        let opts = ExtractOpts::new(&root);
        let link = symlink_destination(
            "code/rea/nested/link",
            "/home/ajaymenon/code/rea/ghidra_12.1.4_PUBLIC",
            &opts,
        )
        .unwrap();
        assert_eq!(link.target, "../ghidra_12.1.4_PUBLIC");
        drop(guard);
    }

    #[test]
    fn absolute_symlink_outside_the_archive_is_still_refused() {
        let (guard, root) = scratch();
        let opts = ExtractOpts::new(&root);
        let err = symlink_destination("code/rea/ghidra-release", "/etc/passwd", &opts).unwrap_err();
        assert!(matches!(err, Error::UnsafePath(_)), "{err}");
        let mut allowed = ExtractOpts::new(&root);
        allowed.allow_unsafe_links = true;
        let link = symlink_destination("code/rea/ghidra-release", "/etc/passwd", &allowed).unwrap();
        assert_eq!(link.target, "/etc/passwd");
        drop(guard);
    }
}
