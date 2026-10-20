//! Single-file gzip and zstd streams. Directories belong in a tar wrapper.

use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use std::time::UNIX_EPOCH;

use flate2::Compression as GzLevel;
use flate2::write::GzEncoder;

use super::{ArchiveWriter, commit_output, copy_in, prepare_output};
use crate::entry::EntryMeta;
use crate::error::Error;
use crate::options::CompressOpts;

enum Kind {
    Gzip { level: u32 },
    Zstd { level: i32 },
}

enum Stream {
    /// Header not written yet, so the first member can set the gzip filename.
    Idle {
        file: BufWriter<std::fs::File>,
        kind: Kind,
    },
    Gzip(GzEncoder<BufWriter<std::fs::File>>),
    Zstd(zstd::stream::write::Encoder<'static, BufWriter<std::fs::File>>),
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Idle { .. } => Err(io::Error::other("encoder is not started")),
            Self::Gzip(w) => w.write(buf),
            Self::Zstd(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Idle { file, .. } => file.flush(),
            Self::Gzip(w) => w.flush(),
            Self::Zstd(w) => w.flush(),
        }
    }
}

pub struct SingleArchiver {
    stream: Option<Stream>,
    tmp: std::path::PathBuf,
    final_path: std::path::PathBuf,
    committed: bool,
    buf: Vec<u8>,
    written: bool,
}

impl SingleArchiver {
    pub fn gzip(path: &Path, opts: &CompressOpts) -> Result<Self, Error> {
        Self::idle(
            path,
            opts.force,
            Kind::Gzip {
                level: opts.gzip_level(),
            },
        )
    }

    pub fn zstd(path: &Path, opts: &CompressOpts) -> Result<Self, Error> {
        Self::idle(
            path,
            opts.force,
            Kind::Zstd {
                level: opts.zstd_level(),
            },
        )
    }

    fn idle(path: &Path, force: bool, kind: Kind) -> Result<Self, Error> {
        let (tmp, file) = prepare_output(path, force)?;
        Ok(Self {
            stream: Some(Stream::Idle { file, kind }),
            tmp,
            final_path: path.to_path_buf(),
            committed: false,
            buf: Vec::new(),
            written: false,
        })
    }

    fn label(&self) -> &'static str {
        match self.stream.as_ref() {
            Some(
                Stream::Idle {
                    kind: Kind::Zstd { .. },
                    ..
                }
                | Stream::Zstd(_),
            ) => "zstd",
            _ => "gzip",
        }
    }
}

impl Drop for SingleArchiver {
    fn drop(&mut self) {
        self.stream.take();
        if !self.committed {
            let _ = std::fs::remove_file(&self.tmp);
        }
    }
}

impl ArchiveWriter for SingleArchiver {
    fn add_file(&mut self, rel: &Path, src: &mut dyn Read, meta: &EntryMeta) -> Result<(), Error> {
        if self.written {
            let ext = if self.label() == "gzip" { "gz" } else { "zst" };
            return Err(Error::Usage(format!(
                "{} stores one file; use .tar.{ext} for multiple inputs",
                self.label()
            )));
        }
        self.start(rel, meta)?;
        let mut buf = std::mem::take(&mut self.buf);
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| Error::Archive("stream writer is closed".to_string()))?;
        let copied = copy_in(src, stream, &mut buf);
        self.buf = buf;
        let copied = copied?;
        if copied != meta.size {
            return Err(Error::Archive(format!(
                "read {copied} bytes of `{}`, expected {}",
                rel.display(),
                meta.size
            )));
        }
        self.written = true;
        Ok(())
    }

    fn add_dir(&mut self, rel: &Path, _meta: &EntryMeta) -> Result<(), Error> {
        Err(Error::Usage(format!(
            "cannot store directory `{}` in a single {} stream; use a .tar wrapper",
            rel.display(),
            self.label()
        )))
    }

    fn add_symlink(&mut self, rel: &Path, _target: &Path, _meta: &EntryMeta) -> Result<(), Error> {
        Err(Error::Usage(format!(
            "cannot store symlink `{}` in a single {} stream",
            rel.display(),
            self.label()
        )))
    }

    fn finish(mut self: Box<Self>) -> Result<u64, Error> {
        if !self.written {
            return Err(Error::NoFiles);
        }
        match self.stream.take() {
            Some(Stream::Gzip(encoder)) => encoder
                .finish()
                .and_then(|mut inner| inner.flush())
                .map_err(|err| Error::io("finish gzip", err))?,
            Some(Stream::Zstd(encoder)) => encoder
                .finish()
                .and_then(|mut inner| inner.flush())
                .map_err(|err| Error::io("finish zstd", err))?,
            Some(Stream::Idle { .. }) | None => return Err(Error::NoFiles),
        }
        let bytes = commit_output(&self.tmp, &self.final_path)?;
        self.committed = true;
        Ok(bytes)
    }
}

impl SingleArchiver {
    fn start(&mut self, rel: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let Some(Stream::Idle { file, kind }) = self.stream.take() else {
            return Err(Error::Archive("stream already started".to_string()));
        };
        let stream = match kind {
            Kind::Gzip { level } => {
                let name = crate::entry::to_archive_path(rel);
                let name: Vec<u8> = name.bytes().filter(|byte| *byte != 0).collect();
                let mut builder = flate2::GzBuilder::new();
                if !name.is_empty() {
                    builder = builder.filename(name);
                }
                if let Some(secs) = meta
                    .modified
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|duration| duration.as_secs())
                {
                    builder = builder.mtime(u32::try_from(secs).unwrap_or(u32::MAX));
                }
                Stream::Gzip(builder.write(file, GzLevel::new(level)))
            }
            Kind::Zstd { level } => {
                let mut encoder = zstd::stream::write::Encoder::new(file, level)
                    .map_err(|err| Error::io("zstd encoder", err))?;
                encoder
                    .set_pledged_src_size(Some(meta.size))
                    .map_err(|err| Error::io("zstd content size", err))?;
                Stream::Zstd(encoder)
            }
        };
        self.stream = Some(stream);
        Ok(())
    }
}
