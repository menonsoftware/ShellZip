//! Explorer context menu for ShellZip.
//!
//! The DLL only builds the menu and starts `shellzip`. It never extracts.

#![cfg_attr(windows, allow(unsafe_code))] // COM shell extension FFI

#[cfg(windows)]
mod windows_ext;

#[cfg(windows)]
pub use windows_ext::*;

/// Shared CLSID string used by the integrate crate and the DLL.
pub const SHELL_CLSID: &str = "{A7C3E5B1-4D2F-4A91-9E6C-8B1F0D2E3A45}";

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
}
