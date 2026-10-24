//! Copy members that an update is keeping into the new archive.
//!
//! Each payload is extracted on one thread and compressed on the other. The
//! pipe between them is the only buffer, so a kept member is not held in memory.

use std::collections::HashSet;

use super::{SevenZArchiver, archive_path, seven_meta};
use crate::entry::{EntryMeta, member_key};
use crate::error::{Error, map_zesven};
use crate::extractor::sevenz::open_streaming;

/// Re-add every member of `source` that this run is not replacing.
pub(super) fn keep(
    archiver: &mut SevenZArchiver,
    source: &std::path::Path,
    replacing: &[String],
) -> Result<(), Error> {
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
    let mut iter = archive
        .entries()
        .map_err(|err| map_zesven(err, had_password))?;
    while let Some(item) = iter.next() {
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
        if member.is_directory {
            let path = archive_path(std::path::Path::new(&member.name))?;
            let meta = seven_meta(&meta_for(&member), true, false);
            archiver
                .writer()?
                .add_directory(path, meta)
                .map_err(|err| map_zesven(err, had_password))?;
            continue;
        }
        let path = archive_path(std::path::Path::new(&member.name))?;
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
                out.add_stream(path, &mut reader, meta)
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
    }
    Ok(())
}

fn meta_for(member: &crate::extractor::sevenz::Member) -> EntryMeta {
    EntryMeta {
        size: member.size,
        modified: member.modified,
        mode: if member.is_symlink { 0o120777 } else { 0o644 },
        size_known: true,
    }
}
