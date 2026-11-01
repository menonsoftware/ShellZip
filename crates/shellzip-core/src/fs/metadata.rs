//! Filesystem metadata captured while packing and restored while extracting.

use std::path::Path;
use std::time::SystemTime;

use filetime::{FileTime, set_file_mtime};

use crate::entry::{EntryKind, EntryMeta};
use crate::error::Error;

/// Read size, mtime, and permission bits without following a symlink the caller
/// already classified.
pub fn capture(path: &Path, kind: EntryKind) -> Result<EntryMeta, Error> {
    let meta = std::fs::symlink_metadata(path)
        .map_err(|err| Error::io(format!("stat {}", path.display()), err))?;
    Ok(EntryMeta {
        size: if kind == EntryKind::File {
            meta.len()
        } else {
            0
        },
        modified: meta.modified().ok(),
        mode: mode_bits(&meta, kind),
        size_known: true,
    })
}

/// Restore a modification time. Failures are reported by the caller as warnings.
pub fn restore_mtime(path: &Path, time: SystemTime) -> Result<(), Error> {
    set_file_mtime(path, FileTime::from_system_time(time))
        .map_err(|err| Error::io(format!("set mtime {}", path.display()), err))
}

/// Restore unix permission bits. No-op on Windows.
pub fn restore_mode(path: &Path, mode: u32) -> Result<(), Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o777))
            .map_err(|err| Error::io(format!("set mode {}", path.display()), err))?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}

/// Create `link_path` pointing at `target`. `target` is stored as given.
pub fn create_symlink(target: &str, link_path: &Path) -> Result<(), Error> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link_path)
            .map_err(|err| Error::io(format!("symlink {}", link_path.display()), err))
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link_path)
            .map_err(|err| Error::io(format!("symlink {}", link_path.display()), err))
    }
}

#[cfg(unix)]
fn mode_bits(meta: &std::fs::Metadata, _kind: EntryKind) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o777
}

#[cfg(not(unix))]
fn mode_bits(_meta: &std::fs::Metadata, kind: EntryKind) -> u32 {
    match kind {
        EntryKind::Dir => 0o755,
        EntryKind::Symlink => 0o777,
        _ => 0o644,
    }
}
