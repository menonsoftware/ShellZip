//! Per-user macOS Launch Services registration helpers.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{IntegrateError, IntegratePaths};
use crate::writers::macos_document_types;

/// Write document-type metadata next to the app and request default roles.
pub fn integrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let resources = app_resources(&paths.exe)?;
    fs::create_dir_all(&resources)?;
    fs::write(
        resources.join("ShellZipDocumentTypes.fragment.plist"),
        macos_document_types(),
    )?;
    copy_icns(&paths.icons_source, &resources)?;

    // Launch Services may prompt; failure is non-fatal for CI-less installs.
    let _ = Command::new("lsregister")
        .args([
            "-f",
            &paths
                .exe
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .unwrap_or_else(|| paths.exe.parent().unwrap_or(std::path::Path::new(".")))
                .to_string_lossy(),
        ])
        .status();
    Ok(())
}

/// Remove the document-type fragment and copied icns files.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    if let Ok(resources) = app_resources(&paths.exe) {
        let _ = fs::remove_file(resources.join("ShellZipDocumentTypes.fragment.plist"));
        if let Ok(entries) = fs::read_dir(&resources) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.ends_with(".icns") && name != "shellzip.icns" && name != "AppIcon.icns" {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    let _ = paths;
    Ok(())
}

fn app_resources(exe: &std::path::Path) -> Result<PathBuf, IntegrateError> {
    // Prefer Contents/Resources when running from a .app bundle.
    if let Some(macos) = exe.parent() {
        if macos.file_name().and_then(|n| n.to_str()) == Some("MacOS") {
            if let Some(contents) = macos.parent() {
                return Ok(contents.join("Resources"));
            }
        }
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| IntegrateError::Message("HOME is not set".into()))?;
    Ok(home.join("Library/Application Support/ShellZip/Resources"))
}

fn copy_icns(source: &std::path::Path, dest: &std::path::Path) -> Result<(), IntegrateError> {
    let icns = source.join("icns");
    if !icns.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(icns)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("icns") {
            fs::copy(&path, dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}
