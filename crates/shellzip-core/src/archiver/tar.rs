//! Tar writer over a plain file, gzip, or zstd stream.

use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression as GzLevel;
use flate2::write::GzEncoder;
use tar::{Builder, EntryType, Header};

use super::{ArchiveWriter, commit_output, prepare_output};
use crate::detect::Compression;
use crate::entry::EntryMeta;
use crate::error::Error;
use crate::options::CompressOpts;

enum CompWriter {
    Plain(io::BufWriter<std::fs::File>),
    Gzip(GzEncoder<io::BufWriter<std::fs::File>>),
    Zstd(zstd::stream::write::Encoder<'static, io::BufWriter<std::fs::File>>),
}

impl Write for CompWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(w) => w.write(buf),
            Self::Gzip(w) => w.write(buf),
            Self::Zstd(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(w) => w.flush(),
            Self::Gzip(w) => w.flush(),
            Self::Zstd(w) => w.flush(),
        }
    }
}

impl CompWriter {
    fn finish(self) -> Result<(), Error> {
        match self {
            Self::Plain(mut w) => w.flush().map_err(|err| Error::io("flush tar", err)),
            Self::Gzip(w) => w
                .finish()
                .and_then(|mut inner| inner.flush())
                .map_err(|err| Error::io("finish gzip", err)),
            Self::Zstd(w) => w
                .finish()
                .and_then(|mut inner| inner.flush())
                .map_err(|err| Error::io("finish zstd", err)),
        }
    }
}

pub struct TarArchiver {
    builder: Option<Builder<CompWriter>>,
    tmp: std::path::PathBuf,
    final_path: std::path::PathBuf,
    committed: bool,
}

impl TarArchiver {
    pub fn create(
        path: &Path,
        compression: Compression,
        opts: &CompressOpts,
    ) -> Result<Self, Error> {
        let (tmp, file) = prepare_output(path, opts.force)?;
        let comp = match compression {
            Compression::None => CompWriter::Plain(file),
            Compression::Gzip => {
                CompWriter::Gzip(GzEncoder::new(file, GzLevel::new(opts.gzip_level())))
            }
            Compression::Zstd => CompWriter::Zstd(
                zstd::stream::write::Encoder::new(file, opts.zstd_level())
                    .map_err(|err| Error::io("zstd encoder", err))?,
            ),
        };
        Ok(Self {
            builder: Some(Builder::new(comp)),
            tmp,
            final_path: path.to_path_buf(),
            committed: false,
        })
    }
}

impl Drop for TarArchiver {
    fn drop(&mut self) {
        self.builder.take();
        if !self.committed {
            let _ = std::fs::remove_file(&self.tmp);
        }
    }
}

impl ArchiveWriter for TarArchiver {
    fn add_file(&mut self, rel: &Path, src: &mut dyn Read, meta: &EntryMeta) -> Result<(), Error> {
        let mut header = header_for(meta, EntryType::Regular);
        header.set_size(meta.size);
        let builder = self
            .builder
            .as_mut()
            .ok_or_else(|| Error::Archive("tar writer is closed".to_string()))?;
        builder
            .append_data(&mut header, rel, src)
            .map_err(|err| Error::io(format!("tar add {}", rel.display()), err))
    }

    fn add_dir(&mut self, rel: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let mut header = header_for(meta, EntryType::Directory);
        header.set_size(0);
        let name = dir_name(rel);
        self.builder
            .as_mut()
            .ok_or_else(|| Error::Archive("tar writer is closed".to_string()))?
            .append_data(&mut header, name, std::io::empty())
            .map_err(|err| Error::io(format!("tar add dir {}", rel.display()), err))
    }

    fn add_symlink(&mut self, rel: &Path, target: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let mut header = header_for(meta, EntryType::Symlink);
        header.set_size(0);
        self.builder
            .as_mut()
            .ok_or_else(|| Error::Archive("tar writer is closed".to_string()))?
            .append_link(&mut header, rel, target)
            .map_err(|err| Error::io(format!("tar add link {}", rel.display()), err))
    }

    fn finish(mut self: Box<Self>) -> Result<u64, Error> {
        let builder = self
            .builder
            .take()
            .ok_or_else(|| Error::Archive("tar writer is closed".to_string()))?;
        let comp = builder
            .into_inner()
            .map_err(|err| Error::io("finish tar", err))?;
        comp.finish()?;
        let bytes = commit_output(&self.tmp, &self.final_path)?;
        self.committed = true;
        Ok(bytes)
    }
}

fn header_for(meta: &EntryMeta, kind: EntryType) -> Header {
    let mut header = Header::new_gnu();
    header.set_entry_type(kind);
    header.set_mode(meta.mode & 0o777);
    header.set_mtime(unix_secs(meta.modified));
    header.set_uid(0);
    header.set_gid(0);
    header
}

fn dir_name(rel: &Path) -> String {
    let mut name = crate::entry::to_archive_path(rel);
    if !name.ends_with('/') {
        name.push('/');
    }
    name
}

fn unix_secs(time: Option<SystemTime>) -> u64 {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
