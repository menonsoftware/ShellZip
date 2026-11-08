//! Store / MSIX `IExplorerCommand` cascade for the Windows 11 compact menu.
//!
//! Explorer may activate a fresh COM object for `EnumSubCommands`, so the root
//! never relies on a cached selection. It always enumerates a fixed child set;
//! each child reads the live `IShellItemArray` in `GetTitle` / `GetState` /
//! `Invoke`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use shellzip_integrate::{
    ExplorerCommand, MenuAction, action_flag, action_label, explorer_commands, menu_for_selection,
    paths_for_action, quote_args, quote_command_line,
};
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, E_NOTIMPL, S_FALSE, S_OK};
use windows::Win32::System::Com::{CoTaskMemFree, IBindCtx, IClassFactory, IClassFactory_Impl};
use windows::Win32::UI::Shell::{
    ECF_HASSUBCOMMANDS, ECF_SEPARATORBEFORE, ECS_ENABLED, ECS_HIDDEN, IEnumExplorerCommand,
    IEnumExplorerCommand_Impl, IExplorerCommand, IExplorerCommand_Impl, IShellItemArray, SHStrDupW,
    SIGDN, SIGDN_DESKTOPABSOLUTEPARSING, SIGDN_FILESYSPATH,
};
use windows::core::{
    BOOL, GUID, HRESULT, IUnknown, Interface, PCWSTR, Result as WinResult, implement,
};

use crate::windows_ext::{
    activate_packaged_shellzip, launch_aumid_for, lock_server, module_instance,
    sibling_shellzip_exe,
};

/// Surrogate CLSID registered in the Store AppxManifest (not the classic in-proc menu).
pub const CLSID_EXPLORER_COMMAND: GUID = GUID::from_u128(0xB8D4_F6C2_5E3A_4B02_AF7D_9C2E_1F3A_4B56);

/// CreateProcessW limit is 32 767 UTF-16 units; leave headroom for the NUL.
const MAX_CMDLINE_UNITS: usize = 32_000;

/// Every child the cascade can expose. Visibility is decided per-selection in
/// [`ChildCommand`]'s `GetState`, not when enumerating.
const ALL_ACTIONS: &[MenuAction] = &[
    MenuAction::Add,
    MenuAction::AddZip,
    MenuAction::Add7z,
    MenuAction::Open,
    MenuAction::ExtractTo,
    MenuAction::ExtractHere,
    MenuAction::ExtractHereNamed,
];

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
        let root: IUnknown = RootCommand.into();
        let hr = unsafe { root.query(riid, ppvobject) };
        if hr.is_ok() { Ok(()) } else { Err(hr.into()) }
    }

    fn LockServer(&self, flock: BOOL) -> WinResult<()> {
        lock_server(flock.as_bool());
        Ok(())
    }
}

#[implement(IExplorerCommand)]
struct RootCommand;

impl IExplorerCommand_Impl for RootCommand_Impl {
    fn GetTitle(
        &self,
        _psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
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
        Ok(GUID::zeroed())
    }

    fn GetState(
        &self,
        psiitemarray: windows::core::Ref<'_, IShellItemArray>,
        _foktobeslow: BOOL,
    ) -> WinResult<u32> {
        // Never hide solely because path extraction failed — children decide.
        // A null array is common for folder background; still show the cascade.
        if psiitemarray.as_ref().is_none() {
            return Ok(ECS_ENABLED.0 as u32);
        }
        let paths = paths_from_items(psiitemarray).unwrap_or_default();
        if paths.is_empty() {
            // Empty array with a non-null pointer: keep enabled so EnumSubCommands
            // still runs (background / odd shell namespaces).
            return Ok(ECS_ENABLED.0 as u32);
        }
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
        Ok(())
    }

    fn GetFlags(&self) -> WinResult<u32> {
        Ok(ECF_HASSUBCOMMANDS.0 as u32)
    }

    fn EnumSubCommands(&self) -> WinResult<IEnumExplorerCommand> {
        let commands: Vec<ExplorerCommand> = ALL_ACTIONS
            .iter()
            .map(|action| ExplorerCommand {
                title: action_label(*action).to_string(),
                action: *action,
                separator_before: matches!(
                    action,
                    MenuAction::Open
                        | MenuAction::ExtractTo
                        | MenuAction::ExtractHere
                        | MenuAction::ExtractHereNamed
                ) && *action == MenuAction::Open,
            })
            .collect();
        let enumerator: IEnumExplorerCommand = CommandEnum::new(commands).into();
        Ok(enumerator)
    }
}

#[implement(IExplorerCommand)]
struct ChildCommand {
    action: MenuAction,
}

impl ChildCommand {
    fn new(action: MenuAction) -> Self {
        Self { action }
    }

    fn spec_for(&self, paths: &[PathBuf]) -> Option<ExplorerCommand> {
        explorer_commands(&menu_for_selection(paths))
            .into_iter()
            .find(|cmd| cmd.action == self.action)
    }
}

impl IExplorerCommand_Impl for ChildCommand_Impl {
    fn GetTitle(
        &self,
        psiitemarray: windows::core::Ref<'_, IShellItemArray>,
    ) -> WinResult<windows::core::PWSTR> {
        let paths = paths_from_items(psiitemarray).unwrap_or_default();
        let title = self
            .spec_for(&paths)
            .map(|spec| spec.title)
            .unwrap_or_else(|| action_label(self.action).to_string());
        alloc_title(&title)
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
        psiitemarray: windows::core::Ref<'_, IShellItemArray>,
        _foktobeslow: BOOL,
    ) -> WinResult<u32> {
        let paths = paths_from_items(psiitemarray).unwrap_or_default();
        if self.spec_for(&paths).is_some() {
            Ok(ECS_ENABLED.0 as u32)
        } else if paths.is_empty()
            && matches!(
                self.action,
                MenuAction::Add | MenuAction::AddZip | MenuAction::Add7z
            )
        {
            // Folder background often yields an empty array; still offer Add.
            Ok(ECS_ENABLED.0 as u32)
        } else {
            Ok(ECS_HIDDEN.0 as u32)
        }
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
        if self.action == MenuAction::Open {
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
        let child: IExplorerCommand = ChildCommand::new(self.commands[index].action).into();
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
    let count = unsafe { array.GetCount().unwrap_or(0) };
    let mut paths = Vec::with_capacity(count as usize);
    for index in 0..count {
        let Ok(item) = (unsafe { array.GetItemAt(index) }) else {
            continue;
        };
        if let Some(path) = display_path(&item) {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn display_path(item: &windows::Win32::UI::Shell::IShellItem) -> Option<PathBuf> {
    for sigdn in [SIGDN_FILESYSPATH, SIGDN_DESKTOPABSOLUTEPARSING] {
        if let Some(path) = display_path_with(item, sigdn) {
            return Some(path);
        }
    }
    None
}

fn display_path_with(
    item: &windows::Win32::UI::Shell::IShellItem,
    sigdn: SIGDN,
) -> Option<PathBuf> {
    let name = unsafe { item.GetDisplayName(sigdn).ok()? };
    let text = unsafe { name.to_string().ok() };
    unsafe {
        CoTaskMemFree(Some(name.0 as *const _));
    }
    text.filter(|s| !s.is_empty()).map(PathBuf::from)
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
    if let Some(aumid) = launch_aumid_for(&exe) {
        let args_line = quote_args(&arg_refs);
        if args_line.encode_utf16().count() > MAX_CMDLINE_UNITS {
            return Err(E_FAIL.into());
        }
        return activate_packaged_shellzip(&aumid, &arg_refs);
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
        assert!(sibling_shellzip_exe().is_none());
    }

    #[test]
    fn cascade_enumerates_every_action() {
        assert_eq!(ALL_ACTIONS.len(), 7);
        assert_eq!(ALL_ACTIONS[0], MenuAction::Add);
        assert_eq!(ALL_ACTIONS[3], MenuAction::Open);
    }
}
