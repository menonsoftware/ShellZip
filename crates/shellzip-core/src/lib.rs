//! ShellZip archive core.
//!
//! The library packs and unpacks ZIP, tar, tar.gz, tar.zst, gzip, and zstd,
//! and extracts xz and bzip2. RAR and 7z are recognized and rejected until a
//! later milestone. Passwords are rejected the same way.
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
pub use entry::{
    Entry, EntryKind, EntryMeta, format_attrs, format_blake, format_bytes, format_crc, format_time,
};
pub use error::Error;
pub use extractor::{ArchiveReader, Opened, Verify};
pub use options::{
    AddReport, CompressOpts, ConflictChoice, ConflictDecider, DEFAULT_STORE_EXTS, DeleteReport,
    ExtractOpts, ExtractReport, ListOpts, MAX_THREADS, Overwrite, StoreList, TestReport,
    UpdateMode, ZipMethod,
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

/// List `path`, optionally hashing payloads with BLAKE2sp.
pub fn list_with(path: &std::path::Path, opts: &ListOpts) -> Result<(Opened, Vec<Entry>)> {
    ops::list_with(path, opts)
}

/// Write one member of `path` to `sink`.
pub fn write_member(
    path: &std::path::Path,
    name: Option<&str>,
    sink: &mut dyn std::io::Write,
) -> Result<u64> {
    ops::write_member(path, name, sink)
}

/// Remove ZIP members that match `masks`.
pub fn delete(
    path: &std::path::Path,
    masks: &[String],
    progress: &dyn ProgressSink,
) -> Result<DeleteReport> {
    ops::delete(path, masks, progress)
}

/// Pack a single member read from `input`.
///
/// `.gz` and `.zst` stream. Tar spools to a temporary file because its header
/// stores the size. ZIP is written through a seekable temporary file.
pub fn add_stdin(
    dest: &std::path::Path,
    name: &str,
    input: &mut dyn std::io::Read,
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<AddReport> {
    ops::add_stdin(dest, name, input, opts, progress)
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
