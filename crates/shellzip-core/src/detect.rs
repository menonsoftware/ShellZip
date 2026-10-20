//! Magic-byte identification of archive containers.
//!
//! Detection looks at signatures, not file extensions. A PE image with an
//! archive appended after the stub is reported as [`Format::Sfx`].

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::Error;

/// How a tar or a single-file stream is wrapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// Uncompressed tar.
    None,
    /// gzip wrapper.
    Gzip,
    /// zstd frame.
    Zstd,
}

/// Container identified from bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Format {
    /// ZIP local header or empty ZIP.
    Zip,
    /// ustar / tar header, optionally wrapped.
    Tar(Compression),
    /// gzip stream that is not a tar.
    Gzip,
    /// zstd frame that is not a tar.
    Zstd,
    /// xz stream. Recognized only.
    Xz,
    /// bzip2 stream. Recognized only.
    Bzip2,
    /// RAR 4.x. Recognized only.
    Rar4,
    /// RAR 5. Recognized only.
    Rar5,
    /// 7z signature. Recognized only.
    SevenZ,
    /// Executable stub with an archive payload at `offset`.
    Sfx {
        /// Byte offset of the inner signature.
        offset: u64,
        /// Format of the appended payload.
        inner: Box<Format>,
    },
}

impl Format {
    /// Short label for listings and the window title.
    pub fn label(&self) -> String {
        match self {
            Self::Zip => "ZIP".to_string(),
            Self::Tar(Compression::None) => "TAR".to_string(),
            Self::Tar(Compression::Gzip) => "TAR.GZ".to_string(),
            Self::Tar(Compression::Zstd) => "TAR.ZST".to_string(),
            Self::Gzip => "GZIP".to_string(),
            Self::Zstd => "ZSTD".to_string(),
            Self::Xz => "XZ".to_string(),
            Self::Bzip2 => "BZIP2".to_string(),
            Self::Rar4 => "RAR4".to_string(),
            Self::Rar5 => "RAR5".to_string(),
            Self::SevenZ => "7Z".to_string(),
            Self::Sfx { inner, .. } => format!("SFX ({})", inner.label()),
        }
    }

    /// Formats this milestone can list, test, and extract.
    pub fn is_supported(&self) -> bool {
        match self {
            Self::Zip | Self::Tar(_) | Self::Gzip | Self::Zstd => true,
            Self::Sfx { inner, .. } => matches!(inner.as_ref(), Self::Zip),
            _ => false,
        }
    }
}

/// Choose a container from a destination path's extension.
pub fn format_from_extension(path: &Path) -> Result<Format, Error> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let format = if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Format::Tar(Compression::Gzip)
    } else if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        Format::Tar(Compression::Zstd)
    } else if name.ends_with(".tar") {
        Format::Tar(Compression::None)
    } else if name.ends_with(".zip") {
        Format::Zip
    } else if name.ends_with(".gz") {
        Format::Gzip
    } else if name.ends_with(".zst") {
        Format::Zstd
    } else {
        return Err(Error::Usage(format!(
            "cannot tell archive format from `{name}`; use .zip, .tar, .tar.gz, .tgz, .tar.zst, .tzst, .gz, or .zst"
        )));
    };
    Ok(format)
}

/// Identify `bytes`, including a stub with an appended archive.
pub fn detect_bytes(bytes: &[u8]) -> Option<Format> {
    if let Some(format) = detect_header(bytes) {
        return Some(format);
    }
    if bytes.len() >= 2 && &bytes[..2] == b"MZ" {
        return find_embedded(bytes).map(|(offset, inner)| Format::Sfx {
            offset: offset as u64,
            inner: Box::new(inner),
        });
    }
    None
}

/// Identify the file at `path`.
pub fn detect_file(path: &Path) -> Result<Format, Error> {
    let mut file =
        File::open(path).map_err(|e| Error::io(format!("open {}", path.display()), e))?;
    let mut head = [0u8; 560];
    let n = file
        .read(&mut head)
        .map_err(|e| Error::io(format!("read {}", path.display()), e))?;
    if let Some(format) = detect_header(&head[..n]) {
        return Ok(format);
    }
    if n >= 2 && &head[..2] == b"MZ" {
        if let Some((offset, inner)) = find_embedded(&head[..n]) {
            return Ok(Format::Sfx {
                offset: offset as u64,
                inner: Box::new(inner),
            });
        }
        // The stub is longer than the first read. Scan the rest of the file.
        if let Some((offset, inner)) = scan_file(&mut file, &head[..n])? {
            return Ok(Format::Sfx {
                offset,
                inner: Box::new(inner),
            });
        }
    }
    Err(Error::Archive(format!(
        "unrecognized archive `{}`",
        path.display()
    )))
}

fn detect_header(bytes: &[u8]) -> Option<Format> {
    if bytes.len() >= 4 && (bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06")) {
        return Some(Format::Zip);
    }
    if bytes.len() >= 8 && bytes.starts_with(b"Rar!\x1a\x07\x01\x00") {
        return Some(Format::Rar5);
    }
    if bytes.len() >= 7 && bytes.starts_with(b"Rar!\x1a\x07\x00") {
        return Some(Format::Rar4);
    }
    if bytes.len() >= 6 && bytes.starts_with(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
        return Some(Format::SevenZ);
    }
    if bytes.len() >= 6 && bytes.starts_with(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) {
        return Some(Format::Xz);
    }
    if bytes.len() >= 3 && bytes.starts_with(b"BZh") {
        return Some(Format::Bzip2);
    }
    if bytes.len() >= 2 && bytes.starts_with(&[0x1F, 0x8B]) {
        return Some(Format::Gzip);
    }
    if bytes.len() >= 4 && bytes.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
        return Some(Format::Zstd);
    }
    if is_tar_block(bytes) {
        return Some(Format::Tar(Compression::None));
    }
    None
}

fn find_embedded(bytes: &[u8]) -> Option<(usize, Format)> {
    const NEEDLES: [&[u8]; 6] = [
        b"PK\x03\x04",
        b"Rar!\x1a\x07\x01\x00",
        b"Rar!\x1a\x07\x00",
        &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C],
        &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00],
        &[0x1F, 0x8B],
    ];
    let mut best: Option<(usize, Format)> = None;
    for needle in NEEDLES {
        if let Some(offset) = find_subslice(bytes, needle) {
            if offset == 0 {
                continue;
            }
            if best.as_ref().is_none_or(|(at, _)| offset < *at)
                && let Some(inner) = detect_header(&bytes[offset..])
            {
                best = Some((offset, inner));
            }
        }
    }
    best
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn scan_file(file: &mut File, already: &[u8]) -> Result<Option<(u64, Format)>, Error> {
    let mut overlap = already[already.len().saturating_sub(16)..].to_vec();
    let mut consumed = already.len() as u64;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| Error::io("scan archive", e))?;
        if n == 0 {
            return Ok(None);
        }
        let mut window = Vec::with_capacity(overlap.len() + n);
        window.extend_from_slice(&overlap);
        window.extend_from_slice(&buf[..n]);
        if let Some((rel, inner)) = find_embedded(&window) {
            let base = consumed - overlap.len() as u64;
            return Ok(Some((base + rel as u64, inner)));
        }
        // Also accept a signature that begins at the start of this window only
        // when it was not part of the carried overlap. `find_embedded` skips
        // offset 0, so check the boundary explicitly.
        if let Some(inner) = detect_header(&window)
            && !matches!(inner, Format::Tar(_))
        {
            let base = consumed - overlap.len() as u64;
            if base > 0 {
                return Ok(Some((base, inner)));
            }
        }
        overlap.clear();
        let keep = 16.min(window.len());
        overlap.extend_from_slice(&window[window.len() - keep..]);
        consumed += n as u64;
    }
}

/// A 512-byte tar header has either a matching checksum or an ustar magic.
pub fn is_tar_block(bytes: &[u8]) -> bool {
    if bytes.len() < 512 || bytes.iter().take(512).all(|b| *b == 0) {
        return false;
    }
    if bytes.len() >= 262 && &bytes[257..262] == b"ustar" {
        return true;
    }
    let recorded = std::str::from_utf8(&bytes[148..156])
        .ok()
        .map(str::trim)
        .map(|s| s.trim_end_matches('\0'))
        .filter(|s| !s.is_empty())
        .and_then(|s| u32::from_str_radix(s, 8).ok());
    let Some(recorded) = recorded else {
        return false;
    };
    let mut sum = 0u32;
    for (i, byte) in bytes.iter().take(512).enumerate() {
        if (148..156).contains(&i) {
            sum += u32::from(b' ');
        } else {
            sum += u32::from(*byte);
        }
    }
    sum == recorded
}

/// Read the first tar block from a decompressed prefix, if one was fully read.
pub fn prefix_is_tar(prefix: &[u8]) -> bool {
    prefix.len() >= 512 && is_tar_block(prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers() {
        assert_eq!(detect_bytes(b"PK\x03\x04rest"), Some(Format::Zip));
        assert_eq!(detect_bytes(b"PK\x05\x06"), Some(Format::Zip));
        assert_eq!(detect_bytes(&[0x1F, 0x8B, 0x08]), Some(Format::Gzip));
        assert_eq!(
            detect_bytes(&[0x28, 0xB5, 0x2F, 0xFD, 0x00]),
            Some(Format::Zstd)
        );
        assert_eq!(
            detect_bytes(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]),
            Some(Format::Xz)
        );
        assert_eq!(detect_bytes(b"BZh9"), Some(Format::Bzip2));
        assert_eq!(detect_bytes(b"Rar!\x1a\x07\x00"), Some(Format::Rar4));
        assert_eq!(detect_bytes(b"Rar!\x1a\x07\x01\x00"), Some(Format::Rar5));
        assert_eq!(
            detect_bytes(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
            Some(Format::SevenZ)
        );
    }

    #[test]
    fn ustar_at_257() {
        let mut block = vec![0u8; 512];
        block[257..262].copy_from_slice(b"ustar");
        assert!(is_tar_block(&block));
        assert_eq!(detect_bytes(&block), Some(Format::Tar(Compression::None)));
    }

    #[test]
    fn pe_stub_with_zip_tail() {
        let mut bytes = b"MZ".to_vec();
        bytes.extend_from_slice(&[0u8; 64]);
        bytes.extend_from_slice(b"PK\x03\x04");
        match detect_bytes(&bytes) {
            Some(Format::Sfx { offset, inner }) => {
                assert_eq!(offset, 66);
                assert_eq!(*inner, Format::Zip);
            }
            other => panic!("expected sfx zip, got {other:?}"),
        }
    }

    #[test]
    fn pe_stub_with_rar5_tail() {
        let mut bytes = b"MZ----".to_vec();
        bytes.extend_from_slice(b"Rar!\x1a\x07\x01\x00");
        match detect_bytes(&bytes) {
            Some(Format::Sfx { inner, .. }) => assert_eq!(*inner, Format::Rar5),
            other => panic!("expected sfx rar, got {other:?}"),
        }
    }

    #[test]
    fn extensions() {
        assert_eq!(
            format_from_extension(Path::new("a.TAR.GZ")).unwrap(),
            Format::Tar(Compression::Gzip)
        );
        assert_eq!(
            format_from_extension(Path::new("a.tgz")).unwrap(),
            Format::Tar(Compression::Gzip)
        );
        assert_eq!(
            format_from_extension(Path::new("a.tzst")).unwrap(),
            Format::Tar(Compression::Zstd)
        );
        assert!(format_from_extension(Path::new("a.rar")).is_err());
    }
}
