//! Parallel ZIP compression.
//!
//! Each worker compresses one entry into its own zip piece (in memory under
//! 1 MiB, otherwise a temporary file). The main thread merges those pieces in
//! input order with `ZipWriter::raw_copy_file`, so the archive layout matches
//! the single-thread writer. At most `--threads` pieces exist at once.

use std::fs::File;
use std::io::{BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use zip::ZipWriter;

use super::zip::{ZipArchiver, method_for, options_for, rel_name};
use super::{ArchiveWriter, copy_in};
use crate::entry::EntryKind;
use crate::error::Error;
use crate::fs::walk::Planned;
use crate::options::CompressOpts;
use crate::progress::{ProgressReader, ProgressSink};

const MEMORY_LIMIT: u64 = 1024 * 1024;

enum Piece {
    Memory(Vec<u8>),
    Temp(PathBuf),
}

impl Drop for Piece {
    fn drop(&mut self) {
        if let Self::Temp(path) = self {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Pack `planned` into a ZIP, compressing file entries `threads` at a time.
pub(crate) fn add_zip(
    dest: &Path,
    planned: &[Planned],
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<u64, Error> {
    let mut archiver = ZipArchiver::create(dest, opts)?;
    let replacing: Vec<String> = planned.iter().map(|item| item.rel.clone()).collect();
    archiver.retain_existing(&replacing)?;

    let threads = opts.thread_count();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|err| Error::Archive(format!("thread pool: {err}")))?;

    let mut offset = 0;
    while offset < planned.len() {
        let end = (offset + threads).min(planned.len());
        let chunk = &planned[offset..end];
        let pieces = pool.install(|| {
            chunk
                .par_iter()
                .enumerate()
                .map(|(index, item)| {
                    if item.kind == EntryKind::File {
                        compress_file(item, opts, progress)
                            .map(Some)
                            .map_err(|err| (index, err))
                    } else {
                        Ok(None)
                    }
                })
                .collect::<Result<Vec<_>, (usize, Error)>>()
        });
        let pieces = match pieces {
            Ok(pieces) => pieces,
            Err((_, err)) => return Err(err),
        };
        for (item, piece) in chunk.iter().zip(pieces) {
            crate::progress::ensure_running(progress)?;
            progress.begin_entry(&item.rel, item.meta.size);
            match item.kind {
                EntryKind::Dir => archiver.add_dir(Path::new(&item.rel), &item.meta)?,
                EntryKind::Symlink => {
                    let target = item.link_target.as_deref().unwrap_or("");
                    archiver.add_symlink(Path::new(&item.rel), Path::new(target), &item.meta)?;
                }
                EntryKind::File => {
                    let piece = piece.ok_or_else(|| {
                        Error::Archive(format!("missing compressed piece for `{}`", item.rel))
                    })?;
                    merge(&mut archiver, &piece)?;
                }
                EntryKind::Other => {}
            }
            progress.end_entry();
        }
        offset = end;
    }

    super::ArchiveWriter::finish(Box::new(archiver))
}

fn merge(archiver: &mut ZipArchiver, piece: &Piece) -> Result<(), Error> {
    match piece {
        Piece::Memory(bytes) => archiver.merge_compressed(Cursor::new(bytes.as_slice())),
        Piece::Temp(path) => {
            let file = File::open(path)
                .map_err(|err| Error::io(format!("open {}", path.display()), err))?;
            archiver.merge_compressed(file)
        }
    }
}

fn compress_file(
    item: &Planned,
    opts: &CompressOpts,
    progress: &dyn ProgressSink,
) -> Result<Piece, Error> {
    let source = item
        .source
        .as_ref()
        .ok_or_else(|| Error::Archive(format!("missing source for `{}`", item.rel)))?;
    let file =
        File::open(source).map_err(|err| Error::io(format!("open {}", source.display()), err))?;
    let mut reader = ProgressReader::new(file, progress);
    let name = rel_name(Path::new(&item.rel));
    let (method, level) = method_for(&name, opts);
    let secret = opts.password.clone();
    let options = options_for(
        &item.meta,
        method,
        level,
        secret.as_ref().map(crate::crypto::Password::expose),
    )?;
    let mut buf = Vec::new();
    if item.meta.size < MEMORY_LIMIT {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file(name, options)?;
        copy_in(&mut reader, &mut writer, &mut buf)?;
        let cursor = writer.finish()?;
        return Ok(Piece::Memory(cursor.into_inner()));
    }
    let path = piece_path(&item.rel);
    let outcome = (|| {
        let out = File::create(&path)
            .map_err(|err| Error::io(format!("create {}", path.display()), err))?;
        let mut writer = ZipWriter::new(BufWriter::new(out));
        writer.start_file(name, options)?;
        copy_in(&mut reader, &mut writer, &mut buf)?;
        let mut inner = writer.finish()?;
        inner
            .flush()
            .map_err(|err| Error::io(format!("flush {}", path.display()), err))?;
        Ok(Piece::Temp(path.clone()))
    })();
    if outcome.is_err() {
        let _ = std::fs::remove_file(&path);
    }
    outcome
}

fn piece_path(rel: &str) -> PathBuf {
    let slug: String = rel
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .take(40)
        .collect();
    std::env::temp_dir().join(format!(
        "szip-piece-{}-{}-{slug}.zip",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ))
}
