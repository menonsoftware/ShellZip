//! Self-extracting ZIP packages.
//!
//! A package is `stub || zip || trailer`. The trailer is 24 bytes: the magic
//! `SZSFX01`, the ZIP offset, the ZIP length, and a flags byte. The stub reads
//! that trailer from its own executable and extracts only the ZIP window.
//! [`crate::detect`] prefers the trailer over a signature scan so bytes inside
//! the stub are not mistaken for the archive.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::Error;

/// Trailer magic. The digits are the layout version.
pub const MAGIC: &[u8; 7] = b"SZSFX01";

/// On-disk trailer size, in bytes.
pub const TRAILER_LEN: usize = 24;

/// Which archive is appended to a stub. Stored in bit 0 of [`SfxTrailer::flags`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxPayload {
    /// ZIP local header. Flag bit 0 is clear, which is every package written before 7z SFX.
    Zip,
    /// 7z signature. Flag bit 0 is set.
    SevenZ,
}

impl SfxPayload {
    /// Payload recorded in `flags`.
    pub fn from_flags(flags: u8) -> Self {
        if flags & 1 == 0 {
            Self::Zip
        } else {
            Self::SevenZ
        }
    }

    /// Flag byte for a new trailer. Other bits stay clear.
    pub fn flags(self) -> u8 {
        match self {
            Self::Zip => 0,
            Self::SevenZ => 1,
        }
    }

    /// Extension used when unwrapping a package.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::SevenZ => "7z",
        }
    }

    /// Container this payload is read as.
    pub fn format(self) -> crate::detect::Format {
        match self {
            Self::Zip => crate::detect::Format::Zip,
            Self::SevenZ => crate::detect::Format::SevenZ,
        }
    }
}

/// Names searched beside `szip` when `--sfx-stub` and `SZIP_SFX_STUB` are unset.
pub fn stub_file_names() -> &'static [&'static str] {
    &[
        "shellzip-sfx.exe",
        "shellzip-sfx",
        "szip-sfx-x86_64-pc-windows-msvc.exe",
        "szip-sfx-x86_64-unknown-linux-gnu",
    ]
}

/// Tail of a self-extracting package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SfxTrailer {
    /// Byte offset of the ZIP local header, relative to the start of the file.
    pub archive_offset: u64,
    /// ZIP byte length, not including this trailer.
    pub archive_len: u64,
    /// Reserved. Writers store `0`.
    pub flags: u8,
}

impl SfxTrailer {
    /// Encode the 24-byte little-endian trailer.
    pub fn encode(self) -> [u8; TRAILER_LEN] {
        let mut out = [0u8; TRAILER_LEN];
        out[..7].copy_from_slice(MAGIC);
        out[7..15].copy_from_slice(&self.archive_offset.to_le_bytes());
        out[15..23].copy_from_slice(&self.archive_len.to_le_bytes());
        out[23] = self.flags;
        out
    }

    /// Decode a 24-byte trailer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Archive`] when `bytes` is the wrong length or the magic
    /// does not match.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != TRAILER_LEN {
            return Err(Error::Archive("SFX trailer must be 24 bytes".to_string()));
        }
        if &bytes[..7] != MAGIC {
            return Err(Error::Archive(
                "SFX trailer magic is not SZSFX01".to_string(),
            ));
        }
        Ok(Self {
            archive_offset: read_u64(&bytes[7..15]),
            archive_len: read_u64(&bytes[15..23]),
            flags: bytes[23],
        })
    }

    /// Payload selected by [`Self::flags`].
    pub fn payload(self) -> SfxPayload {
        SfxPayload::from_flags(self.flags)
    }

    /// `offset + length + trailer` is `file_len`, and both spans are non-empty.
    pub fn fits(self, file_len: u64) -> bool {
        let Some(end) = self.archive_offset.checked_add(self.archive_len) else {
            return false;
        };
        let Some(total) = end.checked_add(TRAILER_LEN as u64) else {
            return false;
        };
        total == file_len && self.archive_offset > 0 && self.archive_len > 0
    }
}

/// Read the trailer when `path` ends with a valid `SZSFX01` record.
///
/// # Errors
///
/// Returns [`Error::Archive`] when the magic matches but the lengths disagree
/// with the file, or the payload is not a ZIP. A file with no trailer is
/// [`Ok`]`(None)`.
pub fn locate(path: &Path) -> Result<Option<SfxTrailer>, Error> {
    let mut file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let file_len = file
        .metadata()
        .map_err(|err| Error::io(format!("stat {}", path.display()), err))?
        .len();
    let Some(trailer) = read_trailer(&mut file, file_len)? else {
        return Ok(None);
    };
    if !payload_matches(&mut file, trailer.archive_offset, trailer.payload())? {
        return Err(Error::Archive(format!(
            "SFX payload in `{}` is not a {} archive",
            path.display(),
            trailer.payload().extension()
        )));
    }
    Ok(Some(trailer))
}

/// Copy `stub` and `archive` into `dest` and append a trailer.
///
/// `dest` is replaced only after the temporary file is complete.
///
/// # Errors
///
/// Returns [`Error::Usage`] when `stub` or `archive` is empty, and
/// [`Error::Io`] when a copy or rename fails.
pub fn build(dest: &Path, stub: &Path, archive: &Path, payload: SfxPayload) -> Result<u64, Error> {
    let stub_len = file_len(stub)?;
    let archive_len = file_len(archive)?;
    if stub_len == 0 || archive_len == 0 {
        return Err(Error::Usage(
            "SFX stub and ZIP payload must both be non-empty".to_string(),
        ));
    }
    let trailer = SfxTrailer {
        archive_offset: stub_len,
        archive_len,
        flags: payload.flags(),
    };
    let (tmp, mut writer) = crate::archiver::prepare_temp(dest)?;
    let result = (|| -> Result<(), Error> {
        copy_exact(stub, &mut writer, stub_len)?;
        copy_exact(archive, &mut writer, archive_len)?;
        writer
            .write_all(&trailer.encode())
            .map_err(|err| Error::io(format!("write {}", tmp.display()), err))?;
        writer
            .flush()
            .map_err(|err| Error::io(format!("flush {}", tmp.display()), err))?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|err| Error::io(format!("sync {}", tmp.display()), err))?;
        Ok(())
    })();
    drop(writer);
    if let Err(err) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }
    crate::archiver::commit_output(&tmp, dest)
}

/// Write the ZIP payload of an SFX file to `dest`.
///
/// # Errors
///
/// Returns [`Error::Usage`] when `sfx` has no trailer.
pub fn strip(sfx: &Path, dest: &Path) -> Result<u64, Error> {
    let trailer = locate(sfx)?.ok_or_else(|| {
        Error::Usage(format!(
            "`{}` is not a ShellZip self-extracting archive",
            sfx.display()
        ))
    })?;
    let (tmp, mut writer) = crate::archiver::prepare_temp(dest)?;
    let result = (|| -> Result<(), Error> {
        let mut file =
            File::open(sfx).map_err(|err| Error::io(format!("open {}", sfx.display()), err))?;
        file.seek(SeekFrom::Start(trailer.archive_offset))
            .map_err(|err| Error::io(format!("seek {}", sfx.display()), err))?;
        let mut limited = file.take(trailer.archive_len);
        let mut buf = Vec::new();
        let copied = crate::fs::copy_chunked(&mut limited, &mut writer, &mut buf)
            .map_err(|err| Error::io(format!("write {}", tmp.display()), err))?;
        if copied != trailer.archive_len {
            return Err(Error::Archive(format!(
                "SFX payload in `{}` ended early",
                sfx.display()
            )));
        }
        writer
            .flush()
            .map_err(|err| Error::io(format!("flush {}", tmp.display()), err))?;
        Ok(())
    })();
    drop(writer);
    if let Err(err) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }
    crate::archiver::commit_output(&tmp, dest)
}

/// Which way [`convert`] rewrote the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfxDirection {
    /// A ZIP was wrapped with `stub`.
    Wrapped,
    /// An SFX package was reduced to its ZIP payload.
    Stripped,
}

/// Wrap a ZIP or unwrap an SFX package.
///
/// # Errors
///
/// Returns [`Error::Usage`] when wrapping without a stub, when `source` is not
/// a ZIP, or when `source` and `dest` are the same path.
pub fn convert(source: &Path, dest: &Path, stub: Option<&Path>) -> Result<SfxDirection, Error> {
    if same_path(source, dest) {
        return Err(Error::Usage(
            "SFX conversion needs a different output path".to_string(),
        ));
    }
    if locate(source)?.is_some() {
        strip(source, dest)?;
        return Ok(SfxDirection::Stripped);
    }
    let stub = stub.ok_or_else(|| {
        Error::Usage("pass --sfx-stub or place shellzip-sfx next to szip to wrap a ZIP".to_string())
    })?;
    let format = crate::detect::detect_file(source)?;
    match format {
        crate::detect::Format::Zip => {
            build(dest, stub, source, SfxPayload::Zip)?;
        }
        crate::detect::Format::SevenZ => {
            ensure_stub_can_extract(source)?;
            build(dest, stub, source, SfxPayload::SevenZ)?;
        }
        _ => {
            return Err(Error::Usage(
                "only a ZIP or 7z archive can be wrapped as a self-extractor".to_string(),
            ));
        }
    }
    Ok(SfxDirection::Wrapped)
}

/// The stub links LZMA2 and AES only. Refuse a 7z that needs another codec.
fn ensure_stub_can_extract(path: &Path) -> Result<(), Error> {
    #[cfg(not(feature = "sevenz"))]
    {
        let _ = path;
        return Err(Error::Unsupported {
            format: "7Z".to_string(),
            detail: "rebuild with the `sevenz` feature to wrap a 7z archive".to_string(),
        });
    }
    #[cfg(feature = "sevenz")]
    {
        let archive = crate::extractor::sevenz::open_streaming(path, None)?;
        for folder in archive.header().folders() {
            for coder in &folder.coders {
                if !stub_codec(&coder.method_id) {
                    return Err(Error::Usage(
                        "a 7z self-extractor can only store LZMA2, LZMA, Copy, and AES".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "sevenz")]
fn stub_codec(id: &[u8]) -> bool {
    matches!(
        id,
        [0x00] | [0x21] | [0x03, 0x01, 0x01] | [0x06, 0xF1, 0x07, 0x01]
    )
}

/// A seekable window over `[start, start + len)` of an inner reader.
///
/// The ZIP reader uses this so the 24-byte trailer is not treated as a comment.
pub struct Section<R> {
    inner: R,
    start: u64,
    len: u64,
    pos: u64,
}

impl<R: Seek> Section<R> {
    fn new(mut inner: R, start: u64, len: u64) -> std::io::Result<Self> {
        inner.seek(SeekFrom::Start(start))?;
        Ok(Self {
            inner,
            start,
            len,
            pos: 0,
        })
    }
}

impl<R: Read> Read for Section<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let remaining = self.len.saturating_sub(self.pos);
        if remaining == 0 || buf.is_empty() {
            return Ok(0);
        }
        let max = usize::try_from(remaining)
            .unwrap_or(usize::MAX)
            .min(buf.len());
        let n = self.inner.read(&mut buf[..max])?;
        self.pos = self.pos.saturating_add(n as u64);
        Ok(n)
    }
}

impl<R: Seek> Seek for Section<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let next = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::Current(delta) => apply_delta(self.pos, delta)?,
            SeekFrom::End(delta) => apply_delta(self.len, delta)?,
        };
        if next > self.len {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek past ZIP payload",
            ));
        }
        self.inner.seek(SeekFrom::Start(
            self.start
                .checked_add(next)
                .ok_or_else(|| std::io::Error::other("SFX offset overflow"))?,
        ))?;
        self.pos = next;
        Ok(self.pos)
    }
}

/// Open `path`, limiting the reader to the ZIP payload when a trailer is present.
///
/// # Errors
///
/// Returns the same errors as [`locate`] when a trailer is present but invalid.
pub fn open_section(path: &Path) -> Result<Section<File>, Error> {
    let mut file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let file_len = file
        .metadata()
        .map_err(|err| Error::io(format!("stat {}", path.display()), err))?
        .len();
    if let Some(trailer) = read_trailer(&mut file, file_len)? {
        if !payload_matches(&mut file, trailer.archive_offset, trailer.payload())? {
            return Err(Error::Archive(format!(
                "SFX payload in `{}` is not a {} archive",
                path.display(),
                trailer.payload().extension()
            )));
        }
        return Section::new(file, trailer.archive_offset, trailer.archive_len)
            .map_err(|err| Error::io(format!("seek {}", path.display()), err));
    }
    Section::new(file, 0, file_len)
        .map_err(|err| Error::io(format!("seek {}", path.display()), err))
}

/// Trailer at the end of `bytes` when the magic and lengths agree.
pub(crate) fn trailer_from_bytes(bytes: &[u8]) -> Option<SfxTrailer> {
    if bytes.len() < TRAILER_LEN {
        return None;
    }
    let len = u64::try_from(bytes.len()).ok()?;
    let start = bytes.len() - TRAILER_LEN;
    let trailer = SfxTrailer::decode(&bytes[start..]).ok()?;
    trailer.fits(len).then_some(trailer)
}

fn read_trailer(file: &mut File, file_len: u64) -> Result<Option<SfxTrailer>, Error> {
    if file_len < TRAILER_LEN as u64 {
        return Ok(None);
    }
    let at = file_len - TRAILER_LEN as u64;
    file.seek(SeekFrom::Start(at))
        .map_err(|err| Error::io("seek SFX trailer", err))?;
    let mut buf = [0u8; TRAILER_LEN];
    file.read_exact(&mut buf)
        .map_err(|err| Error::io("read SFX trailer", err))?;
    if &buf[..7] != MAGIC {
        return Ok(None);
    }
    let trailer = SfxTrailer::decode(&buf)?;
    if !trailer.fits(file_len) {
        return Err(Error::Archive(
            "SFX trailer does not match the file size".to_string(),
        ));
    }
    Ok(Some(trailer))
}

fn payload_matches(file: &mut File, offset: u64, payload: SfxPayload) -> Result<bool, Error> {
    file.seek(SeekFrom::Start(offset))
        .map_err(|err| Error::io("seek SFX payload", err))?;
    let mut magic = [0u8; 6];
    let n = file
        .read(&mut magic)
        .map_err(|err| Error::io("read SFX payload", err))?;
    Ok(match payload {
        SfxPayload::Zip => n >= 4 && (&magic[..4] == b"PK\x03\x04" || &magic[..4] == b"PK\x05\x06"),
        SfxPayload::SevenZ => n == 6 && magic == [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C],
    })
}

fn file_len(path: &Path) -> Result<u64, Error> {
    let meta = std::fs::metadata(path)
        .map_err(|err| Error::io(format!("stat {}", path.display()), err))?;
    if !meta.is_file() {
        return Err(Error::Usage(format!("`{}` is not a file", path.display())));
    }
    Ok(meta.len())
}

fn copy_exact(path: &Path, writer: &mut dyn Write, expected: u64) -> Result<(), Error> {
    let mut file =
        File::open(path).map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let mut buf = Vec::new();
    let copied = crate::fs::copy_chunked(&mut file, writer, &mut buf)
        .map_err(|err| Error::io(format!("copy {}", path.display()), err))?;
    if copied != expected {
        return Err(Error::Archive(format!(
            "`{}` changed length while it was copied",
            path.display()
        )));
    }
    Ok(())
}

fn same_path(left: &Path, right: &Path) -> bool {
    if let (Ok(left), Ok(right)) = (left.canonicalize(), right.canonicalize()) {
        return left == right;
    }
    left == right
}

fn read_u64(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
}

fn apply_delta(base: u64, delta: i64) -> std::io::Result<u64> {
    if delta >= 0 {
        let delta = u64::try_from(delta).map_err(|_| std::io::Error::other("seek overflow"))?;
        base.checked_add(delta)
            .ok_or_else(|| std::io::Error::other("seek overflow"))
    } else {
        base.checked_sub(delta.unsigned_abs()).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "seek before start of ZIP")
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};

    use super::*;

    fn tiny_zip() -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(&mut cursor);
        writer
            .start_file(
                "hello.txt",
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(b"hello").unwrap();
        writer.finish().unwrap();
        cursor.into_inner()
    }

    #[test]
    fn trailer_round_trip() {
        let trailer = SfxTrailer {
            archive_offset: 128,
            archive_len: 64,
            flags: 0,
        };
        let bytes = trailer.encode();
        assert_eq!(bytes.len(), 24);
        assert_eq!(SfxTrailer::decode(&bytes).unwrap(), trailer);
        assert!(SfxTrailer::decode(b"nope").is_err());
        assert!(!trailer.fits(24));
        assert!(trailer.fits(128 + 64 + 24));
    }

    #[test]
    fn build_locate_and_strip_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let stub_path = tmp.path().join("stub.bin");
        let zip_path = tmp.path().join("in.zip");
        let sfx_path = tmp.path().join("out.exe");
        let back = tmp.path().join("back.zip");
        let zip = tiny_zip();
        std::fs::write(&stub_path, b"MZ-stub-bytes").unwrap();
        std::fs::write(&zip_path, &zip).unwrap();

        build(&sfx_path, &stub_path, &zip_path, SfxPayload::Zip).unwrap();
        let trailer = locate(&sfx_path).unwrap().expect("trailer");
        assert_eq!(trailer.archive_offset, b"MZ-stub-bytes".len() as u64);
        assert_eq!(trailer.archive_len, zip.len() as u64);

        let section = open_section(&sfx_path).unwrap();
        let mut archive = zip::ZipArchive::new(section).unwrap();
        let mut hello = archive.by_name("hello.txt").unwrap();
        let mut text = String::new();
        hello.read_to_string(&mut text).unwrap();
        assert_eq!(text, "hello");

        strip(&sfx_path, &back).unwrap();
        assert_eq!(std::fs::read(&back).unwrap(), zip);

        let wrapped = tmp.path().join("again.exe");
        assert_eq!(
            convert(&back, &wrapped, Some(&stub_path)).unwrap(),
            SfxDirection::Wrapped
        );
        assert_eq!(
            convert(&wrapped, &tmp.path().join("plain.zip"), None).unwrap(),
            SfxDirection::Stripped
        );
        assert!(convert(&back, &back, Some(&stub_path)).is_err());
    }

    #[cfg(feature = "sevenz")]
    #[test]
    fn sevenz_payload_flag_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("hello.txt");
        std::fs::write(&src, b"hello").unwrap();
        let archive = tmp.path().join("in.7z");
        crate::add(
            &archive,
            &[src],
            &crate::CompressOpts {
                level: 0,
                threads: 1,
                ..crate::CompressOpts::default()
            },
            &crate::NopProgress,
        )
        .unwrap();
        let stub = tmp.path().join("stub.bin");
        std::fs::write(&stub, b"MZ-stub").unwrap();
        let sfx = tmp.path().join("out.exe");
        build(&sfx, &stub, &archive, SfxPayload::SevenZ).unwrap();
        let trailer = locate(&sfx).unwrap().expect("trailer");
        assert_eq!(trailer.payload(), SfxPayload::SevenZ);
        match crate::detect_file(&sfx).unwrap() {
            crate::detect::Format::Sfx { inner, .. } => {
                assert!(matches!(inner.as_ref(), crate::detect::Format::SevenZ));
            }
            other => panic!("expected 7z sfx, got {other:?}"),
        }
        let back = tmp.path().join("back.7z");
        assert_eq!(convert(&sfx, &back, None).unwrap(), SfxDirection::Stripped);
        assert!(locate(&back).unwrap().is_none());
    }

    #[cfg(feature = "sevenz-codecs")]
    #[test]
    fn convert_refuses_a_ppmd_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("text.7z");
        let options = zesven::WriteOptions::new()
            .method(zesven::codec::CodecMethod::PPMd)
            .level_clamped(1);
        let mut writer = zesven::Writer::create_path(&archive)
            .unwrap()
            .options(options);
        writer
            .add_bytes(zesven::ArchivePath::new("a.txt").unwrap(), b"hello")
            .unwrap();
        let _ = writer.finish().unwrap();
        let stub = tmp.path().join("stub.bin");
        std::fs::write(&stub, b"MZ-stub").unwrap();
        let err = convert(&archive, &tmp.path().join("out.exe"), Some(&stub)).unwrap_err();
        assert!(err.to_string().contains("LZMA2"), "{err}");
        assert_eq!(err.exit_code(), 7);
    }

    #[test]
    fn rejects_a_trailer_whose_lengths_do_not_fit() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.exe");
        let mut bytes = b"MZ".to_vec();
        bytes.extend_from_slice(
            &SfxTrailer {
                archive_offset: 2,
                archive_len: 99,
                flags: 0,
            }
            .encode(),
        );
        std::fs::write(&path, bytes).unwrap();
        assert!(locate(&path).is_err());
    }
}
