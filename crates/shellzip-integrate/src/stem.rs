//! Destination folder name derived from an archive path.

use std::path::Path;

/// Stem used by **Extract Here (name)**. Compound archive suffixes are stripped.
pub fn folder_stem(path: &Path) -> String {
    let Some(name) = archive_file_name(path) else {
        return "archive".to_string();
    };
    let lower = name.to_ascii_lowercase();
    for suffix in [
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.zst", ".tgz", ".tbz2", ".txz", ".tzst", ".zipx",
        ".zip", ".7z", ".rar", ".tar", ".gz", ".bz2", ".xz", ".zst",
    ] {
        if lower.ends_with(suffix) {
            let cut = name.len().saturating_sub(suffix.len());
            let stem = &name[..cut];
            if stem.is_empty() {
                return "archive".to_string();
            }
            return stem.to_string();
        }
    }
    // `name` is a single path component (may contain `\` on Unix).
    Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("archive")
        .to_string()
}

/// Basename of `path`, respecting host separators.
///
/// On Unix, `\` is a valid name character, so we use `Path::file_name()`.
/// Tests (and rare cross-compiled strings) may still pass a Windows path
/// like `C:\a\photos.tgz`; those are rewritten to the last segment only.
fn archive_file_name(path: &Path) -> Option<&str> {
    let name = path.file_name()?.to_str()?;
    #[cfg(not(windows))]
    if looks_like_windows_path(name) {
        return name.rsplit(['/', '\\']).find(|part| !part.is_empty());
    }
    Some(name)
}

/// True when `s` is a whole Windows path that Unix `Path` kept as one component.
#[cfg(not(windows))]
fn looks_like_windows_path(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
    {
        return true;
    }
    // UNC: \\server\share\file
    let rest = s.strip_prefix(r"\\").unwrap_or("");
    !rest.is_empty() && !rest.starts_with('\\') && rest.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn strips_compound_suffixes() {
        assert_eq!(folder_stem(&PathBuf::from("report.zip")), "report");
        assert_eq!(folder_stem(&PathBuf::from("backup.tar.gz")), "backup");
        assert_eq!(folder_stem(&PathBuf::from(r"C:\a\photos.tgz")), "photos");
        assert_eq!(folder_stem(&PathBuf::from("data.TAR.ZST")), "data");
        assert_eq!(folder_stem(&PathBuf::from(".zip")), "archive");
    }

    #[cfg(unix)]
    #[test]
    fn unix_backslash_stays_in_the_stem() {
        // On Unix `\` is not a separator; the file name is literally `foo\bar.tar.gz`.
        assert_eq!(folder_stem(Path::new(r"dir/foo\bar.tar.gz")), r"foo\bar");
        assert_eq!(folder_stem(Path::new(r"weird\name.zip")), r"weird\name");
    }

    #[cfg(not(windows))]
    #[test]
    fn windows_drive_path_still_stems_on_unix() {
        assert_eq!(folder_stem(Path::new(r"C:\a\photos.tgz")), "photos");
        assert_eq!(folder_stem(Path::new(r"D:/archives/app.zip")), "app");
        assert_eq!(
            folder_stem(Path::new(r"\\server\share\pack.tar.gz")),
            "pack"
        );
    }
}
