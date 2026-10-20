//! Failures and the process exit codes they map to.
//!
//! Codes follow the RAR convention: `0` success, `1` warning, `2` fatal,
//! `3` bad checksum, `5` write, `6` open, `7` user, `10` no files,
//! `11` password, `255` interrupted.

use std::io;
use std::path::PathBuf;

/// Library error. Display text is safe to show to a user.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Filesystem failure, with the path or action that failed.
    #[error("{context}: {source}")]
    Io {
        /// What was being attempted.
        context: String,
        /// Underlying operating-system error.
        #[source]
        source: io::Error,
    },
    /// The archive bytes are not a valid instance of the detected format.
    #[error("{0}")]
    Archive(String),
    /// A known format this build does not pack or unpack yet.
    #[error("{format} is not supported yet: {detail}")]
    Unsupported {
        /// Format name.
        format: String,
        /// Why this build stops here.
        detail: String,
    },
    /// An entry name would write outside the destination directory.
    #[error("refusing unsafe archive path `{0}`")]
    UnsafePath(String),
    /// No input matched the requested filters.
    #[error("no files")]
    NoFiles,
    /// Stored checksum does not match the decoded bytes.
    #[error("checksum mismatch for `{entry}`")]
    BadChecksum {
        /// Archive-relative name.
        entry: String,
    },
    /// The archive is encrypted and this build cannot unlock it.
    #[error("encrypted entry `{0}` requires a password, which is not supported yet")]
    Encrypted(String),
    /// Password was supplied and rejected by the format implementation.
    #[error("incorrect password")]
    BadPassword,
    /// Caller passed an invalid combination of options.
    #[error("{0}")]
    Usage(String),
    /// The operation was cancelled.
    #[error("interrupted")]
    Interrupted,
}

impl Error {
    /// Attach context to an [`io::Error`].
    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        if source.kind() == io::ErrorKind::Interrupted {
            return Self::Interrupted;
        }
        if source.kind() == io::ErrorKind::InvalidData {
            let text = source.to_string().to_ascii_lowercase();
            if text.contains("checksum") || text.contains("crc") {
                return Self::BadChecksum {
                    entry: context.into(),
                };
            }
        }
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// RAR-compatible process exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io { source, .. } => match source.kind() {
                io::ErrorKind::NotFound => 6,
                io::ErrorKind::PermissionDenied
                | io::ErrorKind::StorageFull
                | io::ErrorKind::WriteZero => 5,
                io::ErrorKind::Interrupted => 255,
                _ => 2,
            },
            Self::BadChecksum { .. } => 3,
            Self::NoFiles => 10,
            Self::Encrypted(_) | Self::BadPassword => 11,
            Self::Usage(_) | Self::UnsafePath(_) => 7,
            Self::Interrupted => 255,
            Self::Unsupported { .. } | Self::Archive(_) => 2,
        }
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::io("I/O", source)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        match err {
            zip::result::ZipError::Io(source) => Self::io("zip", source),
            zip::result::ZipError::InvalidPassword => Self::BadPassword,
            zip::result::ZipError::UnsupportedArchive(msg)
                if msg == zip::result::ZipError::PASSWORD_REQUIRED
                    || msg.to_ascii_lowercase().contains("password") =>
            {
                Self::Encrypted(msg.to_string())
            }
            other => Self::Archive(other.to_string()),
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Self {
        let path = err
            .path()
            .map(PathBuf::from)
            .map_or_else(|| "walk".to_string(), |p| p.display().to_string());
        match err.into_io_error() {
            Some(source) => Self::io(path, source),
            None => Self::Archive(format!("failed to walk {path}")),
        }
    }
}
