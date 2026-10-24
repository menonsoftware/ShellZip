//! Passwords for ZIP AES-256, RAR, and 7z.
//!
//! A password is wiped when the value is dropped. It is never written by
//! [`Debug`], and callers must not put it in a process title, a tracing
//! field, or an error string.
//!
//! ZIP encryption is AES-256 in WinZip AE-2 form. The `zip` crate derives the
//! key with PBKDF2-HMAC-SHA1 for 1000 iterations, which the WinZip specification
//! fixes. ZIP encrypts file contents only: names, sizes, and timestamps stay
//! visible. That matches WinRAR `-p`. It is not header encryption (`-hp`),
//! which ZIP cannot do. RAR `-hp` and encrypted 7z headers stay unreadable
//! until a password is supplied. A wrong password is [`crate::Error::BadPassword`]
//! (exit 11).

use zeroize::Zeroize;

/// Owned password that is cleared on drop.
///
/// Clones do not share a buffer. Dropping one copy leaves the others intact
/// until they are dropped too.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    /// Take ownership of `value`.
    ///
    /// A rejected value is wiped before the error is returned. The stored
    /// password is wiped again when this value is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Usage`] when `value` contains a NUL byte.
    /// The RAR binding rejects those, and a NUL is not a useful password.
    pub fn new(value: impl Into<String>) -> crate::Result<Self> {
        let mut value = value.into();
        if value.as_bytes().contains(&0) {
            value.zeroize();
            return Err(crate::Error::Usage(
                "password cannot contain a NUL byte".to_string(),
            ));
        }
        Ok(Self(value))
    }

    /// Borrow the password for one cryptographic call.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_hides_the_password() {
        let password = Password::new("szip-secret-value").unwrap();
        let rendered = format!("{password:?}");
        assert!(!rendered.contains("szip-secret-value"));
        assert!(rendered.contains("redacted"));
    }

    #[test]
    fn rejects_a_nul_byte() {
        assert!(Password::new("a\0b").is_err());
    }
}
