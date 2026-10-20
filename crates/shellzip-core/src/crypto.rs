//! Password values.
//!
//! Milestone 1 does not decrypt or encrypt. The type exists so later milestones
//! can accept a password without changing the reader trait, and so a password
//! is wiped when it is dropped.

use zeroize::Zeroize;

/// Owned password that is cleared on drop.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    /// Take ownership of `value`.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the password for a single cryptographic call.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl Drop for Password {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password([redacted])")
    }
}

/// Reject a password until encryption support exists.
pub(crate) fn reject_unsupported(password: Option<&Password>) -> crate::Result<()> {
    if password.is_some() {
        Err(crate::Error::Usage(
            "passwords are not supported yet".to_string(),
        ))
    } else {
        Ok(())
    }
}
