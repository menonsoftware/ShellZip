//! `szip l`.

use shellzip_core::{EntryKind, Error, format_bytes, format_time, list as list_archive};

use crate::cli::ArchiveArgs;

/// Print one line per member. Quiet flags do not hide this table.
pub fn list(args: &ArchiveArgs) -> Result<i32, Error> {
    let (opened, entries) = list_archive(&args.archive)?;
    println!(
        "{}  {}  {} entries",
        opened.detected().label(),
        args.archive.display(),
        entries.len()
    );
    println!(
        "{:<16} {:>10} {:>10} {:>6}  Name",
        "Modified", "Size", "Packed", "Ratio"
    );
    for entry in &entries {
        let name = match entry.kind {
            EntryKind::Dir if !entry.path.ends_with('/') => format!("{}/", entry.path),
            EntryKind::Symlink => match &entry.link_target {
                Some(target) => format!("{} -> {target}", entry.path),
                None => entry.path.clone(),
            },
            _ => entry.path.clone(),
        };
        let ratio = entry
            .ratio_percent()
            .map(|percent| format!("{percent}%"))
            .unwrap_or_else(|| "-".to_string());
        let size = entry
            .size
            .map(format_bytes)
            .unwrap_or_else(|| "-".to_string());
        let packed = entry
            .packed_size
            .map(format_bytes)
            .unwrap_or_else(|| "-".to_string());
        let encrypted = if entry.encrypted { " *" } else { "" };
        println!(
            "{:<16} {:>10} {:>10} {:>6}  {name}{encrypted}",
            format_time(entry.modified),
            size,
            packed,
            ratio,
        );
    }
    Ok(0)
}
