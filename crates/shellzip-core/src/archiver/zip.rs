//! ZIP writer. Level 0 and already-compressed extensions are stored.
//!
//! A password selects AES-256 in WinZip AE-2 form. File names stay in the clear.
//!
//! An existing ZIP is updated: members that this run does not replace are copied
//! into the new file, then the new members are appended. `force` still replaces
//! the archive.

use std::collections::HashSet;
use std::io::{self, Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::{ArchiveWriter, commit_output, copy_in, prepare_output, prepare_temp};
use crate::entry::{EntryMeta, dos_parts};
use crate::error::Error;
use crate::options::{CompressOpts, UpdateMode, ZipMethod};

pub struct ZipArchiver {
    inner: Option<ZipWriter<io::BufWriter<std::fs::File>>>,
    tmp: std::path::PathBuf,
    final_path: std::path::PathBuf,
    /// Existing archive to copy from before new members are added.
    source: Option<std::path::PathBuf>,
    committed: bool,
    buf: Vec<u8>,
    opts: CompressOpts,
    /// Comment copied from the archive being updated, when the caller did not set one.
    kept_comment: Option<Vec<u8>>,
}

impl ZipArchiver {
    pub fn create(path: &Path, opts: &CompressOpts) -> Result<Self, Error> {
        let update = path.is_file() && !opts.force;
        let (tmp, file) = if update {
            prepare_temp(path)?
        } else {
            prepare_output(path, opts.force)?
        };
        Ok(Self {
            inner: Some(ZipWriter::new(file)),
            tmp,
            final_path: path.to_path_buf(),
            source: update.then(|| path.to_path_buf()),
            committed: false,
            buf: Vec::new(),
            opts: opts.clone(),
            kept_comment: None,
        })
    }

    fn writer(&mut self) -> Result<&mut ZipWriter<io::BufWriter<std::fs::File>>, Error> {
        self.inner
            .as_mut()
            .ok_or_else(|| Error::Archive("zip writer is closed".to_string()))
    }
}

impl Drop for ZipArchiver {
    fn drop(&mut self) {
        self.inner.take();
        if !self.committed {
            let _ = std::fs::remove_file(&self.tmp);
        }
    }
}

impl ArchiveWriter for ZipArchiver {
    fn add_file(&mut self, rel: &Path, src: &mut dyn Read, meta: &EntryMeta) -> Result<(), Error> {
        let name = rel_name(rel);
        let (method, level) = method_for(&name, &self.opts);
        self.start_with(name, meta, method, level)?;
        let mut buf = std::mem::take(&mut self.buf);
        let copied = copy_in(src, self.writer()?, &mut buf);
        self.buf = buf;
        let copied = copied?;
        if meta.size_known && copied != meta.size {
            return Err(Error::Archive(format!(
                "read {} bytes of `{}`, expected {}",
                copied,
                rel.display(),
                meta.size
            )));
        }
        Ok(())
    }

    fn add_dir(&mut self, rel: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let options = options_for(meta, CompressionMethod::Stored, None, None)?;
        self.writer()?
            .add_directory(rel_name(rel), options)
            .map_err(Error::from)
    }

    fn add_symlink(&mut self, rel: &Path, target: &Path, meta: &EntryMeta) -> Result<(), Error> {
        let secret = self.opts.password.clone();
        let options = options_for(
            meta,
            CompressionMethod::Stored,
            None,
            secret.as_ref().map(crate::crypto::Password::expose),
        )?;
        self.writer()?
            .add_symlink(rel_name(rel), target_text(target), options)
            .map_err(Error::from)
    }

    fn retain_existing(&mut self, replacing: &[String]) -> Result<(), Error> {
        let Some(source) = self.source.take() else {
            return Ok(());
        };
        let replacing: HashSet<String> = replacing
            .iter()
            .map(|name| crate::entry::member_key(name))
            .collect();
        let keep = (self.opts.update == UpdateMode::Sync)
            .then_some(self.opts.sync_keep.as_ref())
            .flatten()
            .map(|names| {
                names
                    .iter()
                    .map(|name| crate::entry::member_key(name))
                    .collect::<HashSet<_>>()
            });
        let file = std::fs::File::open(&source)
            .map_err(|err| Error::io(format!("open {}", source.display()), err))?;
        let mut archive = ZipArchive::new(file).map_err(|err| {
            Error::Archive(format!("cannot add to `{}`: {err}", source.display()))
        })?;
        if self.opts.comment.is_none() {
            let bytes = archive.comment();
            if !bytes.is_empty() {
                self.kept_comment = Some(bytes.to_vec());
            }
        }
        let mut copied = HashSet::new();
        for index in 0..archive.len() {
            let entry = archive.by_index(index)?;
            let key = crate::entry::member_key(entry.name());
            if key.is_empty() || replacing.contains(&key) {
                continue;
            }
            if keep.as_ref().is_some_and(|keep| !keep.contains(&key)) {
                continue;
            }
            if !copied.insert(key) {
                continue;
            }
            self.writer()?.raw_copy_file(entry)?;
        }
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<u64, Error> {
        if self.source.is_some() {
            return Err(Error::Archive(
                "refusing to replace an existing zip without keeping its entries".to_string(),
            ));
        }
        let mut writer = self
            .inner
            .take()
            .ok_or_else(|| Error::Archive("zip writer is closed".to_string()))?;
        apply_comment(&mut writer, &self.opts, self.kept_comment.as_deref())?;
        let mut file = writer.finish()?;
        file.flush().map_err(|err| Error::io("flush zip", err))?;
        drop(file);
        let bytes = commit_output(&self.tmp, &self.final_path)?;
        self.committed = true;
        Ok(bytes)
    }
}

pub(crate) fn method_for(name: &str, opts: &CompressOpts) -> (CompressionMethod, Option<i64>) {
    if opts.level == 0 || opts.store_exts.stores(name) {
        return (CompressionMethod::Stored, None);
    }
    match opts.zip_method {
        ZipMethod::Deflate => (CompressionMethod::Deflated, Some(opts.deflate_level())),
        ZipMethod::Zstd => (CompressionMethod::Zstd, Some(i64::from(opts.zstd_level()))),
    }
}

pub(crate) fn options_for<'a>(
    meta: &EntryMeta,
    method: CompressionMethod,
    level: Option<i64>,
    password: Option<&'a str>,
) -> Result<zip::write::FileOptions<'a, ()>, Error> {
    let mut options = SimpleFileOptions::default()
        .compression_method(method)
        .compression_level(level)
        .unix_permissions(meta.mode & 0o777)
        .large_file(!meta.size_known || meta.size >= u64::from(u32::MAX));
    if let Some((year, month, day, hour, minute, second)) = dos_parts(meta.modified)
        && let Ok(stamp) = zip::DateTime::from_date_and_time(year, month, day, hour, minute, second)
    {
        options = options.last_modified_time(stamp);
    }
    if let Some(password) = password {
        return Ok(options.with_aes_encryption(zip::AesMode::Aes256, password));
    }
    Ok(options)
}

impl ZipArchiver {
    fn start_with(
        &mut self,
        name: String,
        meta: &EntryMeta,
        method: CompressionMethod,
        level: Option<i64>,
    ) -> Result<(), Error> {
        let secret = self.opts.password.clone();
        let options = options_for(
            meta,
            method,
            level,
            secret.as_ref().map(crate::crypto::Password::expose),
        )?;
        self.writer()?
            .start_file(name, options)
            .map_err(Error::from)
    }
}

fn apply_comment(
    writer: &mut ZipWriter<io::BufWriter<std::fs::File>>,
    opts: &CompressOpts,
    kept: Option<&[u8]>,
) -> Result<(), Error> {
    if let Some(text) = &opts.comment {
        if text.len() > usize::from(u16::MAX) {
            return Err(Error::Usage(
                "ZIP comment is longer than 65535 bytes".to_string(),
            ));
        }
        writer.set_comment(text.as_str());
        return Ok(());
    }
    if let Some(kept) = kept.filter(|bytes| !bytes.is_empty()) {
        if kept.len() > usize::from(u16::MAX) {
            return Err(Error::Archive(
                "existing ZIP comment is longer than 65535 bytes".to_string(),
            ));
        }
        writer.set_raw_comment(kept.to_vec().into_boxed_slice());
    }
    Ok(())
}

pub(crate) fn rel_name(rel: &Path) -> String {
    crate::entry::to_archive_path(rel)
}

impl ZipArchiver {
    /// Copy the single compressed member in `reader` into this archive.
    pub(crate) fn merge_compressed<R: Read + std::io::Seek>(
        &mut self,
        reader: R,
    ) -> Result<(), Error> {
        let mut archive = ZipArchive::new(reader)?;
        let file = archive
            .by_index(0)
            .map_err(|err| Error::Archive(format!("compressed piece: {err}")))?;
        self.writer()?.raw_copy_file(file).map_err(Error::from)
    }
}

fn target_text(target: &Path) -> String {
    target.to_string_lossy().replace('\\', "/")
}
