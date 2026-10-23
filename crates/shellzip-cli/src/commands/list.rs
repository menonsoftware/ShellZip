//! `szip l` and `szip lt`.

use shellzip_core::{
    EntryKind, Error, ListOpts, format_attrs, format_blake, format_bytes, format_crc, format_time,
    list as list_archive, list_with,
};

use crate::cli::{ArchiveArgs, ListArgs};

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

/// Technical listing. `-htb` adds a BLAKE2sp column and reads every file payload.
pub fn list_technical(args: &ListArgs) -> Result<i32, Error> {
    let (opened, entries) = list_with(&args.archive, &ListOpts { blake2sp: args.htb })?;
    println!(
        "{}  {}  {} entries",
        opened.detected().label(),
        args.archive.display(),
        entries.len()
    );
    if args.htb {
        println!(
            "{:<24} {:>10} {:>10} {:>6}  {:<16}  {:<6}  {:<8}  {:<64}  {:<8}  Host",
            "Name", "Size", "Packed", "Ratio", "Modified", "Attrs", "CRC32", "BLAKE2sp", "Method"
        );
    } else {
        println!(
            "{:<24} {:>10} {:>10} {:>6}  {:<16}  {:<6}  {:<8}  {:<8}  Host",
            "Name", "Size", "Packed", "Ratio", "Modified", "Attrs", "CRC32", "Method"
        );
    }
    for entry in &entries {
        let name = display_name(entry);
        let ratio = entry
            .ratio_percent()
            .map(|percent| format!("{percent}%"))
            .unwrap_or_else(|| "-".to_string());
        let size = entry
            .size
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".into());
        let packed = entry
            .packed_size
            .map(|n| n.to_string())
            .unwrap_or_else(|| "-".into());
        let method = entry.method.clone().unwrap_or_else(|| "-".into());
        let host = entry.host_os.clone().unwrap_or_else(|| "-".into());
        if args.htb {
            println!(
                "{name:<24} {size:>10} {packed:>10} {ratio:>6}  {:<16}  {:<6}  {:<8}  {:<64}  {method:<8}  {host}",
                format_time(entry.modified),
                format_attrs(entry.kind, entry.mode),
                format_crc(entry.crc32),
                format_blake(entry.blake2sp.as_ref()),
            );
        } else {
            println!(
                "{name:<24} {size:>10} {packed:>10} {ratio:>6}  {:<16}  {:<6}  {:<8}  {method:<8}  {host}",
                format_time(entry.modified),
                format_attrs(entry.kind, entry.mode),
                format_crc(entry.crc32),
            );
        }
    }
    Ok(0)
}

fn display_name(entry: &shellzip_core::Entry) -> String {
    match entry.kind {
        EntryKind::Dir if !entry.path.ends_with('/') => format!("{}/", entry.path),
        EntryKind::Symlink => match &entry.link_target {
            Some(target) => format!("{} -> {target}", entry.path),
            None => entry.path.clone(),
        },
        _ => entry.path.clone(),
    }
}
