//! Per-user Windows ProgIDs, icons, and shell-extension registration.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use winreg::RegKey;
use winreg::enums::*;

use super::{IntegrateError, IntegratePaths};
use crate::formats::EXTENSIONS;
use crate::writers::windows_registry_plan;

/// Classic in-proc `IContextMenu` CLSID (must match `shellzip-shell`).
pub const SHELL_CLSID: &str = "{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Shell Extensions\Approved";
/// Where integrate stores the GUI path for the shell extension to launch.
const HOST_EXE_KEY: &str = r"Software\ShellZip";
const HOST_EXE_VALUE: &str = "ExePath";
/// Packaged app user model id (`FamilyName!AppId`) when `ExePath` is under WindowsApps.
const HOST_AUMID_VALUE: &str = "Aumid";
/// Application Id from the Store MSIX manifest.
const PACKAGE_APP_ID: &str = "ShellZip";
/// Scheduled task that cleans classic keys after Settings removes the MSIX.
const CLEANUP_TASK_NAME: &str = "ShellZipMsixShellCleanup";
const CLEANUP_SCRIPT_NAME: &str = "msix-shell-cleanup.ps1";
/// AppX deployment log channel on Windows 10 1809+.
const APPX_EVENT_LOG: &str = "Microsoft-Windows-AppXDeploymentServer/Operational";
/// `DeploymentOperation` value for Remove (see event 400 / 603).
const APPX_REMOVE_OPERATION: &str = "2";
/// Event fired when an AppX Remove finishes successfully.
const APPX_REMOVE_FINISHED_EVENT_ID: u32 = 400;

/// Files, filesystem folders, and the Folder class (drives and namespace folders).
pub const MENU_HANDLER_KEYS: &[&str] = &[
    r"*\shellex\ContextMenuHandlers\ShellZip",
    r"Directory\shellex\ContextMenuHandlers\ShellZip",
    r"Folder\shellex\ContextMenuHandlers\ShellZip",
];

/// Compound-archive ProgIDs that register an `IconHandler`.
pub const ICON_HANDLER_EXTENSIONS: &[&str] = &["gz", "bz2", "xz", "zst"];

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
        if ICON_HANDLER_EXTENSIONS.contains(&kind.extension) {
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
///
/// # Errors
///
/// Returns when HKLM deletion is denied so the caller can elevate and retry.
pub fn unintegrate(paths: &IntegratePaths) -> Result<(), IntegrateError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = match hkcu.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        Ok(key) => Some(key),
        Err(err) if is_not_found(&err) => None,
        Err(err) => return Err(err.into()),
    };

    if let Some(classes) = classes.as_ref() {
        for kind in EXTENSIONS {
            let _ = classes.delete_subkey_all(kind.prog_id);
            let ext_key = format!(".{}", kind.extension);
            if let Ok(open_with) = classes
                .open_subkey_with_flags(format!("{ext_key}\\OpenWithProgids"), KEY_ALL_ACCESS)
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
            delete_subkey_all_ok(classes, key)?;
        }
    }

    delete_subkey_all_ok(&hkcu, &format!(r"Software\Classes\CLSID\{SHELL_CLSID}"))?;
    if let Ok(approved) = hkcu.open_subkey_with_flags(APPROVED, KEY_ALL_ACCESS) {
        let _ = approved.delete_value(SHELL_CLSID);
    }
    remove_machine_registration()?;
    let _ = hkcu.delete_subkey_all(HOST_EXE_KEY);
    if let Ok(dll) = shell_dll_dest() {
        remove_file_with_retry(&dll);
    }

    if let Ok(dir) = icons_install_dir() {
        let _ = fs::remove_dir_all(&dir);
        if let Some(parent) = dir.parent() {
            // Leave settings.toml alone; remove the folder only when empty of shell bits.
            let _ = fs::remove_dir(parent);
        }
    }
    let _ = unregister_msix_uninstall_cleanup();
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

fn remove_machine_registration() -> Result<(), IntegrateError> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey_with_flags("Software\\Classes", KEY_ALL_ACCESS) {
        Ok(classes) => {
            for key in MENU_HANDLER_KEYS {
                delete_subkey_all_ok(&classes, key)?;
            }
        }
        Err(err) if is_not_found(&err) => {}
        Err(err) if is_access_denied(&err) => {
            return Err(machine_access_denied(err));
        }
        Err(err) => return Err(err.into()),
    }
    match hklm.delete_subkey_all(format!(r"Software\Classes\CLSID\{SHELL_CLSID}")) {
        Ok(()) => {}
        Err(err) if is_not_found(&err) => {}
        Err(err) if is_access_denied(&err) => {
            return Err(machine_access_denied(err));
        }
        Err(err) => return Err(err.into()),
    }
    match hklm.open_subkey_with_flags(APPROVED, KEY_ALL_ACCESS) {
        Ok(approved) => match approved.delete_value(SHELL_CLSID) {
            Ok(()) => {}
            Err(err) if is_not_found(&err) => {}
            Err(err) if is_access_denied(&err) => {
                return Err(machine_access_denied(err));
            }
            Err(err) => return Err(err.into()),
        },
        Err(err) if is_not_found(&err) => {}
        Err(err) if is_access_denied(&err) => {
            return Err(machine_access_denied(err));
        }
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

fn delete_subkey_all_ok(root: &RegKey, path: &str) -> Result<(), IntegrateError> {
    match root.delete_subkey_all(path) {
        Ok(()) => Ok(()),
        Err(err) if is_not_found(&err) => Ok(()),
        Err(err) if is_access_denied(&err) => Err(machine_access_denied(err)),
        Err(err) => Err(err.into()),
    }
}

fn machine_access_denied(err: io::Error) -> IntegrateError {
    IntegrateError::Message(format!(
        "could not unregister the Explorer menu for this machine ({err}). Run ShellZip as administrator and unintegrate again"
    ))
}

fn is_access_denied(err: &io::Error) -> bool {
    err.raw_os_error() == Some(5)
}

fn is_not_found(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::NotFound || matches!(err.raw_os_error(), Some(2) | Some(3))
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

/// ProgIDs, menu handler paths, and IconHandler paths written by [`integrate`].
pub fn registration_key_plan() -> RegistrationKeyPlan {
    let mut prog_ids = Vec::new();
    let mut icon_handlers = Vec::new();
    for kind in EXTENSIONS {
        prog_ids.push(kind.prog_id.to_string());
        if ICON_HANDLER_EXTENSIONS.contains(&kind.extension) {
            icon_handlers.push(format!(r"{prog}\shellex\IconHandler", prog = kind.prog_id));
        }
    }
    RegistrationKeyPlan {
        menu_handlers: MENU_HANDLER_KEYS.iter().map(|s| (*s).to_string()).collect(),
        prog_ids,
        icon_handlers,
        clsid: SHELL_CLSID.to_string(),
        approved: APPROVED.to_string(),
        host_exe_key: HOST_EXE_KEY.to_string(),
    }
}

/// Snapshot of every registry path install writes so uninstall cannot drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationKeyPlan {
    /// `ContextMenuHandlers\ShellZip` under `*`, `Directory`, and `Folder`.
    pub menu_handlers: Vec<String>,
    /// Per-format ProgIDs such as `ShellZip.zip`.
    pub prog_ids: Vec<String>,
    /// ProgID-relative `shellex\IconHandler` paths for compound archives.
    pub icon_handlers: Vec<String>,
    /// Classic shell-extension CLSID string including braces.
    pub clsid: String,
    /// Shell Extensions\Approved key under Software.
    pub approved: String,
    /// `Software\ShellZip` host path key.
    pub host_exe_key: String,
}

/// Install a per-user task that clears classic registration after MSIX removal.
///
/// # Errors
///
/// Returns when `LOCALAPPDATA` is missing or `schtasks` fails.
pub fn register_msix_uninstall_cleanup(identity_name: &str) -> Result<(), IntegrateError> {
    let script_path = cleanup_script_path()?;
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&script_path, msix_cleanup_script(identity_name))?;
    let task_xml = msix_cleanup_task_xml(identity_name, &script_path)?;
    let xml_path = script_path.with_extension("task.xml");
    write_utf16_le_bom(&xml_path, &task_xml)?;
    let status = Command::new("schtasks")
        .args([
            "/Create",
            "/TN",
            CLEANUP_TASK_NAME,
            "/XML",
            &xml_path.to_string_lossy(),
            "/F",
        ])
        .status()
        .map_err(|err| {
            IntegrateError::Message(format!("could not create MSIX cleanup task ({err})"))
        })?;
    let _ = fs::remove_file(&xml_path);
    if !status.success() {
        return Err(IntegrateError::Message(format!(
            "schtasks /Create failed with {status}"
        )));
    }
    Ok(())
}

/// Delete the MSIX cleanup task and its script if they exist.
pub fn unregister_msix_uninstall_cleanup() -> Result<(), IntegrateError> {
    let _ = Command::new("schtasks")
        .args(["/Delete", "/TN", CLEANUP_TASK_NAME, "/F"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if let Ok(script) = cleanup_script_path() {
        let _ = fs::remove_file(&script);
        let _ = fs::remove_file(script.with_extension("task.xml"));
    }
    Ok(())
}

/// Explorer often keeps `shellzip_shell.dll` mapped briefly after Stop-Process.
fn remove_file_with_retry(path: &Path) {
    for attempt in 0..8 {
        match fs::remove_file(path) {
            Ok(()) => return,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return,
            Err(_) if attempt + 1 < 8 => {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Err(_) => return,
        }
    }
}

fn cleanup_script_path() -> Result<PathBuf, IntegrateError> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| IntegrateError::Message("LOCALAPPDATA is not set".into()))?;
    Ok(base.join("ShellZip").join(CLEANUP_SCRIPT_NAME))
}

/// PowerShell body that mirrors [`unintegrate`] for Settings / Remove-AppxPackage.
pub fn msix_cleanup_script(identity_name: &str) -> String {
    let plan = registration_key_plan();
    let handlers = plan
        .menu_handlers
        .iter()
        .map(|k| format!("  '{}'", ps_single_quoted(k)))
        .collect::<Vec<_>>()
        .join(",\n");
    let prog_ids = plan
        .prog_ids
        .iter()
        .map(|k| format!("  '{}'", ps_single_quoted(k)))
        .collect::<Vec<_>>()
        .join(",\n");
    let identity = ps_single_quoted(identity_name);
    let clsid = ps_single_quoted(&plan.clsid);
    let approved = ps_single_quoted(&plan.approved);
    let host = ps_single_quoted(&plan.host_exe_key);
    let task = ps_single_quoted(CLEANUP_TASK_NAME);
    format!(
        r#"param(
  [Parameter(Mandatory = $false)]
  [string]$PackageFullName = ''
)
$ErrorActionPreference = 'SilentlyContinue'
$identity = '{identity}'
if ([string]::IsNullOrWhiteSpace($PackageFullName) -or -not ($PackageFullName.StartsWith($identity + '_'))) {{
  exit 0
}}
$stillInstalled = @(Get-AppxPackage | Where-Object {{
  $_.Name -eq $identity -or $_.Name -like ($identity + '_*') -or $_.PackageFullName -like ($identity + '_*')
}})
if ($stillInstalled.Count -gt 0) {{
  exit 0
}}

$menuHandlers = @(
{handlers}
)
$progIds = @(
{prog_ids}
)
$clsid = '{clsid}'
$approved = '{approved}'
$hostKey = '{host}'

function Remove-Key([Microsoft.Win32.RegistryKey]$root, [string]$path) {{
  try {{ $root.DeleteSubKeyTree($path, $false) }} catch {{ }}
}}
function Remove-Value([Microsoft.Win32.RegistryKey]$root, [string]$path, [string]$name) {{
  try {{
    $key = $root.OpenSubKey($path, $true)
    if ($null -ne $key) {{ $key.DeleteValue($name, $false); $key.Close() }}
  }} catch {{ }}
}}

$hkcu = [Microsoft.Win32.Registry]::CurrentUser
$hklm = [Microsoft.Win32.Registry]::LocalMachine
$classes = $hkcu.OpenSubKey('Software\Classes', $true)
if ($null -ne $classes) {{
  foreach ($prog in $progIds) {{ Remove-Key $classes $prog }}
  foreach ($handler in $menuHandlers) {{ Remove-Key $classes $handler }}
  foreach ($prog in $progIds) {{
    $ext = '.' + ($prog -replace '^ShellZip\.', '')
    Remove-Value $classes ($ext + '\OpenWithProgids') $prog
    try {{
      $extKey = $classes.OpenSubKey($ext, $true)
      if ($null -ne $extKey) {{
        $current = [string]$extKey.GetValue('')
        if ($current -eq $prog) {{ $extKey.DeleteValue('', $false) }}
        $extKey.Close()
      }}
    }} catch {{ }}
  }}
  $classes.Close()
}}
Remove-Key $hkcu ("Software\Classes\CLSID\" + $clsid)
Remove-Value $hkcu $approved $clsid
Remove-Key $hkcu $hostKey

$machineClasses = $hklm.OpenSubKey('Software\Classes', $true)
if ($null -ne $machineClasses) {{
  foreach ($handler in $menuHandlers) {{ Remove-Key $machineClasses $handler }}
  $machineClasses.Close()
}}
Remove-Key $hklm ("Software\Classes\CLSID\" + $clsid)
Remove-Value $hklm $approved $clsid

$root = Join-Path $env:LOCALAPPDATA 'ShellZip'
$dll = Join-Path $root 'shellzip_shell.dll'
$icons = Join-Path $root 'icons'
$script = Join-Path $root '{script_name}'
Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue
$deadline = (Get-Date).AddSeconds(5)
while ((Get-Date) -lt $deadline) {{
  if (-not (Get-Process -Name explorer -ErrorAction SilentlyContinue)) {{ break }}
  Start-Sleep -Milliseconds 200
}}
for ($i = 0; $i -lt 8; $i++) {{
  if (-not (Test-Path -LiteralPath $dll)) {{ break }}
  Remove-Item -LiteralPath $dll -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 250
}}
Remove-Item -LiteralPath $icons -Recurse -Force -ErrorAction SilentlyContinue
Start-Process explorer.exe
Start-Process -FilePath ie4uinit.exe -ArgumentList '-show' -WindowStyle Hidden -ErrorAction SilentlyContinue
schtasks /Delete /TN '{task}' /F | Out-Null
Remove-Item -LiteralPath $script -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath ($script -replace '\.ps1$','.task.xml') -Force -ErrorAction SilentlyContinue
"#,
        identity = identity,
        handlers = handlers,
        prog_ids = prog_ids,
        clsid = clsid,
        approved = approved,
        host = host,
        task = task,
        script_name = CLEANUP_SCRIPT_NAME,
    )
}

fn msix_cleanup_task_xml(
    _identity_name: &str,
    script_path: &Path,
) -> Result<String, IntegrateError> {
    let script = xml_escape(&script_path.to_string_lossy());
    // Event Viewer XPath on 1809 rejects starts-with/contains; filter Remove
    // completion broadly and let the script match PackageFullName.
    let query = xml_escape(&format!(
        r"*[System[Provider[@Name='Microsoft-Windows-AppXDeployment-Server'] and EventID={event_id}]] and *[EventData[Data[@Name='DeploymentOperation']='{op}']]",
        event_id = APPX_REMOVE_FINISHED_EVENT_ID,
        op = APPX_REMOVE_OPERATION,
    ));
    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Removes ShellZip classic Explorer registration after the MSIX package is uninstalled.</Description>
  </RegistrationInfo>
  <Triggers>
    <EventTrigger>
      <Enabled>true</Enabled>
      <Subscription>&lt;QueryList&gt;&lt;Query Id="0" Path="{log}"&gt;&lt;Select Path="{log}"&gt;{query}&lt;/Select&gt;&lt;/Query&gt;&lt;/QueryList&gt;</Subscription>
      <ValueQueries>
        <Value name="PackageFullName">Event/EventData/Data[@Name='PackageFullName']</Value>
      </ValueQueries>
    </EventTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>true</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT10M</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>powershell.exe</Command>
      <Arguments>-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "{script}" "$(PackageFullName)"</Arguments>
    </Exec>
  </Actions>
</Task>
"#,
        log = APPX_EVENT_LOG,
        query = query,
        script = script,
    ))
}

fn ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `schtasks /Create /XML` expects Unicode (UTF-16 LE with BOM) on Windows 10.
fn write_utf16_le_bom(path: &Path, text: &str) -> Result<(), IntegrateError> {
    let mut bytes = Vec::with_capacity(2 + text.len() * 2);
    bytes.extend_from_slice(&[0xFF, 0xFE]);
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    fs::write(path, bytes)?;
    Ok(())
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
    fn registration_plan_covers_install_writes() {
        let plan = registration_key_plan();
        assert_eq!(plan.menu_handlers.len(), MENU_HANDLER_KEYS.len());
        for key in MENU_HANDLER_KEYS {
            assert!(plan.menu_handlers.iter().any(|k| k == key));
        }
        assert_eq!(plan.prog_ids.len(), EXTENSIONS.len());
        for kind in EXTENSIONS {
            assert!(plan.prog_ids.iter().any(|p| p == kind.prog_id));
        }
        for ext in ICON_HANDLER_EXTENSIONS {
            let expected = format!("ShellZip.{ext}\\shellex\\IconHandler");
            assert!(
                plan.icon_handlers.iter().any(|p| p == &expected),
                "missing icon handler {expected}"
            );
        }
        assert_eq!(plan.clsid, SHELL_CLSID);
        assert_eq!(plan.approved, APPROVED);
        assert_eq!(plan.host_exe_key, HOST_EXE_KEY);
    }

    #[test]
    fn cleanup_script_lists_shared_keys() {
        let script = msix_cleanup_script("ShellZip");
        assert!(script.contains(r"*\shellex\ContextMenuHandlers\ShellZip"));
        assert!(script.contains("ShellZip.zip"));
        assert!(script.contains(SHELL_CLSID));
        assert!(script.contains("Get-AppxPackage"));
        assert!(script.contains(CLEANUP_TASK_NAME));
    }

    #[test]
    fn cleanup_task_xml_targets_appx_remove_finished() {
        let xml =
            msix_cleanup_task_xml("ShellZip", Path::new(r"C:\Temp\cleanup.ps1")).expect("task xml");
        assert!(xml.contains(&APPX_REMOVE_FINISHED_EVENT_ID.to_string()));
        assert!(xml.contains(APPX_EVENT_LOG));
        assert!(xml.contains("DeploymentOperation"));
        assert!(xml.contains("PackageFullName"));
        assert!(xml.contains(r#"$(PackageFullName)"#));
        assert!(xml.contains(r"C:\Temp\cleanup.ps1"));
        assert!(xml.contains("HighestAvailable"));
    }

    #[test]
    fn cleanup_script_requires_matching_package_full_name() {
        let script = msix_cleanup_script("ShellZip");
        assert!(script.contains("PackageFullName.StartsWith($identity + '_')"));
        assert!(script.contains("param("));
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
