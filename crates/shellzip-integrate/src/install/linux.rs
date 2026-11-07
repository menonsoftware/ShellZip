//! Per-user Linux desktop entry, MIME defaults, icons, and service menus.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{IntegrateError, IntegratePaths};
use crate::formats::MIME_TYPES;
use crate::writers::{desktop_entry, kde_servicemenu, nemo_actions, thunar_uca};

/// Install desktop file, hicolor icons, and file-manager actions.
pub fn integrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let exe = paths.exe.to_string_lossy();
    let home = home_dir()?;
    let apps = home.join(".local/share/applications");
    let icons = home.join(".local/share/icons/hicolor");
    let kde = home.join(".local/share/kio/servicemenus");
    let nemo = home.join(".local/share/nemo/actions");
    let thunar = home.join(".config/Thunar");
    fs::create_dir_all(&apps)?;
    fs::create_dir_all(&icons)?;
    fs::create_dir_all(&kde)?;
    fs::create_dir_all(&nemo)?;
    fs::create_dir_all(&thunar)?;

    fs::write(apps.join("shellzip.desktop"), desktop_entry(&exe))?;
    install_hicolor(&paths.icons_source, &icons)?;
    fs::write(kde.join("shellzip.desktop"), kde_servicemenu(&exe))?;
    for (name, body) in nemo_actions(&exe) {
        fs::write(nemo.join(name), body)?;
    }
    fs::write(thunar.join("shellzip-uca.xml"), thunar_uca(&exe))?;

    let _ = Command::new("update-desktop-database").arg(&apps).status();
    let _ = Command::new("gtk-update-icon-cache")
        .args(["-f", &icons.to_string_lossy()])
        .status();
    for mime in MIME_TYPES {
        let _ = Command::new("xdg-mime")
            .args(["default", "shellzip.desktop", mime])
            .status();
    }
    Ok(())
}

/// Remove the files this crate installed under the home directory.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let home = home_dir()?;
    let _ = fs::remove_file(home.join(".local/share/applications/shellzip.desktop"));
    let _ = fs::remove_file(home.join(".local/share/kio/servicemenus/shellzip.desktop"));
    let nemo = home.join(".local/share/nemo/actions");
    for (name, _) in nemo_actions("shellzip") {
        let _ = fs::remove_file(nemo.join(name));
    }
    let _ = fs::remove_file(home.join(".config/Thunar/shellzip-uca.xml"));
    remove_shellzip_mime_icons(&home.join(".local/share/icons/hicolor"))?;
    let _ = paths;
    Ok(())
}

fn home_dir() -> Result<PathBuf, IntegrateError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| IntegrateError::Message("HOME is not set".into()))
}

fn install_hicolor(source: &Path, dest: &Path) -> Result<(), IntegrateError> {
    let src = source.join("linux").join("hicolor");
    if !src.is_dir() {
        return Ok(());
    }
    copy_dir(&src, dest)?;
    // Alias for the other common RAR MIME name.
    let mime_dir = dest.join("scalable/mimetypes");
    let rar = mime_dir.join("application-vnd.rar.svg");
    let alias = mime_dir.join("application-x-rar-compressed.svg");
    if rar.is_file() && !alias.is_file() {
        fs::copy(&rar, &alias)?;
    }
    Ok(())
}

const MIME_ICON_STEMS: &[&str] = &[
    "application-zip",
    "application-x-7z-compressed",
    "application-vnd.rar",
    "application-x-rar-compressed",
    "application-gzip",
    "application-x-bzip2",
    "application-x-xz",
    "application-zstd",
    "application-x-tar",
    "application-x-compressed-tar",
    "application-x-bzip-compressed-tar",
    "application-x-xz-compressed-tar",
    "application-x-zstd-compressed-tar",
];

/// Pixel sizes under `linux/hicolor` plus the scalable SVG tree.
const HICOLOR_SIZES: &[&str] = &[
    "16x16", "22x22", "24x24", "32x32", "48x48", "64x64", "128x128", "256x256", "512x512",
    "scalable",
];

fn remove_shellzip_mime_icons(hicolor: &Path) -> Result<(), IntegrateError> {
    for size in HICOLOR_SIZES {
        let mime_dir = hicolor.join(size).join("mimetypes");
        if !mime_dir.is_dir() {
            continue;
        }
        let ext = if *size == "scalable" { "svg" } else { "png" };
        for stem in MIME_ICON_STEMS {
            let _ = fs::remove_file(mime_dir.join(format!("{stem}.{ext}")));
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), IntegrateError> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
