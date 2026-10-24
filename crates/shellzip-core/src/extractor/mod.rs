//! Archive readers and the high-level extract/test entry points.

mod policy;
#[cfg(feature = "rar")]
mod rar;
#[cfg(feature = "sevenz")]
pub(crate) mod sevenz;
mod tar;
mod zip;

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::crypto::Password;
use crate::detect::{Compression, Format, detect_file};
use crate::entry::Entry;
use crate::error::Error;
use crate::options::{ConflictDecider, ExtractOpts, ExtractReport, TestReport};
use crate::progress::ProgressSink;

/// Random-access view of an archive. Streaming formats reopen the file per call.
pub trait ArchiveReader {
    /// Container that was identified.
    fn format(&self) -> Format;
    /// Member metadata.
    fn entries(&mut self) -> Result<Vec<Entry>, Error>;
    /// Decode member `idx` into `sink`.
    fn extract_entry(
        &mut self,
        idx: usize,
        sink: &mut dyn Write,
        password: Option<&Password>,
    ) -> Result<Verify, Error>;
    /// Decode every member and check stored checksums or sizes.
    fn test(
        &mut self,
        progress: &dyn ProgressSink,
        password: Option<&Password>,
    ) -> Result<TestReport, Error>;
}

/// Bytes written for one member and their CRC32.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verify {
    /// Uncompressed bytes.
    pub bytes: u64,
    /// CRC32 of those bytes.
    pub crc32: u32,
}

/// Opened archive path plus the format used to read it.
#[derive(Debug)]
pub struct Opened {
    path: PathBuf,
    format: Format,
    password: Option<Password>,
}

impl Opened {
    /// Sniff `path` and remember how to read it.
    pub fn open(path: &Path) -> Result<Self, Error> {
        Self::open_with(path, None)
    }

    /// Sniff `path`, keeping `password` for encrypted headers and members.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Usage`] when `password` is set for a format that has no
    /// encryption, and [`Error::Unsupported`] when this build cannot read the
    /// format.
    pub fn open_with(path: &Path, password: Option<Password>) -> Result<Self, Error> {
        let format = resolve(path)?;
        if password.is_some() && !format.accepts_password() {
            return Err(Error::Usage(format!(
                "{} does not use a password",
                format.label()
            )));
        }
        Ok(Self {
            path: path.to_path_buf(),
            format,
            password,
        })
    }

    /// Borrow the detected format.
    pub fn detected(&self) -> &Format {
        &self.format
    }

    /// Member metadata. When `hash` is set, file payloads are read for BLAKE2sp.
    pub fn read_entries(&mut self, hash: bool) -> Result<Vec<Entry>, Error> {
        let path = self.path.clone();
        let format = self.format.clone();
        let password = self.password.clone();
        match &format {
            Format::Zip => zip::list_zip(&path, hash, password.as_ref()),
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::Zip) => {
                zip::list_zip(&path, hash, password.as_ref())
            }
            Format::Tar(compression) => tar::list_tar(&path, *compression, hash),
            Format::Gzip => tar::list_single(&path, Compression::Gzip, hash),
            Format::Zstd => tar::list_single(&path, Compression::Zstd, hash),
            Format::Xz => tar::list_single(&path, Compression::Xz, hash),
            Format::Bzip2 => tar::list_single(&path, Compression::Bzip2, hash),
            #[cfg(feature = "rar")]
            Format::Rar4 | Format::Rar5 => rar::list_rar(&path, hash, password.as_ref()),
            #[cfg(feature = "sevenz")]
            Format::SevenZ => sevenz::list_sevenz(&path, hash, password.as_ref()),
            #[cfg(feature = "sevenz")]
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::SevenZ) => {
                sevenz::list_sevenz(&path, hash, password.as_ref())
            }
            Format::Sfx { .. } => Err(unsupported(&format)),
            #[cfg(not(feature = "rar"))]
            Format::Rar4 | Format::Rar5 => Err(unsupported(&format)),
            #[cfg(not(feature = "sevenz"))]
            Format::SevenZ => Err(unsupported(&format)),
        }
    }

    /// Extract members selected by `selected` into `opts.output`.
    pub fn extract_to(
        &self,
        opts: &ExtractOpts,
        decide: Option<&ConflictDecider<'_>>,
        selected: &dyn Fn(&str) -> bool,
        progress: &dyn ProgressSink,
    ) -> Result<ExtractReport, Error> {
        match &self.format {
            Format::Zip => zip::extract_zip(&self.path, opts, decide, selected, progress),
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::Zip) => {
                zip::extract_zip(&self.path, opts, decide, selected, progress)
            }
            Format::Tar(compression) => {
                tar::extract_tar(&self.path, *compression, opts, decide, selected, progress)
            }
            Format::Gzip => tar::extract_single(
                &self.path,
                Compression::Gzip,
                opts,
                decide,
                selected,
                progress,
            ),
            Format::Zstd => tar::extract_single(
                &self.path,
                Compression::Zstd,
                opts,
                decide,
                selected,
                progress,
            ),
            Format::Xz => tar::extract_single(
                &self.path,
                Compression::Xz,
                opts,
                decide,
                selected,
                progress,
            ),
            Format::Bzip2 => tar::extract_single(
                &self.path,
                Compression::Bzip2,
                opts,
                decide,
                selected,
                progress,
            ),
            #[cfg(feature = "rar")]
            Format::Rar4 | Format::Rar5 => {
                rar::extract_rar(&self.path, opts, decide, selected, progress)
            }
            #[cfg(feature = "sevenz")]
            Format::SevenZ => sevenz::extract_sevenz(&self.path, opts, decide, selected, progress),
            #[cfg(feature = "sevenz")]
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::SevenZ) => {
                sevenz::extract_sevenz(&self.path, opts, decide, selected, progress)
            }
            Format::Sfx { .. } => Err(unsupported(&self.format)),
            #[cfg(not(feature = "rar"))]
            Format::Rar4 | Format::Rar5 => Err(unsupported(&self.format)),
            #[cfg(not(feature = "sevenz"))]
            Format::SevenZ => Err(unsupported(&self.format)),
        }
    }
}

impl ArchiveReader for Opened {
    fn format(&self) -> Format {
        self.format.clone()
    }

    fn entries(&mut self) -> Result<Vec<Entry>, Error> {
        self.read_entries(false)
    }

    fn extract_entry(
        &mut self,
        idx: usize,
        sink: &mut dyn Write,
        password: Option<&Password>,
    ) -> Result<Verify, Error> {
        let path = self.path.clone();
        let format = self.format.clone();
        let stored = self.password.clone();
        let password = password.or(stored.as_ref());
        let (bytes, crc32) = match &format {
            Format::Zip => {
                zip::extract_zip_index(&path, idx, sink, password, &crate::progress::NopProgress)?
            }
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::Zip) => {
                zip::extract_zip_index(&path, idx, sink, password, &crate::progress::NopProgress)?
            }
            Format::Tar(compression) => tar::copy_tar_index(&self.path, *compression, idx, sink)?,
            Format::Gzip => {
                only_first(idx)?;
                tar::copy_single(&self.path, Compression::Gzip, sink)?
            }
            Format::Zstd => {
                only_first(idx)?;
                tar::copy_single(&self.path, Compression::Zstd, sink)?
            }
            Format::Xz => {
                only_first(idx)?;
                tar::copy_single(&self.path, Compression::Xz, sink)?
            }
            Format::Bzip2 => {
                only_first(idx)?;
                tar::copy_single(&path, Compression::Bzip2, sink)?
            }
            #[cfg(feature = "rar")]
            Format::Rar4 | Format::Rar5 => rar::copy_rar_index(&path, idx, sink, password)?,
            #[cfg(feature = "sevenz")]
            Format::SevenZ => sevenz::copy_sevenz_index(&path, idx, sink, password)?,
            #[cfg(feature = "sevenz")]
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::SevenZ) => {
                sevenz::copy_sevenz_index(&path, idx, sink, password)?
            }
            Format::Sfx { .. } => return Err(unsupported(&format)),
            #[cfg(not(feature = "rar"))]
            Format::Rar4 | Format::Rar5 => return Err(unsupported(&format)),
            #[cfg(not(feature = "sevenz"))]
            Format::SevenZ => return Err(unsupported(&format)),
        };
        Ok(Verify { bytes, crc32 })
    }

    fn test(
        &mut self,
        progress: &dyn ProgressSink,
        password: Option<&Password>,
    ) -> Result<TestReport, Error> {
        let path = self.path.clone();
        let format = self.format.clone();
        let stored = self.password.clone();
        let password = password.or(stored.as_ref());
        match &format {
            Format::Zip => zip::test_zip(&path, password, progress),
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::Zip) => {
                zip::test_zip(&path, password, progress)
            }
            Format::Tar(compression) => tar::test_tar(&self.path, *compression, progress),
            Format::Gzip => tar::test_single(&self.path, Compression::Gzip, progress),
            Format::Zstd => tar::test_single(&self.path, Compression::Zstd, progress),
            Format::Xz => tar::test_single(&self.path, Compression::Xz, progress),
            Format::Bzip2 => tar::test_single(&path, Compression::Bzip2, progress),
            #[cfg(feature = "rar")]
            Format::Rar4 | Format::Rar5 => rar::test_rar(&path, progress, password),
            #[cfg(feature = "sevenz")]
            Format::SevenZ => sevenz::test_sevenz(&path, progress, password),
            #[cfg(feature = "sevenz")]
            Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::SevenZ) => {
                sevenz::test_sevenz(&path, progress, password)
            }
            Format::Sfx { .. } => Err(unsupported(&format)),
            #[cfg(not(feature = "rar"))]
            Format::Rar4 | Format::Rar5 => Err(unsupported(&format)),
            #[cfg(not(feature = "sevenz"))]
            Format::SevenZ => Err(unsupported(&format)),
        }
    }
}

fn resolve(path: &Path) -> Result<Format, Error> {
    match detect_file(path)? {
        Format::Zip => Ok(Format::Zip),
        Format::Sfx { inner, offset }
            if matches!(inner.as_ref(), Format::Zip | Format::SevenZ) && inner.is_supported() =>
        {
            Ok(Format::Sfx { inner, offset })
        }
        Format::Gzip => {
            if tar::payload_is_tar(path, Compression::Gzip)? {
                Ok(Format::Tar(Compression::Gzip))
            } else {
                Ok(Format::Gzip)
            }
        }
        Format::Zstd => {
            if tar::payload_is_tar(path, Compression::Zstd)? {
                Ok(Format::Tar(Compression::Zstd))
            } else {
                Ok(Format::Zstd)
            }
        }
        Format::Xz => {
            if tar::payload_is_tar(path, Compression::Xz)? {
                Ok(Format::Tar(Compression::Xz))
            } else {
                Ok(Format::Xz)
            }
        }
        Format::Bzip2 => {
            if tar::payload_is_tar(path, Compression::Bzip2)? {
                Ok(Format::Tar(Compression::Bzip2))
            } else {
                Ok(Format::Bzip2)
            }
        }
        Format::Tar(Compression::None) => Ok(Format::Tar(Compression::None)),
        format @ (Format::Rar4 | Format::Rar5 | Format::SevenZ) if format.is_supported() => {
            Ok(format)
        }
        other => Err(unsupported(&other)),
    }
}

fn unsupported(format: &Format) -> Error {
    let detail = match format {
        Format::Rar4 | Format::Rar5 => {
            "rebuild with the `rar` feature to extract RAR; creation stays unsupported"
        }
        Format::SevenZ => "rebuild with the `sevenz` feature to extract 7z",
        _ => "extraction for this format is a later milestone",
    };
    Error::Unsupported {
        format: format.label(),
        detail: detail.to_string(),
    }
}

fn only_first(idx: usize) -> Result<(), Error> {
    if idx == 0 {
        Ok(())
    } else {
        Err(Error::Usage(format!("entry {idx} is out of range")))
    }
}
