//! Per-user Windows ProgIDs, icons, and shell-extension registration.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use winreg::RegKey;
use winreg::enums::*;

use super::{IntegrateError, IntegratePaths};
use crate::formats::EXTENSIONS;
use crate::writers::windows_registry_plan;

const SHELL_CLSID: &str = "{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Shell Extensions\Approved";

/// Install associations and copy format icons into LocalAppData.
pub fn integrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let exe_str = registry_fs_path(&paths.exe);
    let icons_dir = icons_install_dir()?;
    fs::create_dir_all(&icons_dir)?;
    copy_icons(&paths.icons_source, &icons_dir)?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu.create_subkey("Software\\Classes")?.0;

    for kind in EXTENSIONS {
        let icon_name = format!("{}.ico", kind.icon);
        let icon_path = icons_dir.join(&icon_name);
        // `,0` selects the first image; Explorer still picks the matching
        // BMP/PNG size from the multi-resolution .ico for each view mode.
        let icon_value = if icon_path.is_file() {
            format!("{},0", icon_path.to_string_lossy())
        } else {
            format!("{exe_str},0")
        };
        let prog = classes.create_subkey(kind.prog_id)?.0;
        prog.set_value("", &kind.type_name)?;
        prog.create_subkey("DefaultIcon")?
            .0
            .set_value("", &icon_value)?;
        let command = format!("\"{exe_str}\" \"%1\"");
        prog.create_subkey("shell\\open\\command")?
            .0
            .set_value("", &command)?;

        let ext_key = format!(".{}", kind.extension);
        let open_with = classes
            .create_subkey(format!("{ext_key}\\OpenWithProgids"))?
            .0;
        open_with.set_value(kind.prog_id, &"")?;
        // Per-user default when Windows still honours HKCU Classes\<ext>.
        let _ = classes
            .create_subkey(&ext_key)?
            .0
            .set_value("", &kind.prog_id);

        let assoc =
            format!("SystemFileAssociations\\{ext_key}\\shellex\\ContextMenuHandlers\\ShellZip");
        classes
            .create_subkey(&assoc)?
            .0
            .set_value("", &SHELL_CLSID)?;

        // Compound archives end in .gz / .bz2 / .xz / .zst; IconHandler picks tar.*.ico.
        if matches!(kind.extension, "gz" | "bz2" | "xz" | "zst") {
            let icon_handler = format!("{prog}\\shellex\\IconHandler", prog = kind.prog_id);
            classes
                .create_subkey(&icon_handler)?
                .0
                .set_value("", &SHELL_CLSID)?;
        }
    }

    if let Some(dll) = &paths.shell_dll
        && dll.is_file()
    {
        let dll = registry_fs_path(dll);
        let clsid_path = format!(r"Software\Classes\CLSID\{SHELL_CLSID}");
        let clsid = hkcu.create_subkey(&clsid_path)?.0;
        clsid.set_value("", &"ShellZip Context Menu")?;
        let inproc = clsid.create_subkey("InprocServer32")?.0;
        inproc.set_value("", &dll)?;
        inproc.set_value("ThreadingModel", &"Apartment")?;
        hkcu.create_subkey(APPROVED)?
            .0
            .set_value(SHELL_CLSID, &"ShellZip Context Menu")?;
    }

    let _plan = windows_registry_plan(&exe_str, &icons_dir.to_string_lossy());
    let _ = notify_assoc_changed();
    Ok(())
}

/// Remove ProgIDs, OpenWith entries, icons, and the shell-extension CLSID.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = match hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        Ok(key) => key,
        Err(_) => return Ok(()),
    };

    for kind in EXTENSIONS {
        let _ = classes.delete_subkey_all(kind.prog_id);
        let ext_key = format!(".{}", kind.extension);
        if let Ok(open_with) =
            classes.open_subkey_with_flags(format!("{ext_key}\\OpenWithProgids"), KEY_ALL_ACCESS)
        {
            let _ = open_with.delete_value(kind.prog_id);
        }
        let assoc = format!(
            "SystemFileAssociations\\.{ext}\\shellex\\ContextMenuHandlers\\ShellZip",
            ext = kind.extension
        );
        let _ = classes.delete_subkey_all(&assoc);
        if let Ok(ext) = classes.open_subkey_with_flags(&ext_key, KEY_ALL_ACCESS)
            && let Ok(current) = ext.get_value::<String, _>("")
            && current == kind.prog_id
        {
            let _ = ext.delete_value("");
        }
    }

    let _ = hkcu.delete_subkey_all(format!(r"Software\Classes\CLSID\{SHELL_CLSID}"));
    if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED, KEY_ALL_ACCESS) {
        let _ = approved.delete_value(SHELL_CLSID);
    }

    if let Ok(dir) = icons_install_dir() {
        let _ = fs::remove_dir_all(dir);
    }
    let _ = paths;
    let _ = notify_assoc_changed();
    Ok(())
}

fn icons_install_dir() -> Result<PathBuf, IntegrateError> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| IntegrateError::Message("LOCALAPPDATA is not set".into()))?;
    Ok(base.join("ShellZip").join("icons"))
}

/// Absolute path suitable for ProgID / InprocServer32 values.
///
/// `Path::canonicalize` on Windows yields a verbatim `\\?\…` form that
/// Explorer's `shell\open\command` and `ShellExecute` reject with
/// "Windows cannot access the specified device, path, or file."
fn registry_fs_path(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    strip_verbatim_prefix(&path).to_string_lossy().into_owned()
}

fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path.to_path_buf()
    }
}

fn copy_icons(source: &Path, dest: &Path) -> Result<(), IntegrateError> {
    let ico_dir = source.join("ico");
    if ico_dir.is_dir() {
        for entry in fs::read_dir(&ico_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ico") {
                let name = entry.file_name();
                fs::copy(&path, dest.join(name))?;
            }
        }
        return Ok(());
    }
    // Fall back to embedding nothing; ProgIDs will use shellzip.exe,0.
    Ok(())
}

fn notify_assoc_changed() -> Result<(), IntegrateError> {
    // Refresh Explorer's icon cache without `unsafe` FFI. `ie4uinit -show`
    // is the supported user-mode equivalent of SHChangeNotify(ASSOCCHANGED).
    let _ = Command::new("ie4uinit.exe").arg("-show").status();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_dir_under_local_app_data() {
        if std::env::var_os("LOCALAPPDATA").is_some() {
            let dir = icons_install_dir().expect("dir");
            assert!(dir.ends_with(Path::new("ShellZip\\icons")) || dir.ends_with("ShellZip/icons"));
        }
    }

    #[test]
    fn strips_verbatim_drive_and_unc_prefixes() {
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\C:\ShellZip\shellzip.exe")),
            PathBuf::from(r"C:\ShellZip\shellzip.exe")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"\\?\UNC\server\share\shellzip.exe")),
            PathBuf::from(r"\\server\share\shellzip.exe")
        );
        assert_eq!(
            strip_verbatim_prefix(Path::new(r"C:\ShellZip\shellzip.exe")),
            PathBuf::from(r"C:\ShellZip\shellzip.exe")
        );
    }

    #[test]
    fn registry_path_drops_canonicalize_prefix() {
        let exe = std::env::current_exe().expect("current_exe");
        let registered = registry_fs_path(&exe);
        assert!(
            !registered.starts_with(r"\\?\"),
            "Explorer rejects verbatim paths in open commands: {registered}"
        );
        assert!(
            registered.contains("shellzip") || registered.contains("deps"),
            "unexpected path: {registered}"
        );
    }
}
