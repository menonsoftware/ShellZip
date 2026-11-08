//! Per-user install and uninstall of file associations and menus.

use std::path::{Path, PathBuf};

use thiserror::Error;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

/// Failure while writing or removing shell integration.
#[derive(Debug, Error)]
pub enum IntegrateError {
    /// Operating-system I/O failure.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// A required path was missing or invalid.
    #[error("{0}")]
    Message(String),
}

/// Paths used when registering ShellZip with the file manager.
#[derive(Debug, Clone)]
pub struct IntegratePaths {
    /// Absolute path to the `shellzip` GUI binary.
    pub exe: PathBuf,
    /// Directory that holds committed format icons (`ico/`, `linux/hicolor`, …).
    pub icons_source: PathBuf,
    /// Optional path to the Windows shell-extension DLL.
    pub shell_dll: Option<PathBuf>,
}

impl IntegratePaths {
    /// Resolve defaults from the running executable and the workspace icons tree.
    pub fn from_exe(exe: impl Into<PathBuf>) -> Self {
        let exe = exe.into();
        let icons_source = find_icons_source(&exe)
            .unwrap_or_else(|| exe.parent().unwrap_or_else(|| Path::new(".")).join("icons"));
        let shell_dll = exe.parent().map(|dir| {
            #[cfg(windows)]
            {
                dir.join("shellzip_shell.dll")
            }
            #[cfg(not(windows))]
            {
                dir.join("libshellzip_shell.so")
            }
        });
        Self {
            exe,
            icons_source,
            shell_dll,
        }
    }
}

fn find_icons_source(exe: &Path) -> Option<PathBuf> {
    // Installed / Velopack layout: `icons/` next to the GUI binary.
    if let Some(parent) = exe.parent() {
        let beside = parent.join("icons");
        if beside.is_dir() {
            return Some(beside);
        }
    }
    // Dev layout: walk up for `grx/shellzip_file_icons`.
    let mut dir = exe.parent()?;
    for _ in 0..8 {
        let candidate = dir.join("grx").join("shellzip_file_icons");
        if candidate.is_dir() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
    None
}

/// Absolute GUI path stored by the last Windows `--integrate`, if it still exists.
#[cfg(windows)]
pub fn registered_host_exe() -> Option<PathBuf> {
    windows::registered_host_exe()
}

/// Install per-user associations, icons, and menus for this OS.
pub fn integrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    #[cfg(windows)]
    {
        windows::integrate(paths)
    }
    #[cfg(target_os = "linux")]
    {
        linux::integrate(paths)
    }
    #[cfg(target_os = "macos")]
    {
        macos::integrate(paths)
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = paths;
        Err(IntegrateError::Message(
            "shell integration is not supported on this OS".into(),
        ))
    }
}

/// Remove per-user associations, icons, and menus for this OS.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    #[cfg(windows)]
    {
        windows::unintegrate(paths)
    }
    #[cfg(target_os = "linux")]
    {
        linux::unintegrate(paths)
    }
    #[cfg(target_os = "macos")]
    {
        macos::unintegrate(paths)
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = paths;
        Err(IntegrateError::Message(
            "shell integration is not supported on this OS".into(),
        ))
    }
}
