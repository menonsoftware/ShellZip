//! Explorer context menu for ShellZip.
//!
//! The DLL only builds the menu and starts `shellzip`. It never extracts.

#![cfg_attr(windows, allow(unsafe_code))] // COM shell extension FFI

#[cfg(windows)]
mod explorer_command;
#[cfg(windows)]
mod windows_ext;

#[cfg(windows)]
pub use explorer_command::CLSID_EXPLORER_COMMAND;
#[cfg(windows)]
pub use windows_ext::*;

/// Shared CLSID string used by the integrate crate and the classic DLL.
pub const SHELL_CLSID: &str = "{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}";

/// Store-package `IExplorerCommand` CLSID (surrogate; separate from [`SHELL_CLSID`]).
pub const EXPLORER_COMMAND_CLSID: &str = "{B8D4F6C2-5E3A-4B02-AF7D-9C2E1F3A4B56}";

#[cfg(not(windows))]
/// Placeholder so non-Windows CI can compile the crate as an rlib.
pub fn unsupported() -> &'static str {
    "shellzip-shell is Windows-only"
}

#[cfg(test)]
mod tests {
    #[test]
    fn clsid_matches_integrate() {
        assert!(super::SHELL_CLSID.starts_with('{'));
        assert_eq!(super::SHELL_CLSID.len(), 38);
    }

    #[test]
    fn explorer_command_clsid_is_distinct() {
        assert_ne!(super::SHELL_CLSID, super::EXPLORER_COMMAND_CLSID);
        assert_eq!(super::EXPLORER_COMMAND_CLSID.len(), 38);
    }
}
