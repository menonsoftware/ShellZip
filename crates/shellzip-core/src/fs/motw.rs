//! Mark-of-the-Web propagation.
//!
//! Windows stores the zone in an NTFS alternate data stream named
//! `Zone.Identifier`. The stream is opened through the ordinary path
//! `file:Zone.Identifier`, so this stays in safe Rust. The workspace forbids
//! `unsafe`, which a direct `windows-sys` call would need.

use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;

use crate::error::Error;

#[cfg(windows)]
const MAX_ZONE_BYTES: u64 = 64 * 1024;

/// Copy `archive`'s zone stream onto `dest` when the archive has one.
///
/// Missing streams are not an error. Other platforms do nothing.
pub fn propagate(archive: &Path, dest: &Path) -> Result<(), Error> {
    #[cfg(windows)]
    {
        copy_zone(archive, dest)
    }
    #[cfg(not(windows))]
    {
        let _ = (archive, dest);
        Ok(())
    }
}

#[cfg(windows)]
fn copy_zone(archive: &Path, dest: &Path) -> Result<(), Error> {
    let Some(source) = zone_path(archive) else {
        return Ok(());
    };
    let Some(target) = zone_path(dest) else {
        return Ok(());
    };
    let file = match std::fs::File::open(&source) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(Error::io(format!("open {}", source.display()), err)),
    };
    let len = file
        .metadata()
        .map_err(|err| Error::io(format!("stat {}", source.display()), err))?
        .len();
    if len > MAX_ZONE_BYTES {
        return Err(Error::Archive(
            "Zone.Identifier stream is unexpectedly large".to_string(),
        ));
    }
    let bytes = std::fs::read(&source)
        .map_err(|err| Error::io(format!("read {}", source.display()), err))?;
    std::fs::write(&target, bytes)
        .map_err(|err| Error::io(format!("write {}", target.display()), err))?;
    Ok(())
}

#[cfg(windows)]
fn zone_path(path: &Path) -> Option<PathBuf> {
    let text = path.to_str()?;
    if text.is_empty() || text.ends_with(['\\', '/']) {
        return None;
    }
    Some(PathBuf::from(format!("{text}:Zone.Identifier")))
}
