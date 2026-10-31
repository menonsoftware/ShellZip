//! Pack, extract, and theme choices stored in a TOML file.
//!
//! The path comes from the `directories` config directory, not the registry.
//! Destination paths are remembered after a successful extract. Save settings
//! does not store the destination itself.

use shellzip_core::{ExtractUpdate, Overwrite, SevenZMethod, ZipMethod};

use crate::chrome::PackSettings;
use crate::theme::Preference;

/// How archive paths are written into the destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathMode {
    /// Drop the archive folder that is open in the shell.
    #[default]
    Relative,
    /// Keep each member's full archive path.
    Full,
    /// Write every file into the destination, without folders.
    None,
}

/// What to do with the archive after a clean extract of every member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeleteArchive {
    /// Leave the archive in place.
    #[default]
    Never,
    /// Ask after a successful extract.
    Ask,
    /// Delete after a successful extract.
    Always,
}

/// Extract dialog choices, except the destination path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtractSettings {
    /// Which existing files may be replaced.
    pub update: ExtractUpdate,
    /// What to do when a chosen file already exists.
    pub overwrite: Overwrite,
    /// Relative, full, or no pathnames.
    pub paths: PathMode,
    /// Keep a file that failed to decode.
    pub keep_broken: bool,
    /// Open the destination in Explorer when the extract finishes.
    pub show_in_explorer: bool,
    /// Restore modification time. This is the only time the archives record.
    pub restore_mtime: bool,
    /// Saved for the dialog. Creation time is not stored in entries yet.
    pub restore_created: bool,
    /// Saved for the dialog. Last access time is not stored in entries yet.
    pub restore_accessed: bool,
    /// Delete the archive after a clean extract of every member.
    pub delete: DeleteArchive,
    /// Send a deleted archive to the recycle bin.
    pub recycle: bool,
    /// Start with the progress window hidden so the file list stays usable.
    pub background: bool,
}

impl Default for ExtractSettings {
    fn default() -> Self {
        Self {
            update: ExtractUpdate::Replace,
            overwrite: Overwrite::Ask,
            paths: PathMode::Relative,
            keep_broken: false,
            show_in_explorer: false,
            restore_mtime: true,
            restore_created: false,
            restore_accessed: false,
            delete: DeleteArchive::Never,
            recycle: false,
            background: false,
        }
    }
}

/// Pack options, extract options, theme, recent destinations, and recent archives.
pub struct StoredSettings {
    /// Compression choices.
    pub pack: PackSettings,
    /// Extract dialog choices.
    pub extract: ExtractSettings,
    /// Recent destination paths, newest first.
    pub recent: Vec<String>,
    /// Recently opened archives, newest first.
    pub opened: Vec<String>,
    /// Light, dark, or follow the operating system.
    pub theme: Preference,
}

/// Load the saved options, or the defaults when the file is missing.
pub fn load() -> StoredSettings {
    let Some(path) = settings_path() else {
        return defaults();
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return defaults();
    };
    parse(&text).unwrap_or_else(defaults)
}

/// Write pack options, extract options, the theme, and recent paths.
pub fn save(
    pack: &PackSettings,
    extract: &ExtractSettings,
    recent: &[String],
    opened: &[String],
    theme: Preference,
) -> Result<(), String> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    let body = serialize(pack, extract, recent, opened, theme)?;
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, body).map_err(|err| format!("write {}: {err}", tmp.display()))?;
    std::fs::rename(&tmp, &path).map_err(|err| format!("rename {}: {err}", path.display()))?;
    Ok(())
}

fn defaults() -> StoredSettings {
    StoredSettings {
        pack: PackSettings {
            level: 3,
            method: ZipMethod::Deflate,
            sevenz_method: SevenZMethod::Lzma2,
            solid: true,
            background: false,
        },
        extract: ExtractSettings::default(),
        recent: Vec::new(),
        opened: Vec::new(),
        theme: Preference::System,
    }
}

fn settings_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("com", "ShellZip", "shellzip")
        .map(|dirs| dirs.config_dir().join("settings.toml"))
}

fn serialize(
    pack: &PackSettings,
    extract: &ExtractSettings,
    recent: &[String],
    opened: &[String],
    theme: Preference,
) -> Result<String, String> {
    let mut table = toml::Table::new();
    table.insert(
        "level".to_string(),
        toml::Value::Integer(i64::from(pack.level)),
    );
    table.insert(
        "method".to_string(),
        toml::Value::String(method_name(pack.method).to_string()),
    );
    table.insert(
        "sevenz_method".to_string(),
        toml::Value::String(sevenz_name(pack.sevenz_method).to_string()),
    );
    table.insert("solid".to_string(), toml::Value::Boolean(pack.solid));
    table.insert(
        "add_background".to_string(),
        toml::Value::Boolean(pack.background),
    );
    table.insert(
        "overwrite".to_string(),
        toml::Value::String(overwrite_name(extract.overwrite).to_string()),
    );
    table.insert(
        "flat".to_string(),
        toml::Value::Boolean(extract.paths == PathMode::None),
    );
    table.insert(
        "paths".to_string(),
        toml::Value::String(path_name(extract.paths).to_string()),
    );
    table.insert(
        "update".to_string(),
        toml::Value::String(update_name(extract.update).to_string()),
    );
    table.insert(
        "keep_broken".to_string(),
        toml::Value::Boolean(extract.keep_broken),
    );
    table.insert(
        "show_in_explorer".to_string(),
        toml::Value::Boolean(extract.show_in_explorer),
    );
    table.insert(
        "restore_mtime".to_string(),
        toml::Value::Boolean(extract.restore_mtime),
    );
    table.insert(
        "restore_created".to_string(),
        toml::Value::Boolean(extract.restore_created),
    );
    table.insert(
        "restore_accessed".to_string(),
        toml::Value::Boolean(extract.restore_accessed),
    );
    table.insert(
        "delete".to_string(),
        toml::Value::String(delete_name(extract.delete).to_string()),
    );
    table.insert("recycle".to_string(), toml::Value::Boolean(extract.recycle));
    table.insert(
        "extract_background".to_string(),
        toml::Value::Boolean(extract.background),
    );
    table.insert(
        "recent".to_string(),
        toml::Value::Array(recent.iter().cloned().map(toml::Value::String).collect()),
    );
    table.insert(
        "opened".to_string(),
        toml::Value::Array(opened.iter().cloned().map(toml::Value::String).collect()),
    );
    table.insert(
        "theme".to_string(),
        toml::Value::String(theme.as_str().to_string()),
    );
    toml::to_string(&table).map_err(|err| err.to_string())
}

fn parse(text: &str) -> Option<StoredSettings> {
    let value: toml::Value = toml::from_str(text).ok()?;
    let table = value.as_table()?;
    let level = table.get("level").and_then(toml::Value::as_integer)?;
    let level = u8::try_from(level).ok()?.min(5);
    let method = match table.get("method").and_then(toml::Value::as_str)? {
        "zstd" => ZipMethod::Zstd,
        _ => ZipMethod::Deflate,
    };
    let overwrite = match table.get("overwrite").and_then(toml::Value::as_str)? {
        "always" => Overwrite::Always,
        "rename" => Overwrite::Rename,
        "ask" => Overwrite::Ask,
        _ => Overwrite::Never,
    };
    let flat = table
        .get("flat")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let paths = match table.get("paths").and_then(toml::Value::as_str) {
        Some("none") => PathMode::None,
        Some("full") => PathMode::Full,
        Some("relative") => PathMode::Relative,
        _ if flat => PathMode::None,
        _ => PathMode::Full,
    };
    let update = match table.get("update").and_then(toml::Value::as_str) {
        Some("update") => ExtractUpdate::Update,
        Some("freshen") => ExtractUpdate::Freshen,
        _ => ExtractUpdate::Replace,
    };
    let sevenz_method = match table.get("sevenz_method").and_then(toml::Value::as_str) {
        Some("lzma") => SevenZMethod::Lzma,
        Some("ppmd") => SevenZMethod::Ppmd,
        Some("bzip2") => SevenZMethod::Bzip2,
        Some("deflate") => SevenZMethod::Deflate,
        Some("zstd") => SevenZMethod::Zstd,
        _ => SevenZMethod::Lzma2,
    };
    let solid = table
        .get("solid")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    let bool_or = |key, default| {
        table
            .get(key)
            .and_then(toml::Value::as_bool)
            .unwrap_or(default)
    };
    let delete = match table.get("delete").and_then(toml::Value::as_str) {
        Some("ask") => DeleteArchive::Ask,
        Some("always") => DeleteArchive::Always,
        _ => DeleteArchive::Never,
    };
    let recent = table
        .get("recent")
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let opened = table
        .get("opened")
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    Some(StoredSettings {
        pack: PackSettings {
            level,
            method,
            sevenz_method,
            solid,
            background: bool_or("add_background", false),
        },
        extract: ExtractSettings {
            update,
            overwrite,
            paths,
            keep_broken: bool_or("keep_broken", false),
            show_in_explorer: bool_or("show_in_explorer", false),
            restore_mtime: bool_or("restore_mtime", true),
            restore_created: bool_or("restore_created", false),
            restore_accessed: bool_or("restore_accessed", false),
            delete,
            recycle: bool_or("recycle", false),
            background: bool_or("extract_background", false),
        },
        recent,
        opened,
        theme: table
            .get("theme")
            .and_then(toml::Value::as_str)
            .map(Preference::from_token)
            .unwrap_or_default(),
    })
}

fn sevenz_name(method: SevenZMethod) -> &'static str {
    match method {
        SevenZMethod::Lzma2 => "lzma2",
        SevenZMethod::Lzma => "lzma",
        SevenZMethod::Ppmd => "ppmd",
        SevenZMethod::Bzip2 => "bzip2",
        SevenZMethod::Deflate => "deflate",
        SevenZMethod::Zstd => "zstd",
    }
}

fn method_name(method: ZipMethod) -> &'static str {
    match method {
        ZipMethod::Deflate => "deflate",
        ZipMethod::Zstd => "zstd",
    }
}

fn overwrite_name(mode: Overwrite) -> &'static str {
    match mode {
        Overwrite::Ask => "ask",
        Overwrite::Always => "always",
        Overwrite::Never => "never",
        Overwrite::Rename => "rename",
    }
}

fn path_name(mode: PathMode) -> &'static str {
    match mode {
        PathMode::Relative => "relative",
        PathMode::Full => "full",
        PathMode::None => "none",
    }
}

fn update_name(mode: ExtractUpdate) -> &'static str {
    match mode {
        ExtractUpdate::Replace => "replace",
        ExtractUpdate::Update => "update",
        ExtractUpdate::Freshen => "freshen",
    }
}

fn delete_name(mode: DeleteArchive) -> &'static str {
    match mode {
        DeleteArchive::Never => "never",
        DeleteArchive::Ask => "ask",
        DeleteArchive::Always => "always",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_options_file() {
        let pack = PackSettings {
            level: 1,
            method: ZipMethod::Zstd,
            sevenz_method: SevenZMethod::Ppmd,
            solid: false,
            background: true,
        };
        let extract = ExtractSettings {
            update: ExtractUpdate::Freshen,
            overwrite: Overwrite::Rename,
            paths: PathMode::None,
            keep_broken: true,
            show_in_explorer: true,
            restore_mtime: false,
            restore_created: true,
            restore_accessed: true,
            delete: DeleteArchive::Ask,
            recycle: true,
            background: true,
        };
        let recent = vec![r"C:\work\out".to_string()];
        let opened = vec![r"C:\work\docs.zip".to_string()];
        let parsed =
            parse(&serialize(&pack, &extract, &recent, &opened, Preference::Dark).expect("toml"))
                .expect("settings");
        assert_eq!(parsed.pack, pack);
        assert_eq!(parsed.extract, extract);
        assert_eq!(parsed.recent, recent);
        assert_eq!(parsed.opened, opened);
        assert_eq!(parsed.theme, Preference::Dark);
    }

    #[test]
    fn missing_opened_list_stays_empty() {
        let text = "\
level = 3
method = \"deflate\"
overwrite = \"ask\"
";
        let parsed = parse(text).expect("old settings");
        assert!(parsed.opened.is_empty());
    }

    #[test]
    fn old_flat_flag_selects_no_pathnames_when_paths_is_absent() {
        let text = "\
level = 3
method = \"deflate\"
overwrite = \"never\"
flat = true
";
        let parsed = parse(text).expect("old settings");
        assert_eq!(parsed.extract.paths, PathMode::None);
        assert_eq!(parsed.extract.overwrite, Overwrite::Never);
        assert_eq!(parsed.pack.level, 3);
        assert!(!parsed.pack.background);
        assert!(!parsed.extract.background);
        assert_eq!(parsed.theme, Preference::System);
    }
}
