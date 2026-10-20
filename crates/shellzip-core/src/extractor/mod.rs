//! Archive readers and the high-level extract/test entry points.

mod policy;
mod tar;
mod zip;

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::crypto::{Password, reject_unsupported};
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
}

impl Opened {
    /// Sniff `path` and remember how to read it.
    pub fn open(path: &Path) -> Result<Self, Error> {
        Ok(Self {
            path: path.to_path_buf(),
            format: resolve(path)?,
        })
    }

    /// Borrow the detected format.
    pub fn detected(&self) -> &Format {
        &self.format
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
            Format::Zip | Format::Sfx { .. } => {
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
            other => Err(unsupported(other)),
        }
    }
}

impl ArchiveReader for Opened {
    fn format(&self) -> Format {
        self.format.clone()
    }

    fn entries(&mut self) -> Result<Vec<Entry>, Error> {
        match &self.format {
            Format::Zip | Format::Sfx { .. } => zip::list_zip(&self.path),
            Format::Tar(compression) => tar::list_tar(&self.path, *compression),
            Format::Gzip => tar::list_single(&self.path, Compression::Gzip),
            Format::Zstd => tar::list_single(&self.path, Compression::Zstd),
            other => Err(unsupported(other)),
        }
    }

    fn extract_entry(
        &mut self,
        idx: usize,
        sink: &mut dyn Write,
        password: Option<&Password>,
    ) -> Result<Verify, Error> {
        reject_unsupported(password)?;
        let (bytes, crc32) = match &self.format {
            Format::Zip | Format::Sfx { .. } => {
                zip::extract_zip_index(&self.path, idx, sink, &crate::progress::NopProgress)?
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
            other => return Err(unsupported(other)),
        };
        Ok(Verify { bytes, crc32 })
    }

    fn test(
        &mut self,
        progress: &dyn ProgressSink,
        password: Option<&Password>,
    ) -> Result<TestReport, Error> {
        reject_unsupported(password)?;
        match &self.format {
            Format::Zip | Format::Sfx { .. } => zip::test_zip(&self.path, progress),
            Format::Tar(compression) => tar::test_tar(&self.path, *compression, progress),
            Format::Gzip => tar::test_single(&self.path, Compression::Gzip, progress),
            Format::Zstd => tar::test_single(&self.path, Compression::Zstd, progress),
            other => Err(unsupported(other)),
        }
    }
}

fn resolve(path: &Path) -> Result<Format, Error> {
    match detect_file(path)? {
        Format::Zip => Ok(Format::Zip),
        Format::Sfx { inner, offset } if matches!(inner.as_ref(), Format::Zip) => {
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
        Format::Tar(Compression::None) => Ok(Format::Tar(Compression::None)),
        other => Err(unsupported(&other)),
    }
}

fn unsupported(format: &Format) -> Error {
    Error::Unsupported {
        format: format.label(),
        detail: "extraction for this format is a later milestone".to_string(),
    }
}

fn only_first(idx: usize) -> Result<(), Error> {
    if idx == 0 {
        Ok(())
    } else {
        Err(Error::Usage(format!("entry {idx} is out of range")))
    }
}
