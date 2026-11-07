//! Destination folder name derived from an archive path.

use std::path::Path;

/// Stem used by **Extract Here (name)**. Compound archive suffixes are stripped.
pub fn folder_stem(path: &Path) -> String {
    // Split on both separators so a Windows path still yields a basename when
    // this crate is built and tested on Unix (CI).
    let Some(name) = path
        .to_str()
        .and_then(|s| s.rsplit(['/', '\\']).find(|part| !part.is_empty()))
    else {
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
    Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("archive")
        .to_string()
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
}
