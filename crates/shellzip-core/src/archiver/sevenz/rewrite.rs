//! Copy members that an update is keeping into the new archive.
//!
//! Each payload is extracted on one thread and compressed on the other. The
//! pipe between them is the only buffer, so a kept member is not held in memory.
//! A rename map changes names on that same stream. Payloads are compressed
//! again, which is how every 7z update works.

use std::collections::{HashMap, HashSet};

use zesven::ArchivePath;

use super::{SevenZArchiver, archive_path, seven_meta};
use crate::entry::{EntryMeta, member_key};
use crate::error::{Error, map_zesven};
use crate::extractor::sevenz::open_streaming;
use crate::progress::{ProgressReader, ProgressSink, ensure_running};

/// Re-add every member of `source` that this run is not replacing.
pub(super) fn keep(
    archiver: &mut SevenZArchiver,
    source: &std::path::Path,
    replacing: &[String],
) -> Result<(), Error> {
    keep_mapped(
        archiver,
        source,
        replacing,
        &HashMap::new(),
        &crate::progress::NopProgress,
    )
    .map(|_| ())
}

/// [`keep`] with a member-key rename map and cancellation checks.
pub(super) fn keep_mapped(
    archiver: &mut SevenZArchiver,
    source: &std::path::Path,
    replacing: &[String],
    rename: &HashMap<String, String>,
    progress: &dyn ProgressSink,
) -> Result<u64, Error> {
    let had_password = archiver.opts.password.is_some();
    let mut archive = open_streaming(source, archiver.opts.password.as_ref())?;
    let replacing: HashSet<String> = replacing.iter().map(|name| member_key(name)).collect();
    let sync_keep = (archiver.opts.update == crate::options::UpdateMode::Sync)
        .then_some(archiver.opts.sync_keep.as_ref())
        .flatten()
        .map(|names| {
            names
                .iter()
                .map(|name| member_key(name))
                .collect::<HashSet<_>>()
        });
    let mut kept = 0u64;
    let mut iter = archive
        .entries()
        .map_err(|err| map_zesven(err, had_password))?;
    while let Some(item) = iter.next() {
        ensure_running(progress)?;
        let member = {
            let streamed = item.map_err(|err| map_zesven(err, had_password))?;
            crate::extractor::sevenz::Member::from_entry(streamed.entry())
        };
        if member.name.is_empty() {
            continue;
        }
        let key = member_key(&member.name);
        if replacing.contains(&key) {
            continue;
        }
        if sync_keep.as_ref().is_some_and(|keep| !keep.contains(&key)) {
            continue;
        }
        let path = mapped_path(&member.name, rename)?;
        if member.is_directory {
            let meta = seven_meta(&meta_for(&member), true, false);
            archiver
                .writer()?
                .add_directory(path, meta)
                .map_err(|err| map_zesven(err, had_password))?;
            kept = count_kept(kept)?;
            continue;
        }
        let meta = seven_meta(&meta_for(&member), false, member.is_symlink);
        let (mut reader, mut writer) =
            std::io::pipe().map_err(|err| Error::io("open 7z rewrite pipe", err))?;
        // The pipe writer has to be dropped when extraction finishes, or the
        // compressor reads forever. The iterator stays with this loop.
        let extracted = std::thread::scope(|scope| {
            let reading = &mut iter;
            let producer = scope.spawn(move || {
                reading
                    .extract_current_to(&mut writer)
                    .map(|_| ())
                    .map_err(|err| map_zesven(err, had_password))
            });
            let consumed = archiver.writer().and_then(|out| {
                let mut progress_reader = ProgressReader::new(&mut reader, progress);
                out.add_stream(path, &mut progress_reader, meta)
                    .map_err(|err| map_zesven(err, had_password))
            });
            drop(reader);
            let produced = producer.join().unwrap_or_else(|_| {
                Err(Error::Archive(
                    "7z rewrite stopped unexpectedly".to_string(),
                ))
            });
            match consumed {
                Err(err) => Err(err),
                Ok(()) => produced,
            }
        });
        extracted?;
        kept = count_kept(kept)?;
    }
    Ok(kept)
}

fn mapped_path(original: &str, rename: &HashMap<String, String>) -> Result<ArchivePath, Error> {
    let key = member_key(original);
    let name = if let Some(mapped) = rename.get(&key) {
        mapped.as_str()
    } else {
        original
    };
    archive_path(std::path::Path::new(name))
}

fn count_kept(kept: u64) -> Result<u64, Error> {
    kept.checked_add(1)
        .ok_or_else(|| Error::Archive("entry count overflow".to_string()))
}

fn meta_for(member: &crate::extractor::sevenz::Member) -> EntryMeta {
    EntryMeta {
        size: member.size,
        modified: member.modified,
        mode: if member.is_symlink { 0o120777 } else { 0o644 },
        size_known: true,
    }
}
