//! Detect whether this process is running inside a Windows package (MSIX).

/// True when ShellZip is running from a Store / sideloaded MSIX package.
#[cfg(windows)]
pub fn is_packaged() -> bool {
    packaged_identity().is_some()
}

/// Always false on non-Windows hosts.
#[cfg(not(windows))]
pub fn is_packaged() -> bool {
    false
}

#[cfg(windows)]
fn packaged_identity() -> Option<String> {
    use windows::Win32::Foundation::WIN32_ERROR;
    use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
    use windows::core::PWSTR;

    // APPMODEL_ERROR_NO_PACKAGE — process is not running inside a package.
    const APPMODEL_ERROR_NO_PACKAGE: WIN32_ERROR = WIN32_ERROR(15700);

    let mut length = 0u32;
    // Probe for the required buffer length.
    let probe = {
        #[allow(unsafe_code)] // Win32 package identity size query
        unsafe {
            GetCurrentPackageFullName(&mut length, None)
        }
    };
    if probe == APPMODEL_ERROR_NO_PACKAGE {
        return None;
    }
    if length == 0 {
        return None;
    }
    let mut buffer = vec![0u16; length as usize];
    let status = {
        #[allow(unsafe_code)] // Win32 package identity fill
        unsafe {
            GetCurrentPackageFullName(&mut length, Some(PWSTR(buffer.as_mut_ptr())))
        }
    };
    if status != WIN32_ERROR(0) {
        return None;
    }
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    let name = String::from_utf16_lossy(&buffer[..end]);
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpackaged_dev_builds_report_false() {
        // Unit tests run from cargo, never from an installed package.
        assert!(!is_packaged());
    }
}
