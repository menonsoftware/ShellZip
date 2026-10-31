//! One directory level from the filesystem. Archive bytes stay in `shellzip-core`.

use std::path::Path;

use super::model::{self, Row, RowKind, Target};

/// Immediate children of `path`. A bad directory entry is skipped.
pub fn read_directory(path: &Path) -> Result<Vec<Row>, String> {
    let iter =
        std::fs::read_dir(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;
    let mut rows = Vec::new();
    for item in iter {
        let Ok(item) = item else {
            continue;
        };
        let file_name = item.file_name();
        let name = file_name.to_string_lossy();
        if name == "." || name == ".." {
            continue;
        }
        rows.push(row_for(&name, &item));
    }
    Ok(rows)
}

/// Immediate child directories of `path`. A bad entry is skipped.
pub fn read_subdirs(path: &Path) -> Vec<std::path::PathBuf> {
    let Ok(iter) = std::fs::read_dir(path) else {
        return Vec::new();
    };
    let mut dirs = Vec::new();
    for item in iter {
        let Ok(item) = item else {
            continue;
        };
        let child = item.path();
        let is_dir = std::fs::metadata(&child).is_ok_and(|meta| meta.is_dir());
        if is_dir {
            dirs.push(child);
        }
    }
    dirs.sort_by(|left, right| {
        let left = left.file_name().unwrap_or(left.as_os_str());
        let right = right.file_name().unwrap_or(right.as_os_str());
        left.to_string_lossy()
            .to_lowercase()
            .cmp(&right.to_string_lossy().to_lowercase())
    });
    dirs
}

fn row_for(name: &str, item: &std::fs::DirEntry) -> Row {
    let path = item.path();
    let link = item.file_type().is_ok_and(|kind| kind.is_symlink());
    let followed = std::fs::metadata(&path).ok();
    let is_dir = followed.as_ref().is_some_and(|meta| meta.is_dir());
    let kind = if is_dir {
        RowKind::Directory
    } else if model::is_supported_archive(&path) {
        RowKind::Archive
    } else if link {
        RowKind::Symlink
    } else {
        RowKind::File
    };
    let meta = followed.or_else(|| item.metadata().ok());
    let size = meta.as_ref().and_then(|meta| {
        if kind == RowKind::Directory {
            None
        } else {
            Some(meta.len())
        }
    });
    let modified = meta.as_ref().and_then(|meta| meta.modified().ok());
    Row {
        type_label: model::type_label(name, kind),
        name: name.to_string(),
        kind,
        size,
        packed: None,
        modified,
        crc32: None,
        key: path.display().to_string(),
        target: Target::Path(path),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::read_directory;
    use crate::explorer::RowKind;

    struct TempDir(PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn lists_a_folder_and_a_text_file() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = TempDir(std::env::temp_dir().join(format!("shellzip-explorer-{nanos}")));
        fs::create_dir_all(dir.0.join("talos")).unwrap();
        fs::write(dir.0.join("prompt.txt"), b"hello").unwrap();

        let rows = read_directory(&dir.0).unwrap();
        let folder = rows.iter().find(|row| row.name == "talos").unwrap();
        let file = rows.iter().find(|row| row.name == "prompt.txt").unwrap();
        assert_eq!(folder.kind, RowKind::Directory);
        assert_eq!(folder.type_label, "File folder");
        assert_eq!(folder.size, None);
        assert_eq!(file.kind, RowKind::File);
        assert_eq!(file.type_label, "Text Document");
        assert_eq!(file.size, Some(5));
    }

    #[test]
    fn subdirs_skip_files() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = TempDir(std::env::temp_dir().join(format!("shellzip-subdirs-{nanos}")));
        fs::create_dir_all(dir.0.join("talos")).unwrap();
        fs::write(dir.0.join("prompt.txt"), b"hello").unwrap();

        let dirs = super::read_subdirs(&dir.0);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].file_name().unwrap(), "talos");
    }
}
