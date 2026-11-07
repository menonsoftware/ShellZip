//! `szip s` and the stub lookup used by `szip a --sfx`.

use std::path::{Path, PathBuf};

use shellzip_core::{Error, SfxDirection, convert, stub_file_names};

use super::Shared;
use crate::cli::SfxArgs;

/// Convert `args.archive` between a ZIP and a self-extracting package.
pub fn sfx(args: &SfxArgs, shared: &Shared) -> Result<i32, Error> {
    let dest = args
        .output
        .clone()
        .unwrap_or_else(|| default_output(&args.archive));
    let stub = if args.sfx_stub.is_some() || !has_trailer(&args.archive)? {
        Some(find_stub(args.sfx_stub.as_deref())?)
    } else {
        None
    };
    let direction = convert(&args.archive, &dest, stub.as_deref(), shared.sink.as_ref())?;
    if shared.quiet < 2 {
        match direction {
            SfxDirection::Wrapped => println!("created {}", dest.display()),
            SfxDirection::Stripped => println!("extracted {}", dest.display()),
        }
    }
    Ok(0)
}

/// Resolve the stub from the flag, `SZIP_SFX_STUB`, or a file beside `szip`.
pub(crate) fn find_stub(explicit: Option<&Path>) -> Result<PathBuf, Error> {
    if let Some(path) = explicit {
        return existing(path);
    }
    if let Some(value) = std::env::var_os("SZIP_SFX_STUB") {
        return existing(Path::new(&value));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        for name in stub_file_names() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(Error::Usage(
        "SFX stub not found; pass --sfx-stub or set SZIP_SFX_STUB to shellzip-sfx".to_string(),
    ))
}

fn existing(path: &Path) -> Result<PathBuf, Error> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(Error::io(
            format!("open {}", path.display()),
            std::io::Error::new(std::io::ErrorKind::NotFound, "SFX stub not found"),
        ))
    }
}

fn has_trailer(path: &Path) -> Result<bool, Error> {
    Ok(shellzip_core::locate(path)?.is_some())
}

fn default_output(source: &Path) -> PathBuf {
    let stem = source
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_string());
    let name = if let Some(trailer) = shellzip_core::locate(source).ok().flatten() {
        format!("{stem}.{}", trailer.payload().extension())
    } else if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        format!("{stem}.sfx")
    };
    source.with_file_name(name)
}
