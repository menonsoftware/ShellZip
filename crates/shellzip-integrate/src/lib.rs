//! File associations, MIME tables, and shell-menu helpers for ShellZip.
//!
//! Handlers spawn `shellzip` with a verb. They never extract inside the file
//! manager process.

#![forbid(unsafe_code)]

mod argv;
mod formats;
mod install;
mod menu;
mod stem;
mod writers;

pub use argv::{quote_arg, quote_args, quote_command_line};
pub use formats::{
    ArchiveKind, EXTENSIONS, FormatIcon, MIME_TYPES, icon_for_extension, icon_for_path,
    kind_for_path,
};
#[cfg(windows)]
pub use install::{
    ICON_HANDLER_EXTENSIONS, MENU_HANDLER_KEYS, RegistrationKeyPlan, SHELL_CLSID,
    aumid_from_windowsapps_path, msix_cleanup_script, register_msix_uninstall_cleanup,
    registered_aumid, registered_host_exe, registration_key_plan,
    unregister_msix_uninstall_cleanup,
};
pub use install::{IntegrateError, IntegratePaths, integrate, unintegrate};
pub use menu::{
    ExplorerCommand, MenuAction, MenuModel, MenuRow, action_flag, action_label, action_verb,
    explorer_commands, is_registered_archive, linux_named_label, menu_for_selection,
    paths_for_action, selection_dir, suggested_archive_path,
};
pub use stem::folder_stem;
pub use writers::{
    desktop_entry, kde_servicemenu, macos_document_types, nemo_actions, thunar_uca,
    windows_registry_plan,
};
