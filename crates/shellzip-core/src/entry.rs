//! Archive entry metadata shared by listings and the file table.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Datelike, Local, TimeZone, Timelike};

/// What an archive member is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// Regular file payload.
    File,
    /// Directory marker.
    Dir,
    /// Symbolic link. `Entry::link_target` is set.
    Symlink,
    /// Something this tool will not materialize (device, fifo, hard link).
    Other,
}

/// One member of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Archive-relative path using `/`.
    pub path: String,
    /// File, directory, or link.
    pub kind: EntryKind,
    /// Uncompressed size, when the container records it.
    pub size: Option<u64>,
    /// Compressed size, when the container records it.
    pub packed_size: Option<u64>,
    /// Modification time.
    pub modified: Option<SystemTime>,
    /// Unix mode bits, including file-type bits when the format stores them.
    pub mode: Option<u32>,
    /// CRC32 of the uncompressed payload, when the format stores one.
    pub crc32: Option<u32>,
    /// Compression method name, when the format stores one per entry.
    pub method: Option<String>,
    /// Creator host (`Unix`, `DOS`, …), when the format records one.
    pub host_os: Option<String>,
    /// BLAKE2sp-256 of the payload, set by `lt -htb`.
    pub blake2sp: Option<[u8; 32]>,
    /// Symlink target as stored.
    pub link_target: Option<String>,
    /// The member is encrypted. This build can list it but not unpack it.
    pub encrypted: bool,
}

impl Entry {
    /// Packed/uncompressed ratio as a percentage, when both sizes are known.
    pub fn ratio_percent(&self) -> Option<u64> {
        let (packed, size) = (self.packed_size?, self.size?);
        if size == 0 {
            return Some(0);
        }
        Some(packed.saturating_mul(100) / size)
    }
}

/// Metadata captured from the filesystem while packing.
#[derive(Debug, Clone)]
pub struct EntryMeta {
    /// Uncompressed byte length. Zero for directories and symlinks.
    pub size: u64,
    /// Source modification time.
    pub modified: Option<SystemTime>,
    /// Permission bits (`0o644` style). File-type bits are not required.
    pub mode: u32,
    /// When false, writers do not check that the reader produced [`Self::size`] bytes.
    pub size_known: bool,
}

/// Render a byte count in binary units.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Format a timestamp in the local timezone, or `-` when it is missing.
pub fn format_time(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "-".to_string();
    };
    let Ok(duration) = time.duration_since(UNIX_EPOCH) else {
        return "-".to_string();
    };
    let Some(local) = Local
        .timestamp_opt(duration.as_secs() as i64, duration.subsec_nanos())
        .single()
    else {
        return "-".to_string();
    };
    local.format("%Y-%m-%d %H:%M").to_string()
}

/// `d0644` / `l0777` style attributes for a technical listing.
pub fn format_attrs(kind: EntryKind, mode: Option<u32>) -> String {
    let kind_ch = match kind {
        EntryKind::Dir => 'd',
        EntryKind::Symlink => 'l',
        EntryKind::File | EntryKind::Other => '-',
    };
    match mode {
        Some(mode) => format!("{kind_ch}{:04o}", mode & 0o7777),
        None => format!("{kind_ch}----"),
    }
}

/// CRC32 as eight uppercase hex digits, or `-`.
pub fn format_crc(crc: Option<u32>) -> String {
    match crc {
        Some(crc) => format!("{crc:08X}"),
        None => "-".to_string(),
    }
}

/// BLAKE2sp-256 as 64 lowercase hex digits, or `-`.
pub fn format_blake(hash: Option<&[u8; 32]>) -> String {
    match hash {
        Some(bytes) => bytes
            .iter()
            .fold(String::with_capacity(64), |mut out, byte| {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                out.push(char::from(HEX[usize::from(byte >> 4)]));
                out.push(char::from(HEX[usize::from(byte & 0x0f)]));
                out
            }),
        None => "-".to_string(),
    }
}

/// Archive name used to decide whether two members are the same file.
pub(crate) fn member_key(name: &str) -> String {
    name.trim()
        .trim_start_matches("./")
        .trim_end_matches(['/', '\\'])
        .replace('\\', "/")
}

/// Local calendar parts for the ZIP DOS timestamp. Out-of-range values yield `None`.
pub fn dos_parts(time: Option<SystemTime>) -> Option<(u16, u8, u8, u8, u8, u8)> {
    let time = time?;
    let duration = time.duration_since(UNIX_EPOCH).ok()?;
    let local = Local
        .timestamp_opt(duration.as_secs() as i64, duration.subsec_nanos())
        .single()?;
    Some((
        local.year().try_into().ok()?,
        local.month().try_into().ok()?,
        local.day().try_into().ok()?,
        local.hour().try_into().ok()?,
        local.minute().try_into().ok()?,
        local.second().try_into().ok()?,
    ))
}

/// Join path components with `/` and no leading `./`.
pub fn to_archive_path(path: &Path) -> String {
    let mut out = String::new();
    for component in path.components() {
        if let std::path::Component::Normal(part) = component {
            if !out.is_empty() {
                out.push('/');
            }
            out.push_str(&part.to_string_lossy());
        }
    }
    out
}
