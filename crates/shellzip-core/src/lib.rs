//! ShellZip archive core.
//!
//! The library packs and unpacks ZIP, tar, tar.gz, tar.zst, gzip, and zstd.
//! RAR, 7z, xz, and bzip2 are recognized and rejected with an explicit error
//! until a later milestone. Passwords are rejected the same way.
//!
//! File bytes are copied through a fixed buffer. Entry names are checked
//! before anything is created on disk.

#![forbid(unsafe_code)]

mod archiver;
mod crypto;
mod detect;
mod entry;
mod error;
mod extractor;
mod fs;
mod hash;
mod ops;
mod options;
mod progress;

pub use crypto::Password;
pub use detect::{Compression, Format, detect_bytes, detect_file, format_from_extension};
pub use entry::{Entry, EntryKind, EntryMeta, format_bytes, format_time};
pub use error::Error;
pub use extractor::{ArchiveReader, Opened, Verify};
pub use options::{
    AddReport, CompressOpts, ConflictChoice, ConflictDecider, DEFAULT_STORE_EXTS, ExtractOpts,
    ExtractReport, Overwrite, StoreList, TestReport, ZipMethod,
};
pub use progress::{NopProgress, ProgressSink};

/// Library result.
pub type Result<T> = std::result::Result<T, Error>;

/// Pack `inputs` into `dest`. The extension of `dest` selects the format.
pub fn add(
    dest: &std::path::Path,
    inputs: &[std::path::PathBuf],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<AddReport> {
    ops::add(dest, inputs, opts, progress)
}

/// List `path` after sniffing its contents.
pub fn list(path: &std::path::Path) -> Result<(Opened, Vec<Entry>)> {
    ops::list(path)
}

/// Extract `path` into `opts.output`.
pub fn extract(
    path: &std::path::Path,
    opts: &ExtractOpts,
    decide: Option<&ConflictDecider<'_>>,
    progress: &dyn ProgressSink,
) -> Result<ExtractReport> {
    ops::extract(path, opts, decide, progress)
}

/// Decode every member and verify checksums or sizes.
pub fn test(path: &std::path::Path, progress: &dyn ProgressSink) -> Result<TestReport> {
    ops::test(path, progress)
}

/// Open `path` for indexed reads.
pub fn open(path: &std::path::Path) -> Result<Box<dyn ArchiveReader>> {
    ops::open(path)
}
