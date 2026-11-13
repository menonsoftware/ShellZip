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
    // The .deb already ships the Nautilus extension, and a working
    // python3-nautilus install shows that submenu directly. Drop the per-user
    // copy and the Scripts fallback so Files does not list ShellZip twice.
    finish_nautilus_menu(
        &home,
        packaged_nautilus_extension_installed(),
        nautilus_python_importable(),
    );
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

    remove_nautilus_python_extension(home);
    remove_nautilus_scripts(home);
    let _ = fs::remove_dir_all(home.join(".local/share/caja/scripts/ShellZip"));

    let pcmanfm = home.join(".local/share/file-manager/actions");
    for (name, _) in pcmanfm_actions("shellzip") {
        let _ = fs::remove_file(pcmanfm.join(name));
    }

    let hicolor = home.join(".local/share/icons/hicolor");
    remove_shellzip_mime_icons(&hicolor)?;
    remove_shellzip_app_icons(&hicolor)?;
    clear_shellzip_mimeapps(home)?;
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

const SHELLZIP_DESKTOP_ID: &str = "shellzip.desktop";

/// Drop `shellzip.desktop` from a mimeapps list. A line whose only handler is
/// ShellZip is removed. Other handlers on the same line stay.
fn strip_shellzip_mimeapps(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let newline = if line.ends_with('\n') { "\n" } else { "" };
        let body = line.trim_end_matches(['\n', '\r']);
        if let Some(kept) = strip_shellzip_mimeapps_line(body) {
            out.push_str(&kept);
            out.push_str(newline);
        }
    }
    out
}

fn strip_shellzip_mimeapps_line(line: &str) -> Option<String> {
    if !line.contains(SHELLZIP_DESKTOP_ID) {
        return Some(line.to_string());
    }
    let Some((key, value)) = line.split_once('=') else {
        return Some(line.to_string());
    };
    let trailing = value.ends_with(';');
    let kept: Vec<&str> = value
        .split(';')
        .filter(|part| !part.is_empty() && *part != SHELLZIP_DESKTOP_ID)
        .collect();
    if kept.is_empty() {
        return None;
    }
    let mut joined = kept.join(";");
    if trailing {
        joined.push(';');
    }
    Some(format!("{key}={joined}"))
}

fn clear_shellzip_mimeapps(home: &Path) -> Result<(), IntegrateError> {
    for relative in [
        ".config/mimeapps.list",
        ".local/share/applications/mimeapps.list",
    ] {
        let path = home.join(relative);
        if !path.is_file() {
            continue;
        }
        let original = fs::read_to_string(&path)?;
        let cleaned = strip_shellzip_mimeapps(&original);
        if cleaned != original {
            fs::write(&path, cleaned)?;
        }
    }
    Ok(())
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

fn nautilus_extension_dir(home: &Path) -> PathBuf {
    home.join(".local/share/nautilus-python/extensions")
}

fn nautilus_scripts_dir(home: &Path) -> PathBuf {
    home.join(".local/share/nautilus/scripts/ShellZip")
}

/// Remove `shellzip.py` and any bytecode left after the source file is gone.
fn remove_nautilus_python_extension(home: &Path) {
    let ext = nautilus_extension_dir(home);
    let _ = fs::remove_file(ext.join("shellzip.py"));
    let cache = ext.join("__pycache__");
    if let Ok(entries) = fs::read_dir(&cache) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            if name.starts_with("shellzip.") && name.ends_with(".pyc") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    let _ = fs::remove_dir(&cache);
}

fn remove_nautilus_scripts(home: &Path) {
    let _ = fs::remove_dir_all(nautilus_scripts_dir(home));
}

/// Keep a single Nautilus menu.
///
/// `packaged_extension` means Files already loads `/usr/share/.../shellzip.py`,
/// so the home copy (and its `__pycache__`) would add a second submenu.
/// `python_extension` means the top-level submenu can load, so the Scripts
/// folder is only a duplicate.
fn finish_nautilus_menu(home: &Path, packaged_extension: bool, python_extension: bool) {
    if packaged_extension {
        remove_nautilus_python_extension(home);
    }
    if packaged_extension || python_extension {
        remove_nautilus_scripts(home);
    }
}

fn packaged_nautilus_extension_installed() -> bool {
    [
        "/usr/share/nautilus-python/extensions/shellzip.py",
        "/usr/local/share/nautilus-python/extensions/shellzip.py",
    ]
    .into_iter()
    .any(|path| Path::new(path).is_file())
}

fn nautilus_python_importable() -> bool {
    Command::new("python3")
        .args(["-c", "from gi.repository import Nautilus"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn hint_nautilus_python() {
    if !nautilus_python_importable() {
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

        let cache = home.join(".local/share/nautilus-python/extensions/__pycache__");
        fs::create_dir_all(&cache).unwrap();
        let pyc = cache.join("shellzip.cpython-314.pyc");
        fs::write(&pyc, b"stale").unwrap();

        unintegrate_into(home, &paths).unwrap();
        assert!(
            !home
                .join(".local/share/nautilus-python/extensions/shellzip.py")
                .exists()
        );
        assert!(!pyc.exists());
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

    #[test]
    fn packaged_nautilus_extension_drops_the_user_copy_and_scripts() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let paths = test_paths(home, None);
        integrate_into(home, &paths).unwrap();
        let cache = home.join(".local/share/nautilus-python/extensions/__pycache__");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("shellzip.cpython-314.pyc"), b"stale").unwrap();

        finish_nautilus_menu(home, true, true);

        assert!(
            !home
                .join(".local/share/nautilus-python/extensions/shellzip.py")
                .exists()
        );
        assert!(!cache.join("shellzip.cpython-314.pyc").exists());
        assert!(!home.join(".local/share/nautilus/scripts/ShellZip").exists());
    }

    #[test]
    fn python_nautilus_keeps_the_extension_and_drops_scripts() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let paths = test_paths(home, None);
        integrate_into(home, &paths).unwrap();

        finish_nautilus_menu(home, false, true);

        assert!(
            home.join(".local/share/nautilus-python/extensions/shellzip.py")
                .is_file()
        );
        assert!(!home.join(".local/share/nautilus/scripts/ShellZip").exists());
    }

    #[test]
    fn scripts_stay_when_nautilus_python_is_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let paths = test_paths(home, None);
        integrate_into(home, &paths).unwrap();

        finish_nautilus_menu(home, false, false);

        assert!(
            home.join(".local/share/nautilus-python/extensions/shellzip.py")
                .is_file()
        );
        assert!(
            home.join(".local/share/nautilus/scripts/ShellZip/Extract Here")
                .is_file()
        );
    }

    #[test]
    fn unintegrate_drops_shellzip_from_mimeapps_and_keeps_other_apps() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path();
        let config = home.join(".config");
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("mimeapps.list"),
            "\
[Default Applications]
application/zip=shellzip.desktop
application/x-7z-compressed=org.gnome.FileRoller.desktop;shellzip.desktop;
text/html=google-chrome.desktop

[Added Associations]
application/zip=shellzip.desktop;
application/gzip=org.gnome.FileRoller.desktop;shellzip.desktop;
",
        )
        .unwrap();
        let paths = test_paths(home, None);
        unintegrate_into(home, &paths).unwrap();
        let text = fs::read_to_string(config.join("mimeapps.list")).unwrap();
        assert!(!text.contains("shellzip.desktop"), "{text}");
        assert!(text.contains("application/x-7z-compressed=org.gnome.FileRoller.desktop;"));
        assert!(text.contains("text/html=google-chrome.desktop"));
        assert!(text.contains("application/gzip=org.gnome.FileRoller.desktop;"));
        assert!(
            !text
                .lines()
                .any(|line| line.starts_with("application/zip="))
        );
    }
}
