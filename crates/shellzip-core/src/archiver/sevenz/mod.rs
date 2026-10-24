//! 7z writer. Bytes are streamed into `zesven` and the destination is replaced
//! only after the archive is finished.
//!
//! An update recompresses members that this run keeps. A solid block cannot be
//! copied through, which is the same constraint `zesven`'s own appender has.

mod options;
mod rewrite;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use zesven::ownership::{UnixOwnership, encode_ownership_attributes};
use zesven::write::EntryMeta as SevenMeta;
use zesven::{ArchivePath, Timestamp, Writer};

use super::{ArchiveWriter, commit_output, prepare_output, prepare_temp};
use crate::entry::{EntryMeta, to_archive_path};
use crate::error::{Error, map_zesven};
use crate::options::CompressOpts;

/// Builds one 7z archive beside `path`, then moves it into place.
pub struct SevenZArchiver {
    inner: Option<Writer<io::BufWriter<File>>>,
    tmp: PathBuf,
    final_path: PathBuf,
    /// Existing archive whose other members are copied before new ones are added.
    source: Option<PathBuf>,
    committed: bool,
    opts: CompressOpts,
}

impl SevenZArchiver {
    /// Start an archive at `path`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Usage`] when the destination exists and `opts.force` is
    /// not set, and [`Error::Unsupported`] for a codec this build cannot write.
    pub fn create(path: &Path, opts: &CompressOpts) -> Result<Self, Error> {
        let options = options::write_options(opts)?;
        let update = path.is_file() && !opts.force;
        let (tmp, file) = if update {
            prepare_temp(path)?
        } else {
            prepare_output(path, opts.force)?
        };
        let writer = match Writer::create(file) {
            Ok(writer) => writer.options(options),
            Err(err) => {
                let _ = std::fs::remove_file(&tmp);
                return Err(map_zesven(err, opts.password.is_some()));
            }
        };
        Ok(Self {
            inner: Some(writer),
            tmp,
            final_path: path.to_path_buf(),
            source: update.then(|| path.to_path_buf()),
            committed: false,
            opts: opts.clone(),
        })
    }

    fn writer(&mut self) -> Result<&mut Writer<io::BufWriter<File>>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::Archive("7z writer is closed".to_string()))
    }
}

impl Drop for SevenZArchiver {
    fn drop(&mut self) {
        self.inner.take();
        if !self.committed {
            let _ = std::fs::remove_file(&self.tmp);
        }
    }
}

impl ArchiveWriter for SevenZArchiver {
    fn add_file(&mut self, rel: &Path, src: &mut dyn Read, meta: &EntryMeta) -> Result<(), Error> {
        if meta.size_known {
            return self.write_file(rel, src, meta);
        }
        let spooled = spool(src)?;
        let mut file = File::open(&spooled.path)
            .map_err(|err| Error::io(format!("open {}", spooled.path.display()), err))?;
        let known = EntryMeta {
            size: spooled.len,
            size_known: true,
            ..meta.clone_meta()
        };
        self.write_file(rel, &mut file, &known)
    }

    fn add_dir(&mut self, rel: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let path = archive_path(rel)?;
        let seven = seven_meta(meta, true, false);
        self.writer()?
            .add_directory(path, seven)
            .map_err(|err| map_zesven(err, self.opts.password.is_some()))
    }

    fn retain_existing(&mut self, replacing: &[String]) -> Result<(), Error> {
        let Some(source) = self.source.take() else {
            return Ok(());
        };
        rewrite::keep(self, &source, replacing)
    }

    fn add_symlink(&mut self, rel: &Path, target: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let text = target.to_string_lossy().replace('\\', "/");
        let mut cursor = io::Cursor::new(text.into_bytes());
        let stored = EntryMeta {
            size: cursor.get_ref().len() as u64,
            size_known: true,
            ..meta.clone_meta()
        };
        let path = archive_path(rel)?;
        let seven = seven_meta(&stored, false, true);
        self.writer()?
            .add_stream(path, &mut cursor, seven)
            .map_err(|err| map_zesven(err, self.opts.password.is_some()))
    }

    fn finish(mut self: Box<Self>) -> Result<u64, Error> {
        if self.source.is_some() {
            return Err(Error::Archive(
                "refusing to replace an existing 7z without keeping its entries".to_string(),
            ));
        }
        let writer = self
            .inner
            .take()
            .ok_or_else(|| Error::Archive("7z writer is closed".to_string()))?;
        let (result, mut file) = writer
            .finish_into_inner()
            .map_err(|err| map_zesven(err, self.opts.password.is_some()))?;
        let _ = result.entries_written;
        file.flush().map_err(|err| Error::io("flush 7z", err))?;
        file.get_ref()
            .sync_all()
            .map_err(|err| Error::io("sync 7z", err))?;
        drop(file);
        let bytes = commit_output(&self.tmp, &self.final_path)?;
        self.committed = true;
        Ok(bytes)
    }
}

fn write_file_body(
    archiver: &mut SevenZArchiver,
    rel: &Path,
    src: &mut dyn Read,
    meta: &EntryMeta,
) -> Result<(), Error> {
    let path = archive_path(rel)?;
    let seven = seven_meta(meta, false, false);
    archiver
        .writer()?
        .add_stream(path, src, seven)
        .map_err(|err| map_zesven(err, archiver.opts.password.is_some()))
}

impl SevenZArchiver {
    fn write_file(
        &mut self,
        rel: &Path,
        src: &mut dyn Read,
        meta: &EntryMeta,
    ) -> Result<(), Error> {
        write_file_body(self, rel, src, meta)
    }
}

fn archive_path(rel: &Path) -> Result<ArchivePath, Error> {
    let name = to_archive_path(rel);
    ArchivePath::new(&name).map_err(|_| Error::UnsafePath(name))
}

fn seven_meta(meta: &EntryMeta, directory: bool, symlink: bool) -> SevenMeta {
    let mut seven = if directory {
        SevenMeta::directory()
    } else {
        SevenMeta::file(meta.size)
    };
    if let Some(time) = meta.modified.and_then(filetime) {
        seven = seven.modification_time(time);
    }
    let mut attributes = encode_ownership_attributes(&UnixOwnership::default(), Some(meta.mode));
    if symlink {
        // Bit 31 marks the Unix mode. The low 16 bits carry the Windows
        // reparse flag so a reader treats the payload as a link target.
        attributes |= 0x400;
    }
    seven.attributes(attributes)
}

fn filetime(time: SystemTime) -> Option<u64> {
    Timestamp::from_system_time(time).map(|stamp| stamp.as_filetime())
}

struct Spooled {
    path: PathBuf,
    len: u64,
}

impl Drop for Spooled {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn spool(src: &mut dyn Read) -> Result<Spooled, Error> {
    let path = std::env::temp_dir().join(format!(
        "szip-7z-{}-{}.bin",
        std::process::id(),
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0)
    ));
    let mut file =
        File::create(&path).map_err(|err| Error::io(format!("create {}", path.display()), err))?;
    let mut buf = Vec::new();
    let len = match crate::fs::copy_chunked(src, &mut file, &mut buf) {
        Ok(len) => len,
        Err(err) => {
            drop(file);
            let _ = std::fs::remove_file(&path);
            return Err(Error::io("read 7z input", err));
        }
    };
    file.flush()
        .map_err(|err| Error::io(format!("flush {}", path.display()), err))?;
    Ok(Spooled { path, len })
}

trait CloneMeta {
    fn clone_meta(&self) -> EntryMeta;
}

impl CloneMeta for EntryMeta {
    fn clone_meta(&self) -> EntryMeta {
        EntryMeta {
            size: self.size,
            modified: self.modified,
            mode: self.mode,
            size_known: self.size_known,
        }
    }
}
