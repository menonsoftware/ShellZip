//! Open a 7z archive, including one stored after an SFX stub.
//!
//! `zesven`'s streaming reader takes its memory cap from [`StreamingConfig`].
//! It does not accept a separate unpacked-size limit, so the extract path
//! enforces [`crate::options::ExtractOpts::max_bytes`] itself.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use zesven::Password as SevenPassword;
use zesven::streaming::{StreamingArchive, StreamingConfig};

use crate::crypto::Password;
use crate::error::{Error, map_zesven};
use crate::sfx::{Section, open_section};

/// Bound on the decompressor buffer. Matches `StreamingConfig`'s own default.
const MAX_MEMORY: usize = 64 * 1024 * 1024;

/// Open `path` for a single forward pass.
///
/// An empty password is used when `password` is `None`. An encrypted header
/// then fails with [`Error::Encrypted`] rather than [`Error::BadPassword`].
pub(crate) fn open_streaming(
    path: &Path,
    password: Option<&Password>,
) -> Result<StreamingArchive<Section<File>>, Error> {
    let section = open_section(path)?;
    let secret = match password {
        Some(password) => SevenPassword::new(password.expose()),
        None => SevenPassword::new(""),
    };
    let config = StreamingConfig::default().max_memory_buffer(MAX_MEMORY);
    match StreamingArchive::open_with_config(section, secret, config) {
        Ok(archive) => Ok(archive),
        Err(err) => Err(explain_open(path, err, password.is_some())),
    }
}

/// `zesven` reports a wrong password on an encrypted header as a decoder
/// failure. An empty password can come back as LZMA input (`first byte is 0`);
/// a wrong one can come back as a short decoded header. The coder list in front
/// of that header is not encrypted, so an AES method id there means the
/// failure is the password.
fn explain_open(path: &Path, err: zesven::Error, had_password: bool) -> Error {
    if decoder_rejected(&err) && header_uses_aes(path) {
        return if had_password {
            Error::BadPassword
        } else {
            Error::Encrypted("archive headers".to_string())
        };
    }
    map_zesven(err, had_password)
}

fn decoder_rejected(err: &zesven::Error) -> bool {
    match err {
        zesven::Error::CorruptHeader { .. } | zesven::Error::CrcMismatch { .. } => true,
        zesven::Error::Io(source) => {
            matches!(
                source.kind(),
                std::io::ErrorKind::InvalidInput
                    | std::io::ErrorKind::InvalidData
                    | std::io::ErrorKind::UnexpectedEof
            ) || {
                let text = source.to_string().to_ascii_lowercase();
                text.contains("corrupt") || text.contains("lzma")
            }
        }
        _ => false,
    }
}

/// AES-256 method id, stored in the clear in front of an encrypted header.
const AES_METHOD: [u8; 4] = [0x06, 0xF1, 0x07, 0x01];

fn header_uses_aes(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let offset = crate::sfx::locate(path)
        .ok()
        .flatten()
        .map(|trailer| trailer.archive_offset)
        .unwrap_or(0);
    let Ok(_) = file.seek(SeekFrom::Start(offset)) else {
        return false;
    };
    let mut start = [0u8; 32];
    if file.read_exact(&mut start).is_err() || start[..6] != [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C] {
        return false;
    }
    let next_off = u64::from_le_bytes(start[12..20].try_into().unwrap_or([0; 8]));
    let next_size = u64::from_le_bytes(start[20..28].try_into().unwrap_or([0; 8]));
    if next_size == 0 || next_size > 1024 * 1024 {
        return false;
    }
    let Some(pos) = offset
        .checked_add(32)
        .and_then(|base| base.checked_add(next_off))
    else {
        return false;
    };
    let Ok(len) = usize::try_from(next_size) else {
        return false;
    };
    if file.seek(SeekFrom::Start(pos)).is_err() {
        return false;
    }
    let mut header = vec![0u8; len];
    if file.read_exact(&mut header).is_err() {
        return false;
    }
    header
        .windows(AES_METHOD.len())
        .any(|window| window == AES_METHOD)
}
