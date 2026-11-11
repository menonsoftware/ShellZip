//! Per-user Linux desktop entry, MIME defaults, icons, and service menus.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::{IntegrateError, IntegratePaths};
use crate::formats::MIME_TYPES;
use crate::writers::{
    APP_ICON_NAME, desktop_entry, kde_servicemenus, merge_thunar_uca, nautilus_python_extension,
    nautilus_scripts, nemo_actions, pcmanfm_actions, strip_thunar_shellzip_actions,
};

/// Install desktop file, hicolor icons, and file-manager actions.
pub fn integrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let home = home_dir()?;
    integrate_into(&home, paths)?;
    refresh_desktop(&home, true);
    hint_nautilus_python();
    Ok(())
}

/// Remove the files this crate installed under the home directory.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let home = home_dir()?;
    unintegrate_into(&home, paths)?;
    refresh_desktop(&home, false);
    Ok(())
}

pub(crate) fn integrate_into(home: &Path, paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let exe = paths.exe.to_string_lossy();
    let apps = home.join(".local/share/applications");
    let icons = home.join(".local/share/icons/hicolor");
    let kde6 = home.join(".local/share/kio/servicemenus");
    let kde5 = home.join(".local/share/kservices5/ServiceMenus");
    let nemo = home.join(".local/share/nemo/actions");
    let thunar = home.join(".config/Thunar");
    let nautilus_py = home.join(".local/share/nautilus-python/extensions");
    let nautilus_scripts_dir = home.join(".local/share/nautilus/scripts/ShellZip");
    let caja_scripts_dir = home.join(".local/share/caja/scripts/ShellZip");
    let pcmanfm = home.join(".local/share/file-manager/actions");

    for dir in [
        &apps,
        &icons,
        &kde6,
        &kde5,
        &nemo,
        &thunar,
        &nautilus_py,
        &nautilus_scripts_dir,
        &caja_scripts_dir,
        &pcmanfm,
    ] {
        fs::create_dir_all(dir)?;
    }

    fs::write(apps.join("shellzip.desktop"), desktop_entry(&exe))?;
    install_hicolor(&paths.icons_source, &icons)?;
    if let Some(png) = &paths.app_icon_png {
        write_app_icon_png(&icons, png)?;
    }

    for (name, body) in kde_servicemenus(&exe) {
        fs::write(kde6.join(&name), &body)?;
        fs::write(kde5.join(&name), &body)?;
    }
    let _ = fs::remove_file(kde6.join("shellzip.desktop"));
    let _ = fs::remove_file(kde5.join("shellzip.desktop"));

    for (name, body) in nemo_actions(&exe) {
        fs::write(nemo.join(name), body)?;
    }

    merge_thunar_config(&thunar, &exe)?;
    let _ = fs::remove_file(thunar.join("shellzip-uca.xml"));

    fs::write(
        nautilus_py.join("shellzip.py"),
        nautilus_python_extension(&exe),
    )?;
    write_script_dir(&nautilus_scripts_dir, &exe)?;
    write_script_dir(&caja_scripts_dir, &exe)?;

    for (name, body) in pcmanfm_actions(&exe) {
        fs::write(pcmanfm.join(name), body)?;
    }
    Ok(())
}

pub(crate) fn unintegrate_into(home: &Path, paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let _ = fs::remove_file(home.join(".local/share/applications/shellzip.desktop"));
    for dir in [
        home.join(".local/share/kio/servicemenus"),
        home.join(".local/share/kservices5/ServiceMenus"),
    ] {
        for name in [
            "shellzip.desktop",
            "shellzip-extract.desktop",
            "shellzip-add.desktop",
        ] {
            let _ = fs::remove_file(dir.join(name));
        }
    }

    let nemo = home.join(".local/share/nemo/actions");
    for (name, _) in nemo_actions("shellzip") {
        let _ = fs::remove_file(nemo.join(name));
    }

    let thunar = home.join(".config/Thunar");
    let _ = fs::remove_file(thunar.join("shellzip-uca.xml"));
    unmerge_thunar_config(&thunar)?;

    let _ = fs::remove_file(home.join(".local/share/nautilus-python/extensions/shellzip.py"));
    let _ = fs::remove_dir_all(home.join(".local/share/nautilus/scripts/ShellZip"));
    let _ = fs::remove_dir_all(home.join(".local/share/caja/scripts/ShellZip"));

    let pcmanfm = home.join(".local/share/file-manager/actions");
    for (name, _) in pcmanfm_actions("shellzip") {
        let _ = fs::remove_file(pcmanfm.join(name));
    }

    let hicolor = home.join(".local/share/icons/hicolor");
    remove_shellzip_mime_icons(&hicolor)?;
    remove_shellzip_app_icons(&hicolor)?;
    let _ = paths;
    Ok(())
}

fn refresh_desktop(home: &Path, set_mime_defaults: bool) {
    let apps = home.join(".local/share/applications");
    let icons = home.join(".local/share/icons/hicolor");
    let _ = Command::new("update-desktop-database").arg(&apps).status();
    let _ = Command::new("gtk-update-icon-cache")
        .args(["-f", "-t", &icons.to_string_lossy()])
        .status();
    if set_mime_defaults {
        for mime in MIME_TYPES {
            let _ = Command::new("xdg-mime")
                .args(["default", "shellzip.desktop", mime])
                .status();
        }
    }
    let _ = Command::new("nautilus").arg("-q").status();
}

fn home_dir() -> Result<PathBuf, IntegrateError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| IntegrateError::Message("HOME is not set".into()))
}

fn write_script_dir(dir: &Path, exe: &str) -> Result<(), IntegrateError> {
    fs::create_dir_all(dir)?;
    for (name, body) in nautilus_scripts(exe) {
        let path = dir.join(name);
        fs::write(&path, body)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        }
    }
    Ok(())
}

fn merge_thunar_config(thunar: &Path, exe: &str) -> Result<(), IntegrateError> {
    let uca = thunar.join("uca.xml");
    let existing = if uca.is_file() {
        fs::read_to_string(&uca)?
    } else {
        String::new()
    };
    fs::write(uca, merge_thunar_uca(&existing, exe))?;
    Ok(())
}

fn unmerge_thunar_config(thunar: &Path) -> Result<(), IntegrateError> {
    let uca = thunar.join("uca.xml");
    if uca.is_file() {
        let existing = fs::read_to_string(&uca)?;
        fs::write(uca, strip_thunar_shellzip_actions(&existing))?;
    }
    Ok(())
}

fn write_app_icon_png(hicolor: &Path, png: &[u8]) -> Result<(), IntegrateError> {
    let dir = hicolor.join("256x256").join("apps");
    fs::create_dir_all(&dir)?;
    fs::write(dir.join(format!("{APP_ICON_NAME}.png")), png)?;
    Ok(())
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

fn remove_shellzip_app_icons(hicolor: &Path) -> Result<(), IntegrateError> {
    for size in HICOLOR_SIZES {
        let apps = hicolor.join(size).join("apps");
        let ext = if *size == "scalable" { "svg" } else { "png" };
        let _ = fs::remove_file(apps.join(format!("{APP_ICON_NAME}.{ext}")));
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

fn hint_nautilus_python() {
    let ok = Command::new("python3")
        .args(["-c", "from gi.repository import Nautilus"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if !ok {
        eprintln!(
            "shellzip: GNOME Files shows a top-level ShellZip submenu after installing python3-nautilus (or nautilus-python) and re-running --integrate; Scripts → ShellZip is available without that package"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_paths(home: &Path, png: Option<Vec<u8>>) -> IntegratePaths {
        let exe = home.join("bin").join("shellzip");
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"").unwrap();
        IntegratePaths {
            exe,
            icons_source: home.join("missing-icons"),
            shell_dll: None,
            app_icon_png: png,
        }
    }

    #[test]
    fn writes_nautilus_kde_thunar_and_app_icon() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let paths = test_paths(home, Some(b"fake-png".to_vec()));
        let exe = paths.exe.to_string_lossy().into_owned();
        integrate_into(home, &paths).unwrap();

        let py =
            fs::read_to_string(home.join(".local/share/nautilus-python/extensions/shellzip.py"))
                .unwrap();
        assert!(py.contains(&exe), "{py}");
        assert!(py.contains("gi.require_version"));

        let script = home
            .join(".local/share/nautilus/scripts/ShellZip")
            .join("Extract Here");
        assert!(script.is_file());
        let mode = fs::metadata(&script).unwrap().permissions();
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(mode.mode() & 0o111, 0o111);

        assert!(
            home.join(".local/share/kio/servicemenus/shellzip-extract.desktop")
                .is_file()
        );
        assert!(
            home.join(".local/share/kservices5/ServiceMenus/shellzip-add.desktop")
                .is_file()
        );
        assert!(
            home.join(".local/share/nemo/actions/shellzip-add.nemo_action")
                .is_file()
        );
        assert!(
            home.join(".local/share/file-manager/actions/shellzip-add.desktop")
                .is_file()
        );

        let uca = fs::read_to_string(home.join(".config/Thunar/uca.xml")).unwrap();
        assert!(uca.contains("<unique-id>shellzip-open</unique-id>"));
        assert!(!home.join(".config/Thunar/shellzip-uca.xml").exists());

        let icon = home
            .join(".local/share/icons/hicolor/256x256/apps")
            .join("shellzip.png");
        assert_eq!(fs::read(&icon).unwrap(), b"fake-png");

        integrate_into(home, &paths).unwrap();
        let uca2 = fs::read_to_string(home.join(".config/Thunar/uca.xml")).unwrap();
        assert_eq!(
            uca2.matches("<unique-id>shellzip-open</unique-id>").count(),
            1
        );

        unintegrate_into(home, &paths).unwrap();
        assert!(
            !home
                .join(".local/share/nautilus-python/extensions/shellzip.py")
                .exists()
        );
        assert!(!home.join(".local/share/nautilus/scripts/ShellZip").exists());
        assert!(!icon.exists());
        let uca3 = fs::read_to_string(home.join(".config/Thunar/uca.xml")).unwrap();
        assert!(!uca3.contains("shellzip-open"));
    }

    #[test]
    fn thunar_merge_keeps_other_actions() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let thunar = home.join(".config/Thunar");
        fs::create_dir_all(&thunar).unwrap();
        fs::write(
            thunar.join("uca.xml"),
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<actions>\n  <action>\n    <name>Open Terminal Here</name>\n    <unique-id>term-1</unique-id>\n  </action>\n</actions>\n",
        )
        .unwrap();
        let paths = test_paths(home, None);
        integrate_into(home, &paths).unwrap();
        let uca = fs::read_to_string(thunar.join("uca.xml")).unwrap();
        assert!(uca.contains("Open Terminal Here"));
        assert!(uca.contains("shellzip-add"));
        unintegrate_into(home, &paths).unwrap();
        let uca = fs::read_to_string(thunar.join("uca.xml")).unwrap();
        assert!(uca.contains("Open Terminal Here"));
        assert!(!uca.contains("shellzip-add"));
    }
}
