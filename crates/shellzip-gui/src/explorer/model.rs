//! Rows, archive folder index, sort, and the address-bar path helpers.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use shellzip_core::{Entry, EntryKind, format_from_extension, format_time};

/// Stable key for the parent row. It cannot collide with a real path.
pub const PARENT_KEY: &str = "\u{1}..";

/// What a row does when opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortColumn {
    /// File name.
    #[default]
    Name,
    /// Uncompressed size.
    Size,
    /// Type label.
    Type,
    /// Modification time.
    Modified,
    /// Compressed size.
    Packed,
    /// CRC32.
    Crc,
}

/// Header click direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// A to Z, small to large, older to newer.
    #[default]
    Asc,
    /// The reverse of [`SortOrder::Asc`].
    Desc,
}

/// How the row is drawn and activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// `..`.
    Parent,
    /// A folder on disk or inside an archive.
    Directory,
    /// A regular file.
    File,
    /// A file this build can list.
    Archive,
    /// A symbolic link that is not itself a folder.
    Symlink,
}

/// Where activation should go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Parent of the current location.
    Parent,
    /// A filesystem path.
    Path(PathBuf),
    /// A folder inside the open archive. No trailing slash.
    ArchiveDir(String),
    /// One archive member path, as stored.
    ArchiveMember(String),
}

/// One visible line in the file pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Label in the name column.
    pub name: String,
    /// Icon and sort group.
    pub kind: RowKind,
    /// Byte size. Folders leave this empty.
    pub size: Option<u64>,
    /// Compressed size, when the archive records one.
    pub packed: Option<u64>,
    /// Modification time, when known.
    pub modified: Option<SystemTime>,
    /// CRC32 of a file payload, when the archive records one.
    pub crc32: Option<u32>,
    /// Type column.
    pub type_label: String,
    /// Selection identity.
    pub key: String,
    /// Double-click target.
    pub target: Target,
}

impl Row {
    /// The `..` row. It stays above every sort.
    pub fn parent() -> Self {
        Self {
            name: "..".to_string(),
            kind: RowKind::Parent,
            size: None,
            packed: None,
            modified: None,
            crc32: None,
            type_label: "File folder".to_string(),
            key: PARENT_KEY.to_string(),
            target: Target::Parent,
        }
    }
}

/// One entry in the address dropdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jump {
    /// Full path shown in the menu.
    pub label: String,
    /// Where the menu item navigates.
    pub target: JumpTarget,
}

/// A dropdown destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpTarget {
    /// A real directory, or a directory that contains the open archive.
    Folder(PathBuf),
    /// A folder inside the open archive. Empty is the archive root.
    Prefix(String),
}

/// `..` when `path` has a parent. Filesystem roots return `None`.
pub fn folder_parent(path: &Path) -> Option<&Path> {
    let parent = path.parent()?;
    if parent.as_os_str().is_empty() {
        None
    } else {
        Some(parent)
    }
}

/// Extension names this build can list, test, and extract.
pub fn is_supported_archive(path: &Path) -> bool {
    format_from_extension(path).is_ok_and(|format| format.is_supported())
}

/// Address text. `prefix` is an archive-relative folder using `/`.
pub fn location_text(path: &Path, prefix: Option<&str>) -> String {
    let mut text = path.display().to_string();
    let Some(prefix) = prefix.filter(|value| !value.is_empty()) else {
        return text;
    };
    for part in prefix.split('/').filter(|part| !part.is_empty()) {
        if !text.ends_with(std::path::MAIN_SEPARATOR) {
            text.push(std::path::MAIN_SEPARATOR);
        }
        text.push_str(part);
    }
    text
}

/// Archive-relative folder when `typed` names `archive` or a folder inside it.
///
/// `None` means `typed` is a different path. An empty string is the archive root.
pub fn archive_prefix(typed: &str, archive: &Path) -> Option<String> {
    let typed = typed.trim();
    let archive_text = archive.display().to_string();
    let typed_norm = normalize_sep(typed);
    let archive_norm = normalize_sep(&archive_text);
    if typed_norm.len() < archive_norm.len() {
        return None;
    }
    if !typed_norm[..archive_norm.len()].eq_ignore_ascii_case(&archive_norm) {
        return None;
    }
    let tail = &typed_norm[archive_norm.len()..];
    if !tail.is_empty() && !tail.starts_with(['/', '\\']) {
        return None;
    }
    Some(
        tail.trim_start_matches(['/', '\\'])
            .replace('\\', "/")
            .trim_matches('/')
            .to_string(),
    )
}

/// Ancestors of a folder, or of an open archive plus its internal prefix.
pub fn jumps(path: &Path, archive_prefix: Option<&str>) -> Vec<Jump> {
    let chain = folder_chain(path);
    let mut out = Vec::new();
    if let Some(prefix) = archive_prefix {
        let archive_at = chain.len().saturating_sub(1);
        for (index, ancestor) in chain.iter().enumerate() {
            if index == archive_at {
                out.push(Jump {
                    label: location_text(ancestor, Some("")),
                    target: JumpTarget::Prefix(String::new()),
                });
            } else {
                out.push(Jump {
                    label: ancestor.display().to_string(),
                    target: JumpTarget::Folder(ancestor.clone()),
                });
            }
        }
        let mut acc = String::new();
        for part in prefix.split('/').filter(|part| !part.is_empty()) {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(part);
            out.push(Jump {
                label: location_text(path, Some(&acc)),
                target: JumpTarget::Prefix(acc.clone()),
            });
        }
    } else {
        for ancestor in chain {
            out.push(Jump {
                label: ancestor.display().to_string(),
                target: JumpTarget::Folder(ancestor),
            });
        }
    }
    out
}

/// True when `path` is `prefix` or a member stored under it.
pub fn path_under(path: &str, prefix: &str) -> bool {
    let path = path.trim_end_matches(['/', '\\']);
    if prefix.is_empty() {
        return true;
    }
    if path == prefix {
        return true;
    }
    path.strip_prefix(prefix)
        .is_some_and(|rest| rest.starts_with('/'))
}

/// Folder count, file count, and total file bytes. `..` is not counted.
pub fn totals(rows: &[Row]) -> (usize, usize, u64) {
    let mut folders = 0;
    let mut files = 0;
    let mut bytes = 0;
    for row in rows {
        match row.kind {
            RowKind::Parent => {}
            RowKind::Directory => folders += 1,
            RowKind::File | RowKind::Archive | RowKind::Symlink => {
                files += 1;
                bytes += row.size.unwrap_or(0);
            }
        }
    }
    (folders, files, bytes)
}

/// Byte count in 1024-based units labeled B, KB, MB, GB, and TB.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Status sentence, for example `Total 1 folder and 912 bytes in 2 files`.
pub fn totals_label(folders: usize, files: usize, bytes: u64) -> String {
    format!(
        "Total {} and {} in {}",
        counted(folders, "folder", "folders"),
        byte_phrase(bytes),
        counted(files, "file", "files"),
    )
}

/// Local `DD-MM-YYYY HH:MM`, matching the file pane.
pub fn format_modified(time: SystemTime) -> String {
    let text = format_time(Some(time));
    let Some((date, clock)) = text.split_once(' ') else {
        return text;
    };
    let mut parts = date.split('-');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(year), Some(month), Some(day)) => format!("{day}-{month}-{year} {clock}"),
        _ => text,
    }
}

/// Parent first, then folders, then files. The column orders each group.
pub fn sort_rows(rows: &mut [Row], column: SortColumn, order: SortOrder) {
    rows.sort_by(|left, right| {
        let rank = rank(left.kind).cmp(&rank(right.kind));
        if rank != Ordering::Equal {
            return rank;
        }
        let mut compared = compare_column(left, right, column);
        if order == SortOrder::Desc {
            compared = compared.reverse();
        }
        if compared == Ordering::Equal {
            compare_name(&left.name, &right.name)
        } else {
            compared
        }
    });
}

/// Precomputed children of every archive folder.
#[derive(Debug, Default)]
pub struct ArchiveIndex {
    folders: HashMap<String, Vec<Row>>,
}

impl ArchiveIndex {
    /// Index `entries` in one pass. Folder keys have no leading or trailing slash.
    pub fn build(entries: &[Entry]) -> Self {
        let mut buckets: HashMap<String, HashMap<String, Partial>> = HashMap::new();
        buckets.entry(String::new()).or_default();
        for entry in entries {
            insert(&mut buckets, entry);
        }
        let folders = buckets
            .into_iter()
            .map(|(prefix, children)| {
                let rows = children.into_values().map(Partial::into_row).collect();
                (prefix, rows)
            })
            .collect();
        Self { folders }
    }

    /// Immediate children of `prefix`. Unknown folders are empty.
    pub fn children(&self, prefix: &str) -> &[Row] {
        self.folders.get(prefix).map(Vec::as_slice).unwrap_or(&[])
    }

    /// `prefix` is empty or names a folder created while indexing.
    pub fn contains_folder(&self, prefix: &str) -> bool {
        prefix.is_empty() || self.folders.contains_key(prefix)
    }
}

struct Partial {
    name: String,
    kind: RowKind,
    size: Option<u64>,
    packed: Option<u64>,
    modified: Option<SystemTime>,
    crc32: Option<u32>,
    archive_path: String,
}

impl Partial {
    fn merge(&mut self, other: Self) {
        let directory = self.kind == RowKind::Directory || other.kind == RowKind::Directory;
        if directory {
            self.kind = RowKind::Directory;
            self.size = None;
            self.packed = None;
            self.crc32 = None;
            if other.modified.is_some() {
                self.modified = other.modified;
            }
            if other.kind == RowKind::Directory {
                self.archive_path = other.archive_path;
            }
            return;
        }
        self.kind = other.kind;
        if other.size.is_some() {
            self.size = other.size;
        }
        if other.packed.is_some() {
            self.packed = other.packed;
        }
        if other.crc32.is_some() {
            self.crc32 = other.crc32;
        }
        if other.modified.is_some() {
            self.modified = other.modified;
        }
        self.archive_path = other.archive_path;
    }

    fn into_row(self) -> Row {
        let type_label = type_label(&self.name, self.kind);
        let target = if self.kind == RowKind::Directory {
            Target::ArchiveDir(self.archive_path.clone())
        } else {
            Target::ArchiveMember(self.archive_path.clone())
        };
        Row {
            name: self.name,
            kind: self.kind,
            size: self.size,
            packed: self.packed,
            modified: self.modified,
            crc32: self.crc32,
            type_label,
            key: self.archive_path,
            target,
        }
    }
}

fn insert(buckets: &mut HashMap<String, HashMap<String, Partial>>, entry: &Entry) {
    let trimmed = entry.path.trim_matches('/');
    if trimmed.is_empty() {
        return;
    }
    let parts: Vec<&str> = trimmed.split('/').filter(|part| !part.is_empty()).collect();
    if parts.is_empty() {
        return;
    }
    let last = parts.len() - 1;
    let mut prefix = String::new();
    for (index, part) in parts.iter().enumerate() {
        let child_path = if prefix.is_empty() {
            (*part).to_string()
        } else {
            format!("{prefix}/{part}")
        };
        let is_dir = index < last || entry.kind == EntryKind::Dir;
        let partial = if is_dir {
            let modified = if index == last { entry.modified } else { None };
            buckets.entry(child_path.clone()).or_default();
            dir_partial(part, child_path, modified)
        } else {
            file_partial(part, child_path, entry)
        };
        upsert(buckets.entry(prefix.clone()).or_default(), partial);
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
    }
}

fn upsert(bucket: &mut HashMap<String, Partial>, partial: Partial) {
    if let Some(existing) = bucket.get_mut(&partial.name) {
        existing.merge(partial);
    } else {
        bucket.insert(partial.name.clone(), partial);
    }
}

fn dir_partial(name: &str, archive_path: String, modified: Option<SystemTime>) -> Partial {
    Partial {
        name: name.to_string(),
        kind: RowKind::Directory,
        size: None,
        packed: None,
        modified,
        crc32: None,
        archive_path,
    }
}

fn file_partial(name: &str, archive_path: String, entry: &Entry) -> Partial {
    let kind = match entry.kind {
        EntryKind::Symlink => RowKind::Symlink,
        EntryKind::Dir => RowKind::Directory,
        EntryKind::File | EntryKind::Other => {
            if is_supported_archive(Path::new(name)) {
                RowKind::Archive
            } else {
                RowKind::File
            }
        }
    };
    Partial {
        name: name.to_string(),
        kind,
        size: entry.size,
        packed: entry.packed_size,
        modified: entry.modified,
        crc32: entry.crc32,
        archive_path,
    }
}

pub(crate) fn type_label(name: &str, kind: RowKind) -> String {
    match kind {
        RowKind::Parent | RowKind::Directory => "File folder".to_string(),
        RowKind::Symlink => "Symbolic link".to_string(),
        RowKind::Archive | RowKind::File => file_type_label(name),
    }
}

fn file_type_label(name: &str) -> String {
    if let Ok(format) = format_from_extension(Path::new(name)) {
        return format!("{} archive", format.label());
    }
    match extension(name).as_deref() {
        Some("txt") => "Text Document".to_string(),
        Some("ini") => "Configuration settings".to_string(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp") => "Image".to_string(),
        Some("md") => "Markdown document".to_string(),
        Some("json") => "JSON document".to_string(),
        Some("xml") => "XML document".to_string(),
        Some("pdf") => "PDF document".to_string(),
        Some("rs") => "Rust source".to_string(),
        Some("exe" | "dll") => "Application".to_string(),
        Some(ext) => format!("{} file", ext.to_ascii_uppercase()),
        None => "File".to_string(),
    }
}

fn extension(name: &str) -> Option<String> {
    let file = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let (_, ext) = file.rsplit_once('.')?;
    if ext.is_empty() {
        None
    } else {
        Some(ext.to_ascii_lowercase())
    }
}

fn rank(kind: RowKind) -> u8 {
    match kind {
        RowKind::Parent => 0,
        RowKind::Directory => 1,
        RowKind::File | RowKind::Archive | RowKind::Symlink => 2,
    }
}

fn compare_column(left: &Row, right: &Row, column: SortColumn) -> Ordering {
    match column {
        SortColumn::Name => compare_name(&left.name, &right.name),
        SortColumn::Size => compare_option(left.size, right.size),
        SortColumn::Type => compare_name(&left.type_label, &right.type_label),
        SortColumn::Modified => compare_option(left.modified, right.modified),
        SortColumn::Packed => compare_option(left.packed, right.packed),
        SortColumn::Crc => compare_option(left.crc32, right.crc32),
    }
}

fn compare_name(left: &str, right: &str) -> Ordering {
    left.to_lowercase().cmp(&right.to_lowercase())
}

fn compare_option<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn counted(count: usize, one: &str, many: &str) -> String {
    let word = if count == 1 { one } else { many };
    format!("{count} {word}")
}

fn byte_phrase(bytes: u64) -> String {
    if bytes < 1024 {
        let unit = if bytes == 1 { "byte" } else { "bytes" };
        format!("{bytes} {unit}")
    } else {
        format_size(bytes)
    }
}

fn folder_chain(path: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = path.to_path_buf();
    loop {
        chain.push(current.clone());
        let Some(parent) = folder_parent(&current).map(Path::to_path_buf) else {
            break;
        };
        if parent == current {
            break;
        }
        current = parent;
    }
    chain.reverse();
    chain
}

fn normalize_sep(text: &str) -> String {
    if cfg!(windows) {
        text.replace('/', "\\")
    } else {
        text.replace('\\', "/")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArchiveIndex, Row, RowKind, SortColumn, SortOrder, Target, format_size, path_under,
        sort_rows, totals, totals_label,
    };
    use shellzip_core::{Entry, EntryKind};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn entry(path: &str, kind: EntryKind, size: Option<u64>) -> Entry {
        Entry {
            path: path.to_string(),
            kind,
            size,
            packed_size: None,
            modified: None,
            mode: None,
            crc32: None,
            method: None,
            host_os: None,
            blake2sp: None,
            link_target: None,
            encrypted: false,
            solid: false,
            dictionary: None,
        }
    }

    fn named(name: &str, kind: RowKind, size: Option<u64>) -> Row {
        Row {
            name: name.to_string(),
            kind,
            size,
            packed: None,
            modified: None,
            crc32: None,
            type_label: String::new(),
            key: name.to_string(),
            target: Target::Parent,
        }
    }

    #[test]
    fn index_builds_nested_folders() {
        let index = ArchiveIndex::build(&[
            entry("readme.txt", EntryKind::File, Some(10)),
            entry("docs/guide.md", EntryKind::File, Some(20)),
            entry("docs/img/", EntryKind::Dir, None),
            entry("src/main.rs", EntryKind::File, Some(30)),
        ]);
        let root: Vec<_> = index
            .children("")
            .iter()
            .map(|row| row.name.as_str())
            .collect();
        assert!(root.contains(&"readme.txt"));
        assert!(root.contains(&"docs"));
        assert!(root.contains(&"src"));
        assert!(index.contains_folder("docs/img"));
        assert_eq!(index.children("docs").len(), 2);
        assert!(
            index
                .children("docs")
                .iter()
                .any(|row| row.name == "guide.md" && row.size == Some(20))
        );
    }

    #[test]
    fn directory_marker_hides_its_own_size() {
        let index = ArchiveIndex::build(&[entry("empty/", EntryKind::Dir, Some(4))]);
        let row = index
            .children("")
            .iter()
            .find(|row| row.name == "empty")
            .expect("folder");
        assert_eq!(row.kind, RowKind::Directory);
        assert_eq!(row.size, None);
        assert!(matches!(row.target, Target::ArchiveDir(ref path) if path == "empty"));
    }

    #[test]
    fn path_under_matches_a_folder_and_its_members() {
        assert!(path_under("docs/a.txt", "docs"));
        assert!(path_under("docs/", "docs"));
        assert!(!path_under("docs2/a.txt", "docs"));
        assert!(path_under("docs/a.txt", ""));
    }

    #[test]
    fn sort_keeps_parent_and_folders_above_files() {
        let mut rows = vec![
            named("zeta.txt", RowKind::File, Some(5)),
            named("..", RowKind::Parent, None),
            named("alpha", RowKind::Directory, None),
            named("mid.txt", RowKind::File, Some(9)),
        ];
        sort_rows(&mut rows, SortColumn::Size, SortOrder::Desc);
        let names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, ["..", "alpha", "mid.txt", "zeta.txt"]);
    }

    #[test]
    fn totals_match_the_status_sentence() {
        let rows = [
            Row::parent(),
            named("talos", RowKind::Directory, None),
            named("desktop.ini", RowKind::File, Some(282)),
            named("prompt.txt", RowKind::File, Some(630)),
        ];
        let (folders, files, bytes) = totals(&rows);
        assert_eq!(
            totals_label(folders, files, bytes),
            "Total 1 folder and 912 bytes in 2 files"
        );
    }

    #[test]
    fn format_size_picks_kb_mb_and_gb() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(282), "282 B");
        assert_eq!(format_size(2421), "2.4 KB");
        assert_eq!(format_size(1_121_928), "1.1 MB");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn archive_prefix_reads_the_folder_inside() {
        let archive = PathBuf::from("docs").join("demo.zip");
        let typed = format!("{}{}notes", archive.display(), std::path::MAIN_SEPARATOR);
        assert_eq!(
            super::archive_prefix(&typed, &archive).as_deref(),
            Some("notes")
        );
        assert_eq!(
            super::archive_prefix(&archive.display().to_string(), &archive).as_deref(),
            Some("")
        );
        let other = format!("{}{}", archive.display(), ".bak");
        assert_eq!(super::archive_prefix(&other, &archive), None);
    }

    #[test]
    fn format_modified_is_day_month_year() {
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(0);
        let text = super::format_modified(time);
        assert!(text.contains('-'));
        assert_ne!(text, "-");
    }
}
