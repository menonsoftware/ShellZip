//! Archive writers. Callers stream payload through [`ArchiveWriter::add_file`].

use std::io::Read;
use std::path::{Path, PathBuf};

use crate::detect::{Format, format_from_extension};
use crate::entry::EntryMeta;
use crate::error::Error;
use crate::options::CompressOpts;

mod parallel;
mod stream;
mod tar;
mod zip;
mod zstd_tune;

/// Builds one archive on disk. The destination is replaced only after [`Self::finish`].
///
/// An existing ZIP is updated: [`Self::retain_existing`] copies members that this
/// run does not replace. Other formats still require `force` before a replace.
pub trait ArchiveWriter {
    /// Add a regular file. `src` must yield exactly `meta.size` bytes.
    fn add_file(&mut self, rel: &Path, src: &mut dyn Read, meta: &EntryMeta) -> Result<(), Error>;
    /// Add a directory marker.
    fn add_dir(&mut self, rel: &Path, meta: &EntryMeta) -> Result<(), Error>;
    /// Add a symlink. `target` is stored as text, not followed.
    fn add_symlink(&mut self, rel: &Path, target: &Path, meta: &EntryMeta) -> Result<(), Error>;
    /// Keep destination members whose names are not in `replacing`.
    ///
    /// Called before any `add_*`. Formats that cannot update an existing file ignore it.
    fn retain_existing(&mut self, replacing: &[String]) -> Result<(), Error> {
        let _ = replacing;
        Ok(())
    }
    /// Flush, close, and move the temporary file onto `path`. Returns the archive size.
    fn finish(self: Box<Self>) -> Result<u64, Error>;
}

/// Create a writer for `format` at `path`.
pub fn create(
    path: &Path,
    format: &Format,
    opts: &CompressOpts,
) -> Result<Box<dyn ArchiveWriter>, Error> {
    match format {
        Format::Zip => Ok(Box::new(zip::ZipArchiver::create(path, opts)?)),
        Format::Tar(compression) => Ok(Box::new(tar::TarArchiver::create(
            path,
            *compression,
            opts,
        )?)),
        Format::Gzip => Ok(Box::new(stream::SingleArchiver::gzip(path, opts)?)),
        Format::Zstd => Ok(Box::new(stream::SingleArchiver::zstd(path, opts)?)),
        Format::Xz | Format::Bzip2 => Err(Error::Unsupported {
            format: format.label(),
            detail: "creating xz and bzip2 is not supported; this build extracts them".to_string(),
        }),
        Format::Rar4 | Format::Rar5 => Err(Error::Unsupported {
            format: format.label(),
            detail: "RAR creation is not implemented; the UnRAR licence allows extraction only"
                .to_string(),
        }),
        Format::SevenZ => Err(Error::Unsupported {
            format: format.label(),
            detail: "creating 7z is not implemented; this build extracts it".to_string(),
        }),
        other => Err(Error::Unsupported {
            format: other.label(),
            detail: "creating this format is outside the current milestone".to_string(),
        }),
    }
}

pub(crate) use parallel::add_zip;

/// Writer for the format implied by `path`'s extension.
pub fn create_from_path(path: &Path, opts: &CompressOpts) -> Result<Box<dyn ArchiveWriter>, Error> {
    let format = format_from_extension(path)?;
    create(path, &format, opts)
}

pub(crate) fn prepare_output(
    path: &Path,
    force: bool,
) -> Result<(PathBuf, std::io::BufWriter<std::fs::File>), Error> {
    if path.exists() && !force {
        return Err(Error::Usage(format!(
            "archive `{}` already exists; pass -y to replace it",
            path.display()
        )));
    }
    prepare_temp(path)
}

/// Create the temporary file beside `path` without checking whether `path` exists.
///
/// ZIP updates write here, then commit over the existing archive after its other
/// members have been copied.
pub(crate) fn prepare_temp(
    path: &Path,
) -> Result<(PathBuf, std::io::BufWriter<std::fs::File>), Error> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        crate::fs::safe_path::ensure_dir(parent)?;
    }
    let tmp = temp_beside(path);
    let file = std::fs::File::create(&tmp)
        .map_err(|err| Error::io(format!("create {}", tmp.display()), err))?;
    Ok((tmp, std::io::BufWriter::with_capacity(1024 * 1024, file)))
}

pub(crate) fn commit_output(tmp: &Path, final_path: &Path) -> Result<u64, Error> {
    if final_path.exists() {
        std::fs::remove_file(final_path)
            .map_err(|err| Error::io(format!("replace {}", final_path.display()), err))?;
    }
    std::fs::rename(tmp, final_path)
        .map_err(|err| Error::io(format!("rename {}", final_path.display()), err))?;
    let meta = std::fs::metadata(final_path)
        .map_err(|err| Error::io(format!("stat {}", final_path.display()), err))?;
    Ok(meta.len())
}

fn temp_beside(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "archive".to_string());
    path.with_file_name(format!(".{name}.{}.szip-tmp", std::process::id()))
}

pub(crate) fn copy_in(
    src: &mut dyn Read,
    dst: &mut dyn std::io::Write,
    buf: &mut Vec<u8>,
) -> Result<u64, Error> {
    crate::fs::copy_chunked(src, dst, buf).map_err(|err| Error::io("write archive", err))
}
