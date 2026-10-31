//! Header-only archive summary for the Info dialog.
//!
//! Member payloads are not decoded. Sizes, methods, and encryption come from
//! the container headers.

use std::collections::HashSet;
use std::path::Path;

use zip::HasZipMetadata;

use crate::detect::{Format, detect_file};
use crate::entry::{EntryKind, member_key};
use crate::error::Error;

/// 4 GiB. A size strictly above this does not fit in a classic ZIP field.
const FOUR_GIB: u64 = 4 * 1024 * 1024 * 1024;

/// Em dash used when a summary field has no single value.
const ABSENT: &str = "—";

/// Facts shown in the archive Info dialog.
///
/// Built from headers only. [`Self::packed_size`] is the sum of member packed
/// sizes, not the archive file length ([`crate::DeleteReport::packed_bytes`]
/// is the file length).
#[derive(Debug, Clone)]
pub struct ArchiveSummary {
    /// Container title.
    ///
    /// `ZIP archive`, or `ZIP64 archive` when any ZIP member has `large_file`
    /// set, an uncompressed or packed size above 4 GiB, or the archive holds
    /// more than 65535 entries. `7z archive` and `RAR archive` for those
    /// containers. A self-extractor whose payload is ZIP uses the ZIP or ZIP64
    /// title and reports the stub in [`Self::sfx_bytes`]. Any other
    /// self-extractor is `SFX archive`. Remaining containers use
    /// `{label} archive`, for example `TAR archive`.
    pub kind: String,
    /// Version line.
    ///
    /// ZIP uses `ZipFileData::version_needed`: `10` renders as `1.0`. When
    /// every file shares one method the line is `{version} / {method}` with
    /// method `Store`, `Deflate`, `Zstd`, or the method's `Debug` name. Mixed
    /// file methods use the highest version and ` / Mixed`. An empty ZIP is
    /// `—`. Other formats are the single shared file method name from the
    /// listing (`LZMA2`, `store`, `gzip`, …), or `—` when methods differ or
    /// none are recorded.
    pub version: String,
    /// Common [`crate::Entry::host_os`], `Mixed` when members disagree, or `—`.
    pub host_os: String,
    /// Explicit directories plus directories implied by member paths.
    ///
    /// `a/b/c.txt` counts `a` and `a/b`. A path is counted once.
    pub folders: u64,
    /// Members that are not directories. Symlinks count as files.
    pub files: u64,
    /// Sum of header uncompressed sizes. A missing size counts as 0.
    pub total_size: u64,
    /// Sum of header packed sizes. A missing size counts as 0.
    ///
    /// RAR listings do not record a packed size, so this stays 0 for RAR.
    pub packed_size: u64,
    /// `packed_size * 100 / total_size`, truncated. `0` when `total_size` is 0.
    pub ratio_percent: u64,
    /// Byte offset of the archive payload inside a self-extractor, otherwise 0.
    pub sfx_bytes: u64,
    /// Non-empty ZIP comment.
    ///
    /// `Some` for a ZIP archive, and for a self-extractor whose payload is ZIP,
    /// when that comment is non-empty. `None` when the comment is missing or
    /// empty, and for every other container.
    pub comment: Option<String>,
    /// `true` when any member is encrypted.
    pub encrypted: bool,
}

/// Read [`ArchiveSummary`] from `path` without decoding payloads.
///
/// # Errors
///
/// Returns [`Error::Encrypted`] when a 7z or RAR header cannot be listed
/// without a password, [`Error::Archive`] when a size or count overflows or a
/// ZIP comment is not UTF-8, and the open errors shared with [`crate::list`].
pub fn archive_summary(path: &Path) -> Result<ArchiveSummary, Error> {
    let format = detect_file(path)?;
    let zip = payload_is_zip(&format);
    let mut tally = Tally::default();
    if zip {
        tally_zip(path, &mut tally)?;
    } else {
        tally_entries(path, &mut tally)?;
    }
    let folders = u64::try_from(tally.dirs.len())
        .map_err(|_| Error::Archive("folder count overflow".to_string()))?;
    Ok(ArchiveSummary {
        kind: kind_label(&format, tally.zip64),
        version: version_line(&tally, zip),
        host_os: host_line(&tally.hosts),
        folders,
        files: tally.files,
        total_size: tally.total_size,
        packed_size: tally.packed_size,
        ratio_percent: ratio(tally.packed_size, tally.total_size)?,
        sfx_bytes: sfx_offset(&format),
        comment: comment_of(path, &format)?,
        encrypted: tally.encrypted,
    })
}

#[derive(Default)]
struct Tally {
    dirs: HashSet<String>,
    files: u64,
    total_size: u64,
    packed_size: u64,
    encrypted: bool,
    zip64: bool,
    hosts: HashSet<String>,
    file_methods: HashSet<String>,
    all_methods: HashSet<String>,
    max_version: Option<u16>,
    plain: Methods,
}

#[derive(Default)]
enum Methods {
    #[default]
    None,
    One(String),
    Mixed,
}

fn tally_zip(path: &Path, tally: &mut Tally) -> Result<(), Error> {
    let file = crate::sfx::open_section(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    if archive.len() > usize::from(u16::MAX) {
        tally.zip64 = true;
    }
    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index)?;
        let name = entry.name().to_string();
        if member_key(&name).is_empty() {
            continue;
        }
        let is_dir = entry.is_dir();
        let size = entry.size();
        let packed = entry.compressed_size();
        let encrypted = entry.encrypted();
        let method = zip_method_label(entry.compression());
        let (version, large, system) = {
            let meta = entry.get_metadata();
            (
                meta.version_needed(),
                meta.large_file,
                format!("{:?}", meta.system),
            )
        };
        if large || size > FOUR_GIB || packed > FOUR_GIB {
            tally.zip64 = true;
        }
        note_member(
            tally,
            &name,
            is_dir,
            size,
            packed,
            encrypted,
            Some(host_label(&system)),
        )?;
        tally.max_version = Some(
            tally
                .max_version
                .map_or(version, |previous| previous.max(version)),
        );
        tally.all_methods.insert(method.clone());
        if !is_dir {
            tally.file_methods.insert(method);
        }
    }
    Ok(())
}

fn tally_entries(path: &Path, tally: &mut Tally) -> Result<(), Error> {
    let (_opened, entries) = crate::ops::list_with(path, &crate::options::ListOpts::default())?;
    for entry in entries {
        let is_dir = entry.kind == EntryKind::Dir;
        note_member(
            tally,
            &entry.path,
            is_dir,
            entry.size.unwrap_or(0),
            entry.packed_size.unwrap_or(0),
            entry.encrypted,
            entry.host_os,
        )?;
        if !is_dir {
            note_method(&mut tally.plain, entry.method);
        }
    }
    Ok(())
}

fn note_member(
    tally: &mut Tally,
    name: &str,
    is_dir: bool,
    size: u64,
    packed: u64,
    encrypted: bool,
    host: Option<String>,
) -> Result<(), Error> {
    record_tree(&mut tally.dirs, &mut tally.files, name, is_dir)?;
    add_bytes(&mut tally.total_size, size)?;
    add_bytes(&mut tally.packed_size, packed)?;
    if encrypted {
        tally.encrypted = true;
    }
    if let Some(host) = host {
        tally.hosts.insert(host);
    }
    Ok(())
}

fn record_tree(
    dirs: &mut HashSet<String>,
    files: &mut u64,
    name: &str,
    is_dir: bool,
) -> Result<(), Error> {
    let key = member_key(name);
    if key.is_empty() {
        return Ok(());
    }
    if is_dir {
        dirs.insert(key.clone());
    } else {
        *files = files
            .checked_add(1)
            .ok_or_else(|| Error::Archive("entry count overflow".to_string()))?;
    }
    let mut rest = key.as_str();
    while let Some((parent, _)) = rest.rsplit_once('/') {
        dirs.insert(parent.to_string());
        rest = parent;
    }
    Ok(())
}

fn add_bytes(slot: &mut u64, extra: u64) -> Result<(), Error> {
    *slot = slot
        .checked_add(extra)
        .ok_or_else(|| Error::Archive("archive size overflow".to_string()))?;
    Ok(())
}

fn note_method(state: &mut Methods, method: Option<String>) {
    *state = match (std::mem::replace(state, Methods::None), method) {
        (_, None) | (Methods::Mixed, _) => Methods::Mixed,
        (Methods::None, Some(method)) => Methods::One(method),
        (Methods::One(previous), Some(method)) if previous == method => Methods::One(previous),
        (Methods::One(_), Some(_)) => Methods::Mixed,
    };
}

fn version_line(tally: &Tally, zip: bool) -> String {
    if !zip {
        return match &tally.plain {
            Methods::One(name) => name.clone(),
            Methods::None | Methods::Mixed => ABSENT.to_string(),
        };
    }
    let Some(version) = tally.max_version else {
        return ABSENT.to_string();
    };
    let version = format!("{}.{}", version / 10, version % 10);
    let methods = if tally.file_methods.is_empty() {
        &tally.all_methods
    } else {
        &tally.file_methods
    };
    let mut names = methods.iter();
    match (names.next(), names.next()) {
        (None, _) => version,
        (Some(name), None) => format!("{version} / {name}"),
        (Some(_), Some(_)) => format!("{version} / Mixed"),
    }
}

fn host_line(hosts: &HashSet<String>) -> String {
    let mut names = hosts.iter();
    match (names.next(), names.next()) {
        (None, _) => ABSENT.to_string(),
        (Some(name), None) => name.clone(),
        (Some(_), Some(_)) => "Mixed".to_string(),
    }
}

fn ratio(packed: u64, total: u64) -> Result<u64, Error> {
    if total == 0 {
        return Ok(0);
    }
    let scaled = packed
        .checked_mul(100)
        .ok_or_else(|| Error::Archive("packed size overflows the ratio".to_string()))?;
    Ok(scaled / total)
}

fn payload_is_zip(format: &Format) -> bool {
    match format {
        Format::Zip => true,
        Format::Sfx { inner, .. } => matches!(inner.as_ref(), Format::Zip),
        _ => false,
    }
}

fn sfx_offset(format: &Format) -> u64 {
    match format {
        Format::Sfx { offset, .. } => *offset,
        _ => 0,
    }
}

fn kind_label(format: &Format, zip64: bool) -> String {
    match format {
        Format::Zip => zip_kind(zip64),
        Format::SevenZ => "7z archive".to_string(),
        Format::Rar4 | Format::Rar5 => "RAR archive".to_string(),
        Format::Sfx { inner, .. } => match inner.as_ref() {
            Format::Zip => zip_kind(zip64),
            _ => "SFX archive".to_string(),
        },
        other => format!("{} archive", other.label()),
    }
}

fn zip_kind(zip64: bool) -> String {
    if zip64 {
        "ZIP64 archive".to_string()
    } else {
        "ZIP archive".to_string()
    }
}

fn comment_of(path: &Path, format: &Format) -> Result<Option<String>, Error> {
    match format {
        Format::Zip => {
            let text = crate::ops::zip_comment(path)?;
            Ok(present_comment(text))
        }
        Format::Sfx { inner, .. } if matches!(inner.as_ref(), Format::Zip) => {
            let file = crate::sfx::open_section(path)?;
            let archive = zip::ZipArchive::new(file)?;
            let text = String::from_utf8(archive.comment().to_vec())
                .map_err(|_| Error::Archive("ZIP comment is not UTF-8".to_string()))?;
            Ok(present_comment(text))
        }
        _ => Ok(None),
    }
}

fn present_comment(text: String) -> Option<String> {
    (!text.is_empty()).then_some(text)
}

fn zip_method_label(method: zip::CompressionMethod) -> String {
    if method == zip::CompressionMethod::Stored {
        "Store".to_string()
    } else if method == zip::CompressionMethod::Deflated {
        "Deflate".to_string()
    } else if method == zip::CompressionMethod::Zstd {
        "Zstd".to_string()
    } else {
        format!("{method:?}")
    }
}

fn host_label(system: &str) -> String {
    match system {
        "Dos" => "DOS".to_string(),
        "Unix" => "Unix".to_string(),
        other => other.to_string(),
    }
}
