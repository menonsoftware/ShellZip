//! Win32 COM shell extension: context menu and compound-extension icons.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use shellzip_integrate::{
    MenuAction, MenuRow, action_flag, action_verb, icon_for_path, menu_for_selection,
    paths_for_action, quote_command_line,
};
use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_INVALIDARG, E_NOTIMPL, E_POINTER,
    HINSTANCE, S_FALSE, S_OK,
};
use windows::Win32::System::Com::{
    IClassFactory, IClassFactory_Impl, IDataObject, IPersist_Impl, IPersistFile, IPersistFile_Impl,
    STGM,
};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::Registry::HKEY;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    CMINVOKECOMMANDINFO, DROPFILES, GCS_HELPTEXTA, GCS_VERBA, IContextMenu, IContextMenu_Impl,
    IExtractIconW, IExtractIconW_Impl, IShellExtInit, IShellExtInit_Impl,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DestroyMenu, HMENU, InsertMenuItemW, MENUITEMINFOW, MFS_ENABLED,
    MFT_SEPARATOR, MFT_STRING, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU,
};
use windows::core::{
    BOOL, GUID, HRESULT, IUnknown, Interface, PCWSTR, PWSTR, Ref, Result as WinResult, implement,
};

/// Matches `shellzip-integrate` / `SHELL_CLSID`.
pub const CLSID_SHELLZIP: GUID = GUID::from_u128(0xA7C3_E5B1_4D2F_4A91_9E6C_8B1F_0D2E_3A45);

const CF_HDROP: u16 = 15;
/// `CMF_DEFAULTONLY` — Explorer is only asking for a default verb.
const CMF_DEFAULTONLY: u32 = 0x0000_0001;

static LOCKS: AtomicU32 = AtomicU32::new(0);
static mut MODULE: HINSTANCE = HINSTANCE(std::ptr::null_mut());

#[implement(IClassFactory)]
struct ClassFactory;

impl IClassFactory_Impl for ClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> WinResult<()> {
        if !punkouter.is_null() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let ext: IUnknown = Ext::new().into();
        let hr = unsafe { ext.query(riid, ppvobject) };
        if hr.is_ok() { Ok(()) } else { Err(hr.into()) }
    }

    fn LockServer(&self, flock: BOOL) -> WinResult<()> {
        if flock.as_bool() {
            LOCKS.fetch_add(1, Ordering::SeqCst);
        } else {
            LOCKS.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
    }
}

#[implement(IShellExtInit, IContextMenu, IExtractIconW, IPersistFile)]
struct Ext {
    files: std::sync::Mutex<Vec<PathBuf>>,
}

impl Ext {
    fn new() -> Self {
        Self {
            files: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl IShellExtInit_Impl for Ext_Impl {
    fn Initialize(
        &self,
        _pidlfolder: *const ITEMIDLIST,
        pdtobj: Ref<'_, IDataObject>,
        _hkeyprogid: HKEY,
    ) -> WinResult<()> {
        let Some(data) = pdtobj.as_ref() else {
            return Ok(());
        };
        let files = read_drop_files(data).unwrap_or_default();
        if let Ok(mut guard) = self.files.lock() {
            *guard = files;
        }
        Ok(())
    }
}

impl IContextMenu_Impl for Ext_Impl {
    fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        indexmenu: u32,
        idcmdfirst: u32,
        _idcmdlast: u32,
        uflags: u32,
    ) -> HRESULT {
        if uflags & CMF_DEFAULTONLY != 0 {
            return S_OK;
        }
        let files = self.files.lock().map(|g| g.clone()).unwrap_or_default();
        if files.is_empty() {
            return S_OK;
        }
        let model = menu_for_selection(&files);
        if model.rows.is_empty() {
            return S_OK;
        }
        let submenu = match unsafe { CreatePopupMenu() } {
            Ok(menu) => menu,
            Err(_) => return E_FAIL,
        };
        let mut next_id = idcmdfirst;
        let mut position = 0u32;
        for row in &model.rows {
            let result = match row {
                MenuRow::Separator => insert_separator(submenu, position),
                MenuRow::Command { action: _, label } => {
                    let inserted = insert_string(submenu, position, next_id, label);
                    if inserted.is_ok() {
                        next_id = next_id.saturating_add(1);
                    }
                    inserted
                }
            };
            if result.is_err() {
                let _ = unsafe { DestroyMenu(submenu) };
                return E_FAIL;
            }
            position = position.saturating_add(1);
        }
        if insert_submenu(hmenu, indexmenu, submenu, "ShellZip").is_err() {
            let _ = unsafe { DestroyMenu(submenu) };
            return E_FAIL;
        }
        // MAKE_HRESULT(SEVERITY_SUCCESS, FACILITY_NULL, command count)
        let count = next_id.saturating_sub(idcmdfirst);
        HRESULT(i32::try_from(count).unwrap_or(1))
    }

    fn InvokeCommand(&self, pici: *const CMINVOKECOMMANDINFO) -> WinResult<()> {
        let info = unsafe { pici.as_ref() }.ok_or(E_POINTER)?;
        let index = info.lpVerb.0 as usize;
        // When the high word is zero, lpVerb is a command-id offset.
        let files = self.files.lock().map(|g| g.clone()).unwrap_or_default();
        let model = menu_for_selection(&files);
        let action = action_at(&model.rows, index).ok_or(E_INVALIDARG)?;
        let paths = paths_for_action(&model, action);
        if paths.is_empty() {
            return Err(E_INVALIDARG.into());
        }
        launch_shellzip(action, paths)
    }

    fn GetCommandString(
        &self,
        idcmd: usize,
        utype: u32,
        _preserved: *const u32,
        pszname: windows::core::PSTR,
        cchmax: u32,
    ) -> WinResult<()> {
        let files = self.files.lock().map(|g| g.clone()).unwrap_or_default();
        let model = menu_for_selection(&files);
        let action = action_at(&model.rows, idcmd).ok_or(E_INVALIDARG)?;
        let text = if utype == GCS_VERBA {
            action_verb(action)
        } else if utype == GCS_HELPTEXTA {
            match model.rows.iter().filter_map(command_label).nth(idcmd) {
                Some(label) => label.as_str(),
                None => return Err(E_INVALIDARG.into()),
            }
        } else {
            return Err(E_NOTIMPL.into());
        };
        write_ansi(pszname.0, cchmax, text)
    }
}

impl IPersist_Impl for Ext_Impl {
    fn GetClassID(&self) -> WinResult<GUID> {
        Ok(CLSID_SHELLZIP)
    }
}

impl IPersistFile_Impl for Ext_Impl {
    fn IsDirty(&self) -> HRESULT {
        S_FALSE
    }

    fn Load(&self, pszfilename: &PCWSTR, _dwmode: STGM) -> WinResult<()> {
        let path = unsafe { pszfilename.to_string() }.map_err(|_| E_FAIL)?;
        if let Ok(mut guard) = self.files.lock() {
            *guard = vec![PathBuf::from(path)];
        }
        Ok(())
    }

    fn Save(&self, _pszfilename: &PCWSTR, _fremember: BOOL) -> WinResult<()> {
        Err(E_NOTIMPL.into())
    }

    fn SaveCompleted(&self, _pszfilename: &PCWSTR) -> WinResult<()> {
        Ok(())
    }

    fn GetCurFile(&self) -> WinResult<PWSTR> {
        Err(E_NOTIMPL.into())
    }
}

impl IExtractIconW_Impl for Ext_Impl {
    fn GetIconLocation(
        &self,
        _uflags: u32,
        psziconfile: PWSTR,
        cchmax: u32,
        piindex: *mut i32,
        pwflags: *mut u32,
    ) -> WinResult<()> {
        let path = self
            .files
            .lock()
            .ok()
            .and_then(|g| g.first().cloned())
            .ok_or(E_FAIL)?;
        let icon = icon_file_for(&path).ok_or(E_FAIL)?;
        unsafe {
            if !piindex.is_null() {
                *piindex = 0;
            }
            if !pwflags.is_null() {
                *pwflags = 0;
            }
        }
        write_wide(psziconfile.0, cchmax, &icon.to_string_lossy())
    }

    fn Extract(
        &self,
        _pszfile: &PCWSTR,
        _niconindex: u32,
        _phiconlarge: *mut windows::Win32::UI::WindowsAndMessaging::HICON,
        _phiconsmall: *mut windows::Win32::UI::WindowsAndMessaging::HICON,
        _niconsize: u32,
    ) -> WinResult<()> {
        Err(S_FALSE.into())
    }
}

fn icon_file_for(path: &Path) -> Option<PathBuf> {
    let stem = icon_for_path(path)?.0;
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
    let ico = base
        .join("ShellZip")
        .join("icons")
        .join(format!("{stem}.ico"));
    ico.is_file().then_some(ico)
}

fn launch_shellzip(action: MenuAction, files: &[PathBuf]) -> WinResult<()> {
    let exe = shellzip_exe().ok_or(E_FAIL)?;
    let flag = action_flag(action);
    let mut command = std::process::Command::new(&exe);
    if !flag.is_empty() {
        command.arg(flag);
    }
    for file in files {
        command.arg(file);
    }
    let arg_refs: Vec<&str> = if flag.is_empty() {
        files.iter().map(|p| p.to_str().unwrap_or("")).collect()
    } else {
        std::iter::once(flag)
            .chain(files.iter().map(|p| p.to_str().unwrap_or("")))
            .collect()
    };
    let _ = quote_command_line(&exe.to_string_lossy(), &arg_refs);
    command
        .spawn()
        .map_err(|_| windows::core::Error::from(E_FAIL))?;
    Ok(())
}

fn shellzip_exe() -> Option<PathBuf> {
    let mut buf = vec![0u16; 520];
    let len = unsafe { GetModuleFileNameW(Some(MODULE.into()), &mut buf) } as usize;
    if len == 0 {
        return None;
    }
    buf.truncate(len);
    let dll = PathBuf::from(OsString::from_wide(&buf));
    let dir = dll.parent()?;
    let candidate = dir.join("shellzip.exe");
    if candidate.is_file() {
        return Some(candidate);
    }
    which("shellzip.exe").or_else(|| which("shellzip"))
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn action_at(rows: &[MenuRow], index: usize) -> Option<MenuAction> {
    rows.iter()
        .filter_map(|row| match row {
            MenuRow::Command { action, .. } => Some(*action),
            MenuRow::Separator => None,
        })
        .nth(index)
}

fn command_label(row: &MenuRow) -> Option<&String> {
    match row {
        MenuRow::Command { label, .. } => Some(label),
        MenuRow::Separator => None,
    }
}

fn insert_string(hmenu: HMENU, position: u32, id: u32, text: &str) -> WinResult<()> {
    let wide = wide_null(text);
    let info = MENUITEMINFOW {
        cbSize: u32::try_from(std::mem::size_of::<MENUITEMINFOW>()).unwrap_or(0),
        fMask: MIIM_ID | MIIM_STRING | MIIM_FTYPE | MIIM_STATE,
        fType: MFT_STRING,
        fState: MFS_ENABLED,
        wID: id,
        dwTypeData: PWSTR(wide.as_ptr() as *mut u16),
        cch: u32::try_from(text.encode_utf16().count()).unwrap_or(0),
        ..Default::default()
    };
    let result = unsafe { InsertMenuItemW(hmenu, position, true, &info) };
    drop(wide);
    result?;
    Ok(())
}

fn insert_separator(hmenu: HMENU, position: u32) -> WinResult<()> {
    let info = MENUITEMINFOW {
        cbSize: u32::try_from(std::mem::size_of::<MENUITEMINFOW>()).unwrap_or(0),
        fMask: MIIM_FTYPE,
        fType: MFT_SEPARATOR,
        ..Default::default()
    };
    unsafe { InsertMenuItemW(hmenu, position, true, &info) }?;
    Ok(())
}

fn insert_submenu(hmenu: HMENU, position: u32, submenu: HMENU, text: &str) -> WinResult<()> {
    let wide = wide_null(text);
    let info = MENUITEMINFOW {
        cbSize: u32::try_from(std::mem::size_of::<MENUITEMINFOW>()).unwrap_or(0),
        fMask: MIIM_STRING | MIIM_FTYPE | MIIM_STATE | MIIM_SUBMENU,
        fType: MFT_STRING,
        fState: MFS_ENABLED,
        hSubMenu: submenu,
        dwTypeData: PWSTR(wide.as_ptr() as *mut u16),
        cch: u32::try_from(text.encode_utf16().count()).unwrap_or(0),
        ..Default::default()
    };
    let result = unsafe { InsertMenuItemW(hmenu, position, true, &info) };
    drop(wide);
    result?;
    Ok(())
}

fn write_wide(dest: *mut u16, cch: u32, text: &str) -> WinResult<()> {
    if dest.is_null() || cch == 0 {
        return Err(E_POINTER.into());
    }
    let wide = wide_null(text);
    let copy = wide.len().min(cch as usize);
    unsafe {
        std::ptr::copy_nonoverlapping(wide.as_ptr(), dest, copy);
        if copy > 0 {
            *dest.add(copy - 1) = 0;
        }
    }
    Ok(())
}

fn write_ansi(dest: *mut u8, cch: u32, text: &str) -> WinResult<()> {
    if dest.is_null() || cch == 0 {
        return Err(E_POINTER.into());
    }
    let bytes = text.as_bytes();
    let copy = bytes.len().min(cch as usize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, copy);
        *dest.add(copy) = 0;
    }
    Ok(())
}

fn wide_null(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_drop_files(data: &IDataObject) -> WinResult<Vec<PathBuf>> {
    use windows::Win32::System::Com::{DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL};

    let fmt = FORMATETC {
        cfFormat: CF_HDROP,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    let medium = unsafe { data.GetData(&fmt)? };
    let hglobal = unsafe { medium.u.hGlobal };
    let ptr = unsafe { GlobalLock(hglobal) };
    if ptr.is_null() {
        return Err(E_FAIL.into());
    }
    let drop = ptr as *const DROPFILES;
    let offset = unsafe { (*drop).pFiles } as usize;
    let mut paths = Vec::new();
    let mut cursor = unsafe { (ptr as *const u8).add(offset) as *const u16 };
    loop {
        let mut len = 0usize;
        while unsafe { *cursor.add(len) } != 0 {
            len += 1;
        }
        if len == 0 {
            break;
        }
        let slice = unsafe { std::slice::from_raw_parts(cursor, len) };
        paths.push(PathBuf::from(OsString::from_wide(slice)));
        cursor = unsafe { cursor.add(len + 1) };
    }
    unsafe {
        let _ = GlobalUnlock(hglobal);
    }
    Ok(paths)
}

/// DLL entry point.
#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    module: HINSTANCE,
    reason: u32,
    _reserved: *mut core::ffi::c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            MODULE = module;
        }
    }
    BOOL(1)
}

/// COM class object lookup.
///
/// Not `pub`: a `pub` `#[no_mangle]` would also emit `/EXPORT:` without
/// `PRIVATE`, which fights `exports.def` and triggers LNK4104.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // COM export; the shell owns the pointers
extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if rclsid.is_null() || unsafe { *rclsid } != CLSID_SHELLZIP {
        return CLASS_E_CLASSNOTAVAILABLE;
    }
    let factory: IClassFactory = ClassFactory.into();
    unsafe { factory.query(riid, ppv) }
}

/// True when no outstanding locks remain.
#[unsafe(no_mangle)]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LOCKS.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}
