//! Store / MSIX `IExplorerCommand` cascade for the Windows 11 compact menu.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use shellzip_integrate::{
    ExplorerCommand, MenuAction, action_flag, explorer_commands, menu_for_selection,
    paths_for_action, quote_command_line,
};
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, E_NOTIMPL, S_FALSE, S_OK};
use windows::Win32::System::Com::{CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl};
use windows::Win32::UI::Shell::{
    ECF_HASSUBCOMMANDS, ECF_SEPARATORBEFORE, ECS_ENABLED, ECS_HIDDEN, IEnumExplorerCommand,
    IEnumExplorerCommand_Impl, IExplorerCommand, IExplorerCommand_Impl, IShellItemArray, SHStrDupW,
    SIGDN_FILESYSPATH,
};
use windows::core::{
    BOOL, GUID, HRESULT, IUnknown, Interface, PCWSTR, Result as WinResult, implement,
};

use crate::windows_ext::{lock_server, module_instance, sibling_shellzip_exe};

/// Surrogate CLSID registered in the Store AppxManifest (not the classic in-proc menu).
pub const CLSID_EXPLORER_COMMAND: GUID = GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B56);

/// CreateProcessW limit is 32 767 UTF-16 units; leave headroom for the NUL.
const MAX_CMDLINE_UNITS: usize = 32_000;

#[implement(IClassFactory)]
pub struct ExplorerCommandFactory;

impl IClassFactory_Impl for ExplorerCommandFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: windows::core::Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut core::ffi::c_void,
    ) -> WinResult<()> {
        if !punkouter.is_null() {
            return Err(windows::Win32::Foundation::CLASS_E_NOAGGREGATION.into());
        }
        let root: IUnknown = RootCommand::new().into();
        let hr = unsafe { root.query(riid, ppvobject) };
        if hr.is_ok() { Ok(()) } else { Err(hr.into()) }
    }

    fn LockServer(&self, flock: BOOL) -> WinResult<()> {
        lock_server(flock.as_bool());
        Ok(())
    }
}

#[implement(IExplorerCommand)]
struct RootCommand {
    paths: Mutex<Vec<PathBuf>>,
}

impl RootCommand {
    fn new() -> Self {
        Self {
            paths: Mutex::new(Vec::new()),
        }
    }

    fn cache_paths(&self, items: windows::core::Ref<'_, IShellItemArray>) {
        let paths = paths_from_items(items).unwrap_or_default();
        if let Ok(mut guard) = self.paths.lock() {
            *guard = paths;
        }
    }

    fn cached_paths(&self) -> Vec<PathBuf> {
        self.paths.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl IExplorerCommand_Impl for RootCommand_Impl {
    fn GetTitle(
        &self,
        psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        self.cache_paths(psiitemarray);
        alloc_title("ShellZip")
    }

    fn GetIcon(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetToolTip(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetCanonicalName(&self) -> WinResult<GUID> {
        Ok(CLSID_EXPLORER_COMMAND)
    }

    fn GetState(
        &self,
        psiitemarray: windows::core::Ref<'_, IShellItemArray>,
        _foktobeslow: BOOL,
    ) -> WinResult<u32> {
        self.cache_paths(psiitemarray);
        let paths = self.cached_paths();
        let model = menu_for_selection(&paths);
        if explorer_commands(&model).is_empty() {
            Ok(ECS_HIDDEN.0 as u32)
        } else {
            Ok(ECS_ENABLED.0 as u32)
        }
    }

    fn Invoke(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
        _pbc: windows::core::Ref<'_, IBindCtx>,
    ) -> WinResult<()> {
        // Root is a flyout parent; Explorer only invokes children.
        Ok(())
    }

    fn GetFlags(&self) -> WinResult<u32> {
        Ok(ECF_HASSUBCOMMANDS.0 as u32)
    }

    fn EnumSubCommands(&self) -> WinResult<IEnumExplorerCommand> {
        let paths = self.cached_paths();
        let model = menu_for_selection(&paths);
        let commands = explorer_commands(&model);
        let enumerator: IEnumExplorerCommand = CommandEnum::new(commands).into();
        Ok(enumerator)
    }
}

#[implement(IExplorerCommand)]
struct ChildCommand {
    title: String,
    action: MenuAction,
    separator_before: bool,
}

impl ChildCommand {
    fn from_spec(spec: ExplorerCommand) -> Self {
        Self {
            title: spec.title,
            action: spec.action,
            separator_before: spec.separator_before,
        }
    }
}

impl IExplorerCommand_Impl for ChildCommand_Impl {
    fn GetTitle(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        alloc_title(&self.title)
    }

    fn GetIcon(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetToolTip(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetCanonicalName(&self) -> WinResult<GUID> {
        Ok(canonical_for(self.action))
    }

    fn GetState(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
        _foktobeslow: BOOL,
    ) -> WinResult<u32> {
        Ok(ECS_ENABLED.0 as u32)
    }

    fn Invoke(
        &self,
        psiitemarray: windows::core::Ref<'_, IShellItemArray>,
        _pbc: windows::core::Ref<'_, IBindCtx>,
    ) -> WinResult<()> {
        let paths = paths_from_items(psiitemarray).unwrap_or_default();
        if paths.is_empty() {
            return Err(E_INVALIDARG.into());
        }
        let model = menu_for_selection(&paths);
        let targets = paths_for_action(&model, self.action);
        if targets.is_empty() {
            return Err(E_INVALIDARG.into());
        }
        launch_package_shellzip(self.action, targets)
    }

    fn GetFlags(&self) -> WinResult<u32> {
        if self.separator_before {
            Ok(ECF_SEPARATORBEFORE.0 as u32)
        } else {
            Ok(0)
        }
    }

    fn EnumSubCommands(&self) -> WinResult<IEnumExplorerCommand> {
        Err(E_NOTIMPL.into())
    }
}

#[implement(IEnumExplorerCommand)]
struct CommandEnum {
    commands: Vec<ExplorerCommand>,
    index: AtomicUsize,
}

impl CommandEnum {
    fn new(commands: Vec<ExplorerCommand>) -> Self {
        Self {
            commands,
            index: AtomicUsize::new(0),
        }
    }
}

impl IEnumExplorerCommand_Impl for CommandEnum_Impl {
    fn Next(
        &self,
        celt: u32,
        puicommand: *mut Option<IExplorerCommand>,
        pceltfetched: *mut u32,
    ) -> HRESULT {
        if celt == 0 || puicommand.is_null() {
            return E_INVALIDARG;
        }
        let index = self.index.load(Ordering::SeqCst);
        if index >= self.commands.len() {
            if !pceltfetched.is_null() {
                unsafe {
                    *pceltfetched = 0;
                }
            }
            return S_FALSE;
        }
        let child: IExplorerCommand = ChildCommand::from_spec(self.commands[index].clone()).into();
        unsafe {
            *puicommand = Some(child);
            if !pceltfetched.is_null() {
                *pceltfetched = 1;
            }
        }
        self.index.store(index + 1, Ordering::SeqCst);
        S_OK
    }

    fn Skip(&self, celt: u32) -> WinResult<()> {
        let add = usize::try_from(celt).unwrap_or(usize::MAX);
        let index = self.index.load(Ordering::SeqCst);
        let next = index.saturating_add(add).min(self.commands.len());
        self.index.store(next, Ordering::SeqCst);
        if next == index && celt > 0 && index >= self.commands.len() {
            Err(S_FALSE.into())
        } else {
            Ok(())
        }
    }

    fn Reset(&self) -> WinResult<()> {
        self.index.store(0, Ordering::SeqCst);
        Ok(())
    }

    fn Clone(&self) -> WinResult<IEnumExplorerCommand> {
        let clone = CommandEnum {
            commands: self.commands.clone(),
            index: AtomicUsize::new(self.index.load(Ordering::SeqCst)),
        };
        Ok(clone.into())
    }
}

fn paths_from_items(items: windows::core::Ref<'_, IShellItemArray>) -> WinResult<Vec<PathBuf>> {
    let Some(array) = items.as_ref() else {
        return Ok(Vec::new());
    };
    let count = unsafe { array.GetCount()? };
    let mut paths = Vec::with_capacity(count as usize);
    for index in 0..count {
        let item = unsafe { array.GetItemAt(index)? };
        let name = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH)? };
        let text = unsafe { name.to_string() }.map_err(|_| E_FAIL)?;
        unsafe {
            CoTaskMemFree(Some(name.0 as *const _));
        }
        if !text.is_empty() {
            paths.push(PathBuf::from(text));
        }
    }
    Ok(paths)
}

fn alloc_title(title: &str) -> WinResult<windows::core::PWSTR> {
    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe { SHStrDupW(PCWSTR(wide.as_ptr())) }
}

fn launch_package_shellzip(action: MenuAction, files: &[PathBuf]) -> WinResult<()> {
    let _ = module_instance();
    let exe = sibling_shellzip_exe().ok_or(E_FAIL)?;
    let flag = action_flag(action);
    let arg_refs: Vec<&str> = if flag.is_empty() {
        files.iter().map(|p| p.to_str().unwrap_or("")).collect()
    } else {
        std::iter::once(flag)
            .chain(files.iter().map(|p| p.to_str().unwrap_or("")))
            .collect()
    };
    let cmdline = quote_command_line(&exe.to_string_lossy(), &arg_refs);
    if cmdline.encode_utf16().count() > MAX_CMDLINE_UNITS {
        return Err(E_FAIL.into());
    }
    let mut command = std::process::Command::new(&exe);
    if !flag.is_empty() {
        command.arg(flag);
    }
    for file in files {
        command.arg(file);
    }
    command
        .spawn()
        .map_err(|_| windows::core::Error::from(E_FAIL))?;
    Ok(())
}

fn canonical_for(action: MenuAction) -> GUID {
    match action {
        MenuAction::Add => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B57),
        MenuAction::AddZip => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B58),
        MenuAction::Add7z => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B59),
        MenuAction::Open => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B5A),
        MenuAction::ExtractTo => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B5B),
        MenuAction::ExtractHere => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B5C),
        MenuAction::ExtractHereNamed => GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B5D),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_clsid_differs_from_classic() {
        assert_ne!(CLSID_EXPLORER_COMMAND, crate::windows_ext::CLSID_SHELLZIP);
    }

    #[test]
    fn sibling_exe_absent_before_dll_attach() {
        // Unit tests load as an rlib; DllMain never ran, so MODULE is null.
        assert!(sibling_shellzip_exe().is_none());
    }
}
