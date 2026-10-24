//! `szip l` and `szip lt`.

use shellzip_core::{
    Entry, EntryKind, Error, ListOpts, Password, format_attrs, format_blake, format_bytes,
    format_crc, format_time, list_with,
};

use crate::cli::{ArchiveArgs, ListArgs};

/// Print one line per member. Quiet flags do not hide this table.
pub fn list(args: &ArchiveArgs, password: Option<&Password>) -> Result<i32, Error> {
    let (opened, entries) = list_with(
        &args.archive,
        &ListOpts {
            password: password.cloned(),
            ..ListOpts::default()
        },
    )?;
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
pub fn list_technical(args: &ListArgs, password: Option<&Password>) -> Result<i32, Error> {
    let (opened, entries) = list_with(
        &args.archive,
        &ListOpts {
            blake2sp: args.htb,
            password: password.cloned(),
        },
    )?;
    println!(
        "{}  {}  {} entries",
        opened.detected().label(),
        args.archive.display(),
        entries.len()
    );
    if args.htb {
        println!(
            "{:<24} {:>10} {:>10} {:>6}  {:<16}  {:<6}  {:<8}  {:<64}  {:<8}  {:<8}  Flags",
            "Name",
            "Size",
            "Packed",
            "Ratio",
            "Modified",
            "Attrs",
            "CRC32",
            "BLAKE2sp",
            "Method",
            "Host"
        );
    } else {
        println!(
            "{:<24} {:>10} {:>10} {:>6}  {:<16}  {:<6}  {:<8}  {:<8}  {:<8}  Flags",
            "Name", "Size", "Packed", "Ratio", "Modified", "Attrs", "CRC32", "Method", "Host"
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
                "{name:<24} {size:>10} {packed:>10} {ratio:>6}  {:<16}  {:<6}  {:<8}  {:<64}  {method:<8}  {host:<8}  {}",
                format_time(entry.modified),
                format_attrs(entry.kind, entry.mode),
                format_crc(entry.crc32),
                format_blake(entry.blake2sp.as_ref()),
                flags_text(entry),
            );
        } else {
            println!(
                "{name:<24} {size:>10} {packed:>10} {ratio:>6}  {:<16}  {:<6}  {:<8}  {method:<8}  {host:<8}  {}",
                format_time(entry.modified),
                format_attrs(entry.kind, entry.mode),
                format_crc(entry.crc32),
                flags_text(entry),
            );
        }
    }
    Ok(0)
}

fn flags_text(entry: &Entry) -> String {
    let mut parts = Vec::new();
    if entry.encrypted {
        parts.push("encrypted".to_string());
    }
    if entry.solid {
        parts.push("solid".to_string());
    }
    if let Some(dictionary) = entry.dictionary {
        parts.push(format!("dict={dictionary}"));
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(",")
    }
}

fn display_name(entry: &Entry) -> String {
    match entry.kind {
        EntryKind::Dir if !entry.path.ends_with('/') => format!("{}/", entry.path),
        EntryKind::Symlink => match &entry.link_target {
            Some(target) => format!("{} -> {target}", entry.path),
            None => entry.path.clone(),
        },
        _ => entry.path.clone(),
    }
}
