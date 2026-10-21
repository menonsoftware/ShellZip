//! Options for creating and extracting archives.

use std::path::{Path, PathBuf};

use crate::error::Error;

/// Called when an extracted file already exists.
pub type ConflictDecider<'a> = dyn Fn(&Path) -> Result<ConflictChoice, Error> + 'a;

/// ZIP compression method selected with `--method`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZipMethod {
    /// Deflate levels mapped from `-m1`..`-m5`.
    #[default]
    Deflate,
    /// Zstd levels mapped from `-m1`..`-m5`.
    Zstd,
}

/// What to do when the destination file already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overwrite {
    /// Ask the front end. Without a callback the file is skipped.
    #[default]
    Ask,
    /// Replace the existing file.
    Always,
    /// Leave the existing file in place.
    Never,
    /// Write `name (n).ext` beside the existing file.
    Rename,
}

/// Extensions stored uncompressed inside a ZIP, matching WinRAR's default `-ms` list.
pub const DEFAULT_STORE_EXTS: &[&str] = &[
    "7z", "ace", "arj", "bz2", "cab", "gz", "jpeg", "jpg", "lha", "lz", "lzh", "mp3", "rar", "taz",
    "tbz2", "tgz", "txz", "zst", "xz", "z", "zip", "zipx", "tzst",
];

/// How ZIP picks Stored versus a compressing method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreList {
    /// Lowercase extensions without a leading dot. Empty disables the list.
    pub extensions: Vec<String>,
}

impl Default for StoreList {
    fn default() -> Self {
        Self {
            extensions: DEFAULT_STORE_EXTS
                .iter()
                .map(|ext| (*ext).to_string())
                .collect(),
        }
    }
}

impl StoreList {
    /// Replace the list from a semicolon-separated string such as `jpg;zip`.
    pub fn parse(text: &str) -> Self {
        let extensions = text
            .split([';', ','])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.trim_start_matches('.').to_ascii_lowercase())
            .collect();
        Self { extensions }
    }

    /// Whether `name` should be stored instead of compressed.
    pub fn stores(&self, name: &str) -> bool {
        let ext = name
            .rsplit(['/', '\\'])
            .next()
            .and_then(|file| file.rsplit_once('.'))
            .map(|(_, ext)| ext.to_ascii_lowercase());
        ext.is_some_and(|ext| self.extensions.iter().any(|item| item == &ext))
    }
}

/// Settings for `szip a`.
#[derive(Debug, Clone)]
pub struct CompressOpts {
    /// `-m` level, 0 (store) through 5 (maximum).
    pub level: u8,
    /// Descend into directories. Default is on.
    pub recurse: bool,
    /// Drop the top directory name from stored paths (`-ep1`).
    pub ep1: bool,
    /// Archive symlink targets instead of the links (`--follow-symlinks`).
    pub follow_symlinks: bool,
    /// ZIP method used when the level is not 0 and the extension is not stored.
    pub zip_method: ZipMethod,
    /// Extensions forced to Stored inside ZIP archives.
    pub store_exts: StoreList,
    /// Additional include globs (`-n`). Empty means include everything not excluded.
    pub includes: Vec<String>,
    /// Exclude globs (`-x`).
    pub excludes: Vec<String>,
    /// Replace an existing archive. When false, an existing ZIP is updated.
    pub force: bool,
}

impl Default for CompressOpts {
    fn default() -> Self {
        Self {
            level: 3,
            recurse: true,
            ep1: false,
            follow_symlinks: false,
            zip_method: ZipMethod::Deflate,
            store_exts: StoreList::default(),
            includes: Vec::new(),
            excludes: Vec::new(),
            force: false,
        }
    }
}

impl CompressOpts {
    /// Deflate level for `-m`.
    pub fn deflate_level(&self) -> i64 {
        match self.level {
            0 => 0,
            1 => 1,
            2 => 3,
            3 => 6,
            4 => 7,
            _ => 9,
        }
    }

    /// Zstd level for `-m`. Level 0 is the fastest setting that still writes a frame.
    pub fn zstd_level(&self) -> i32 {
        match self.level {
            0 | 1 => 1,
            2 => 3,
            3 => 7,
            4 => 12,
            _ => 19,
        }
    }

    /// gzip level for `-m`. gzip has no store method, so 0 maps to the fastest level.
    pub fn gzip_level(&self) -> u32 {
        match self.level {
            0 | 1 => 1,
            2 => 3,
            3 => 6,
            4 => 7,
            _ => 9,
        }
    }
}

/// Settings for `szip x` and `szip e`.
#[derive(Debug, Clone)]
pub struct ExtractOpts {
    /// Directory that receives the extracted tree.
    pub output: PathBuf,
    /// Collision policy.
    pub overwrite: Overwrite,
    /// Drop archive directories and write every file into [`Self::output`].
    pub flat: bool,
    /// Keep a partial file when decoding fails.
    pub keep_broken: bool,
    /// Include globs. Empty means every entry.
    pub includes: Vec<String>,
    /// Exact archive paths. `None` means no extra constraint.
    pub exact_paths: Option<Vec<String>>,
    /// Stop after this many uncompressed bytes have been written.
    pub max_bytes: u64,
}

impl ExtractOpts {
    /// Extract into `output` with the default safety cap (64 GiB).
    pub fn new(output: impl Into<PathBuf>) -> Self {
        Self {
            output: output.into(),
            overwrite: Overwrite::Ask,
            flat: false,
            keep_broken: false,
            includes: Vec::new(),
            exact_paths: None,
            max_bytes: 64 << 30,
        }
    }
}

/// Outcome of a conflict callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    /// Replace the file.
    Overwrite,
    /// Leave it.
    Skip,
    /// Pick a new name.
    Rename,
}

/// Summary of a completed pack.
#[derive(Debug, Clone)]
pub struct AddReport {
    /// Final archive path.
    pub path: PathBuf,
    /// Members written, including directories.
    pub entries: u64,
    /// Uncompressed payload bytes.
    pub bytes: u64,
    /// Archive size on disk.
    pub packed_bytes: u64,
}

/// Summary of a completed extract.
#[derive(Debug, Clone, Default)]
pub struct ExtractReport {
    /// Files, directories, and links written.
    pub extracted: u64,
    /// Entries left in place because of the overwrite policy.
    pub skipped: u64,
    /// Non-fatal notes. A non-empty list is exit code 1.
    pub warnings: Vec<String>,
}

/// Summary of `szip t`.
#[derive(Debug, Clone, Default)]
pub struct TestReport {
    /// Members visited.
    pub entries: u64,
    /// Payload bytes decoded.
    pub bytes: u64,
    /// Checksum failures.
    pub checksums: Vec<String>,
    /// Entries skipped because they are encrypted.
    pub encrypted: Vec<String>,
}

impl TestReport {
    /// Exit code: `0` ok, `3` bad checksum, `11` encrypted and otherwise intact.
    pub fn exit_code(&self) -> i32 {
        if !self.checksums.is_empty() {
            3
        } else if !self.encrypted.is_empty() {
            11
        } else {
            0
        }
    }
}
