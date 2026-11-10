//! Parallel ZIP compression.
//!
//! Each worker compresses one entry into its own zip piece (in memory under
//! 1 MiB, otherwise a temporary file kept open until that piece is merged).
//! The main thread merges those pieces in input order with `ZipWriter::raw_copy_file`,
//! so the archive layout matches the single-thread writer. At most `--threads`
//! pieces exist at once.

use std::fs::{File, OpenOptions};
use std::io::{Cursor, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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

struct Spilled {
    file: File,
    path: PathBuf,
}

enum Piece {
    Memory(Vec<u8>),
    Temp(Spilled),
}

impl Drop for Piece {
    fn drop(&mut self) {
        if let Self::Temp(spilled) = self {
            // Unlink while `file` is still open. Closing first lets a scanner
            // remove the temp zip, and the merge then fails with os error 2.
            let _ = std::fs::remove_file(&spilled.path);
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
                    let mut piece = piece.ok_or_else(|| {
                        Error::Archive(format!("missing compressed piece for `{}`", item.rel))
                    })?;
                    merge(&mut archiver, &mut piece)?;
                }
                EntryKind::Other => {}
            }
            progress.end_entry();
        }
        offset = end;
    }

    super::ArchiveWriter::finish(Box::new(archiver))
}

fn merge(archiver: &mut ZipArchiver, piece: &mut Piece) -> Result<(), Error> {
    match piece {
        Piece::Memory(bytes) => archiver.merge_compressed(Cursor::new(bytes.as_slice())),
        Piece::Temp(spilled) => {
            spilled
                .file
                .seek(SeekFrom::Start(0))
                .map_err(|err| Error::io(format!("rewind {}", spilled.path.display()), err))?;
            archiver.merge_compressed(&mut spilled.file)
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
    let built = (|| -> Result<File, Error> {
        // ZipWriter seeks back to patch local headers. Do not wrap the piece
        // file in BufWriter — seeking through a buffer corrupts large deflate
        // members and later surfaces as "incomplete deflate stream".
        let out = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|err| Error::io(format!("create {}", path.display()), err))?;
        let mut writer = ZipWriter::new(out);
        writer.start_file(name, options)?;
        copy_in(&mut reader, &mut writer, &mut buf)?;
        let mut file = writer.finish()?;
        file.flush()
            .map_err(|err| Error::io(format!("flush {}", path.display()), err))?;
        file.seek(SeekFrom::Start(0))
            .map_err(|err| Error::io(format!("rewind {}", path.display()), err))?;
        Ok(file)
    })();
    match built {
        Ok(file) => Ok(Piece::Temp(Spilled { file, path })),
        Err(err) => {
            let _ = std::fs::remove_file(&path);
            Err(err)
        }
    }
}

fn piece_path(rel: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let mapped: String = rel
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect();
    // The tail, not the shared folder prefix. `take(40)` made every file under
    // `drivers/M2V0D17D/Packages/Drivers/Display…` use one slug.
    let start = mapped.len().saturating_sub(40);
    let slug = &mapped[start..];
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "szip-piece-{}-{nanos}-{id}-{slug}.zip",
        std::process::id(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::zip::ZipArchiver;
    use super::{ArchiveWriter, Piece, compress_file, merge};
    use crate::entry::{EntryKind, EntryMeta};
    use crate::fs::walk::Planned;
    use crate::options::CompressOpts;
    use crate::progress::NopProgress;

    #[test]
    fn spilled_piece_survives_removal_of_its_temp_path() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("nvlddmkm.sys");
        let mut bytes = vec![0u8; 1024 * 1024];
        bytes[0] = b'D';
        std::fs::write(&source, &bytes).unwrap();
        let item = Planned {
            source: Some(source),
            rel: "drivers/M2V0D17D/Packages/Drivers/Display.Driver/nvlddmkm.sys".into(),
            kind: EntryKind::File,
            meta: EntryMeta {
                size: bytes.len() as u64,
                modified: None,
                mode: 0o644,
                size_known: true,
            },
            link_target: None,
        };
        let opts = CompressOpts {
            level: 0,
            ..CompressOpts::default()
        };
        let mut piece = compress_file(&item, &opts, &NopProgress).unwrap();
        let Piece::Temp(spilled) = &piece else {
            panic!("a 1 MiB file should spill to a temp piece");
        };
        std::fs::remove_file(&spilled.path).unwrap();
        let dest = tmp.path().join("out.zip");
        let mut archiver = ZipArchiver::create(&dest, &opts).unwrap();
        merge(&mut archiver, &mut piece).unwrap();
        ArchiveWriter::finish(Box::new(archiver)).unwrap();

        let out = tmp.path().join("out");
        std::fs::create_dir(&out).unwrap();
        let mut extract_opts = crate::ExtractOpts::new(&out);
        extract_opts.overwrite = crate::Overwrite::Always;
        crate::extract(&dest, &extract_opts, None, &NopProgress).unwrap();
        let restored = std::fs::read(
            out.join("drivers/M2V0D17D/Packages/Drivers/Display.Driver/nvlddmkm.sys"),
        )
        .unwrap();
        assert_eq!(restored, bytes);
    }
}
