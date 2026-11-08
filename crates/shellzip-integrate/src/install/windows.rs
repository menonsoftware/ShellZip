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
/// Where integrate stores the GUI path for the shell extension to launch.
const HOST_EXE_KEY: &str = r"Software\ShellZip";
const HOST_EXE_VALUE: &str = "ExePath";
/// Packaged app user model id (`FamilyName!AppId`) when `ExePath` is under WindowsApps.
const HOST_AUMID_VALUE: &str = "Aumid";
/// Application Id from the Store MSIX manifest.
const PACKAGE_APP_ID: &str = "ShellZip";
/// Files, filesystem folders, and the Folder class (drives and namespace folders).
const MENU_HANDLER_KEYS: &[&str] = &[
    r"*\shellex\ContextMenuHandlers\ShellZip",
    r"Directory\shellex\ContextMenuHandlers\ShellZip",
    r"Folder\shellex\ContextMenuHandlers\ShellZip",
];

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

        // Drop legacy per-extension handlers so archives are not double-listed.
        let legacy =
            format!("SystemFileAssociations\\{ext_key}\\shellex\\ContextMenuHandlers\\ShellZip");
        let _ = classes.delete_subkey_all(&legacy);

        // Compound archives end in .gz / .bz2 / .xz / .zst; IconHandler picks tar.*.ico.
        if matches!(kind.extension, "gz" | "bz2" | "xz" | "zst") {
            let icon_handler = format!("{prog}\\shellex\\IconHandler", prog = kind.prog_id);
            classes
                .create_subkey(&icon_handler)?
                .0
                .set_value("", &SHELL_CLSID)?;
        }
    }

    // The DLL is copied under LocalAppData; Explorer's PATH rarely includes the
    // GUI, so store an absolute ExePath for menu verbs to launch. Write this
    // before the DLL copy so a locked Explorer file cannot skip it.
    write_host_exe(&exe_str)?;
    // Explorer loads InprocServer32 from HKLM\Software\Classes\CLSID. A CLSID
    // that exists only under HKCU is ignored, so the menu never appears.
    register_shell_menu(&classes, paths.shell_dll.as_deref())?;

    let _plan = windows_registry_plan(&exe_str, &icons_dir.to_string_lossy());
    let _ = notify_assoc_changed();
    Ok(())
}

/// Remove ProgIDs, OpenWith entries, icons, and the shell-extension CLSID.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = match hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        Ok(key) => key,
        Err(_) => {
            remove_machine_registration();
            if let Ok(dll) = shell_dll_dest() {
                let _ = fs::remove_file(dll);
            }
            return Ok(());
        }
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

    for key in MENU_HANDLER_KEYS {
        let _ = classes.delete_subkey_all(key);
    }

    let _ = hkcu.delete_subkey_all(format!(r"Software\Classes\CLSID\{SHELL_CLSID}"));
    if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED, KEY_ALL_ACCESS) {
        let _ = approved.delete_value(SHELL_CLSID);
    }
    remove_machine_registration();
    let _ = hkcu.delete_subkey_all(HOST_EXE_KEY);
    if let Ok(dll) = shell_dll_dest() {
        let _ = fs::remove_file(dll);
    }

    if let Ok(dir) = icons_install_dir() {
        let _ = fs::remove_dir_all(dir);
    }
    let _ = paths;
    let _ = notify_assoc_changed();
    Ok(())
}

/// Absolute `shellzip.exe` written by the last successful `--integrate`.
pub(crate) fn registered_host_exe() -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(HOST_EXE_KEY).ok()?;
    let path: String = key.get_value(HOST_EXE_VALUE).ok()?;
    let path = PathBuf::from(path);
    path.is_file().then_some(path)
}

fn write_host_exe(exe: &str) -> Result<(), IntegrateError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.create_subkey(HOST_EXE_KEY)?.0;
    key.set_value(HOST_EXE_VALUE, &exe)?;
    if let Some(aumid) = aumid_from_windowsapps_path(Path::new(exe)) {
        key.set_value(HOST_AUMID_VALUE, &aumid)?;
    } else {
        let _ = key.delete_value(HOST_AUMID_VALUE);
    }
    Ok(())
}

/// Absolute packaged AUMID stored by the last Windows `--integrate`, if any.
pub(crate) fn registered_aumid() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey(HOST_EXE_KEY).ok()?;
    let value: String = key.get_value(HOST_AUMID_VALUE).ok()?;
    (!value.is_empty()).then_some(value)
}

/// Derive `Name_PublisherHash!ShellZip` from a WindowsApps install path.
pub(crate) fn aumid_from_windowsapps_path(exe: &Path) -> Option<String> {
    let parent = exe.parent()?;
    let under_apps = parent.components().any(|c| c.as_os_str() == "WindowsApps");
    if !under_apps {
        return None;
    }
    let folder = parent.file_name()?.to_str()?;
    let (left, publisher) = folder.rsplit_once('_')?;
    let name = left.split('_').next()?;
    if name.is_empty() || publisher.is_empty() {
        return None;
    }
    Some(format!("{name}_{publisher}!{PACKAGE_APP_ID}"))
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

/// Copy the shell DLL to a stable path and register it for Explorer.
///
/// Handler keys under HKCU are visible in `HKCR`. The COM server is not:
/// 64-bit Explorer resolves `InprocServer32` from
/// `HKLM\Software\Classes\CLSID` and skips a CLSID that lives only in HKCU.
fn register_shell_menu(hkcu_classes: &RegKey, dll: Option<&Path>) -> Result<(), IntegrateError> {
    let Some(dll) = dll.filter(|path| path.is_file()) else {
        return Err(IntegrateError::Message(
            "shellzip_shell.dll was not found next to shellzip.exe, so the Explorer menu was not installed. Build the shell extension and integrate again.".into(),
        ));
    };
    let installed = install_shell_dll(dll)?;
    let dll_value = registry_fs_path(&installed);
    write_com_server(RegistryHive::User, &dll_value)?;
    write_com_server(RegistryHive::Machine, &dll_value).map_err(|err| {
        IntegrateError::Message(format!(
            "could not register the Explorer menu for this machine ({err}). Run ShellZip as administrator and integrate again"
        ))
    })?;
    for key in MENU_HANDLER_KEYS {
        hkcu_classes
            .create_subkey(key)?
            .0
            .set_value("", &SHELL_CLSID)?;
    }
    let machine_classes = RegKey::predef(HKEY_LOCAL_MACHINE)
        .create_subkey("Software\\Classes")
        .map_err(|err| {
            IntegrateError::Message(format!(
                "could not register the Explorer menu for this machine ({err}). Run ShellZip as administrator and integrate again"
            ))
        })?
        .0;
    for key in MENU_HANDLER_KEYS {
        machine_classes
            .create_subkey(key)?
            .0
            .set_value("", &SHELL_CLSID)?;
    }
    Ok(())
}

enum RegistryHive {
    User,
    Machine,
}

fn write_com_server(hive: RegistryHive, dll: &str) -> Result<(), IntegrateError> {
    let root = RegKey::predef(match hive {
        RegistryHive::User => HKEY_CURRENT_USER,
        RegistryHive::Machine => HKEY_LOCAL_MACHINE,
    });
    let clsid = root
        .create_subkey(format!(r"Software\Classes\CLSID\{SHELL_CLSID}"))?
        .0;
    clsid.set_value("", &"ShellZip Context Menu")?;
    let inproc = clsid.create_subkey("InprocServer32")?.0;
    inproc.set_value("", &dll)?;
    inproc.set_value("ThreadingModel", &"Apartment")?;
    if matches!(hive, RegistryHive::Machine) {
        root.create_subkey(APPROVED)?
            .0
            .set_value(SHELL_CLSID, &"ShellZip Context Menu")?;
    }
    Ok(())
}

fn remove_machine_registration() {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(classes) = hklm.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        for key in MENU_HANDLER_KEYS {
            let _ = classes.delete_subkey_all(key);
        }
    }
    let _ = hklm.delete_subkey_all(format!(r"Software\Classes\CLSID\{SHELL_CLSID}"));
    if let Ok(approved) = hklm.open_subkey_with_flags(APPROVED, KEY_ALL_ACCESS) {
        let _ = approved.delete_value(SHELL_CLSID);
    }
}

fn shell_dll_dest() -> Result<PathBuf, IntegrateError> {
    let icons = icons_install_dir()?;
    let parent = icons
        .parent()
        .ok_or_else(|| IntegrateError::Message("LOCALAPPDATA is not set".into()))?;
    Ok(parent.join("shellzip_shell.dll"))
}

fn install_shell_dll(source: &Path) -> Result<PathBuf, IntegrateError> {
    let dest = shell_dll_dest()?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let same = source.canonicalize().ok() == dest.canonicalize().ok() && dest.is_file();
    if same {
        return Ok(dest);
    }
    match fs::copy(source, &dest) {
        Ok(_) => Ok(dest),
        Err(err) if err.raw_os_error() == Some(32) => Err(IntegrateError::Message(
            "could not update shellzip_shell.dll because Explorer still has it loaded. Restart Explorer (or sign out) and run shellzip --integrate again".into(),
        )),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_handlers_cover_files_and_folders() {
        let joined = MENU_HANDLER_KEYS.join("\n");
        assert!(joined.contains(r"*\shellex\ContextMenuHandlers\ShellZip"));
        assert!(joined.contains(r"Directory\shellex\ContextMenuHandlers\ShellZip"));
        assert!(joined.contains(r"Folder\shellex\ContextMenuHandlers\ShellZip"));
    }

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

    #[test]
    fn aumid_from_store_install_path() {
        let exe = Path::new(
            r"C:\Program Files\WindowsApps\ShellZip_0.1.12.0_x64__g9p8warn4kjs0\shellzip.exe",
        );
        assert_eq!(
            aumid_from_windowsapps_path(exe).as_deref(),
            Some("ShellZip_g9p8warn4kjs0!ShellZip")
        );
        assert!(aumid_from_windowsapps_path(Path::new(r"C:\Tools\shellzip.exe")).is_none());
    }
}
