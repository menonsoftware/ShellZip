//! Pack, extract, report, and theme choices stored in a TOML file.
//!
//! The path comes from the `directories` config directory, not the registry.
//! Destination paths are remembered after a successful extract. Save settings
//! does not store the destination itself.

use shellzip_core::{ExtractUpdate, Overwrite, SevenZMethod, ZipMethod};

use crate::chrome::PackSettings;
use crate::theme::Preference;

/// Toolbar glyph size. Normal is 24 px; Large is 48 px; Extra large is 64 px.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolbarSize {
    /// 24 px glyphs.
    Normal,
    /// 48 px glyphs. The default.
    #[default]
    Large,
    /// 64 px glyphs.
    ExtraLarge,
}

impl ToolbarSize {
    /// Token stored in `settings.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Large => "large",
            Self::ExtraLarge => "extra_large",
        }
    }

    /// Parse a settings token. Anything unrecognized is [`Self::Large`].
    pub fn from_token(token: &str) -> Self {
        match token {
            "normal" => Self::Normal,
            "extra_large" => Self::ExtraLarge,
            _ => Self::Large,
        }
    }

    /// Painted glyph size in points.
    pub fn icon_points(self) -> f32 {
        match self {
            Self::Normal => 24.0,
            Self::Large => 48.0,
            Self::ExtraLarge => 64.0,
        }
    }

    /// Clickable toolbar cell. Tight to the glyph so the strip stays compact.
    pub fn button_size(self) -> egui::Vec2 {
        match self {
            Self::Normal => egui::vec2(72.0, 48.0),
            Self::Large => egui::vec2(80.0, 74.0),
            Self::ExtraLarge => egui::vec2(84.0, 90.0),
        }
    }

    /// Caption under the glyph.
    pub fn label_size(self) -> f32 {
        match self {
            Self::Normal => 11.0,
            Self::Large => 12.0,
            Self::ExtraLarge => 13.0,
        }
    }
}

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

/// Report file written by Generate report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportFormat {
    /// HTML table, UTF-8, no byte-order mark.
    #[default]
    Html,
    /// Markdown table.
    Markdown,
    /// Aligned UTF-8 text.
    Text,
}

/// Row order inside a generated report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReportSort {
    /// Keep the order the listing returned.
    #[default]
    Original,
    /// File or member name.
    Name,
    /// Uncompressed size.
    Size,
    /// Compressed size.
    Packed,
    /// Modification time.
    Modified,
    /// CRC32.
    Crc,
}

/// Generate-report choices, except the output path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportSettings {
    /// HTML, Markdown, or text.
    pub format: ReportFormat,
    /// Copy the report onto the clipboard after it is written.
    pub clipboard: bool,
    /// Column used to order rows.
    pub sort: ReportSort,
    /// Sort inside each archive when more than one is included.
    pub sort_archives_separately: bool,
    /// Header block for each archive.
    pub archives: bool,
    /// Non-archive rows.
    pub usual_files: bool,
    /// Print the archive path above its rows.
    pub archive_name: bool,
    /// Column titles and a totals line.
    pub headers: bool,
    /// Uncompressed size column.
    pub file_size: bool,
    /// Compressed size column.
    pub packed_size: bool,
    /// Modification time column.
    pub file_date: bool,
    /// CRC32 column.
    pub file_checksum: bool,
}

impl Default for ReportSettings {
    fn default() -> Self {
        Self {
            format: ReportFormat::Html,
            clipboard: false,
            sort: ReportSort::Original,
            sort_archives_separately: true,
            archives: true,
            usual_files: true,
            archive_name: true,
            headers: true,
            file_size: true,
            packed_size: true,
            file_date: true,
            file_checksum: false,
        }
    }
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

/// Pack options, extract options, report options, theme, and recent paths.
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
    /// Large or extra-large toolbar glyphs.
    pub toolbar: ToolbarSize,
    /// Generate-report choices.
    pub report: ReportSettings,
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

/// Write pack options, extract options, report options, the theme, and recent paths.
pub fn save(
    pack: &PackSettings,
    extract: &ExtractSettings,
    recent: &[String],
    opened: &[String],
    theme: Preference,
    toolbar: ToolbarSize,
    report: &ReportSettings,
) -> Result<(), String> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    let body = serialize(pack, extract, recent, opened, theme, toolbar, report)?;
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
        toolbar: ToolbarSize::Large,
        report: ReportSettings::default(),
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
    toolbar: ToolbarSize,
    report: &ReportSettings,
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
    table.insert(
        "toolbar".to_string(),
        toml::Value::String(toolbar.as_str().to_string()),
    );
    table.insert(
        "report_format".to_string(),
        toml::Value::String(report_format_name(report.format).to_string()),
    );
    table.insert(
        "report_clipboard".to_string(),
        toml::Value::Boolean(report.clipboard),
    );
    table.insert(
        "report_sort".to_string(),
        toml::Value::String(report_sort_name(report.sort).to_string()),
    );
    table.insert(
        "report_sort_archives".to_string(),
        toml::Value::Boolean(report.sort_archives_separately),
    );
    table.insert(
        "report_archives".to_string(),
        toml::Value::Boolean(report.archives),
    );
    table.insert(
        "report_usual_files".to_string(),
        toml::Value::Boolean(report.usual_files),
    );
    table.insert(
        "report_archive_name".to_string(),
        toml::Value::Boolean(report.archive_name),
    );
    table.insert(
        "report_headers".to_string(),
        toml::Value::Boolean(report.headers),
    );
    table.insert(
        "report_file_size".to_string(),
        toml::Value::Boolean(report.file_size),
    );
    table.insert(
        "report_packed_size".to_string(),
        toml::Value::Boolean(report.packed_size),
    );
    table.insert(
        "report_file_date".to_string(),
        toml::Value::Boolean(report.file_date),
    );
    table.insert(
        "report_file_checksum".to_string(),
        toml::Value::Boolean(report.file_checksum),
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
    let report = ReportSettings {
        format: match table.get("report_format").and_then(toml::Value::as_str) {
            Some("markdown") => ReportFormat::Markdown,
            Some("text") => ReportFormat::Text,
            _ => ReportFormat::Html,
        },
        clipboard: bool_or("report_clipboard", false),
        sort: match table.get("report_sort").and_then(toml::Value::as_str) {
            Some("name") => ReportSort::Name,
            Some("size") => ReportSort::Size,
            Some("packed") => ReportSort::Packed,
            Some("modified") => ReportSort::Modified,
            Some("crc") => ReportSort::Crc,
            _ => ReportSort::Original,
        },
        sort_archives_separately: bool_or("report_sort_archives", true),
        archives: bool_or("report_archives", true),
        usual_files: bool_or("report_usual_files", true),
        archive_name: bool_or("report_archive_name", true),
        headers: bool_or("report_headers", true),
        file_size: bool_or("report_file_size", true),
        packed_size: bool_or("report_packed_size", true),
        file_date: bool_or("report_file_date", true),
        file_checksum: bool_or("report_file_checksum", false),
    };
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
        toolbar: table
            .get("toolbar")
            .and_then(toml::Value::as_str)
            .map(ToolbarSize::from_token)
            .unwrap_or_default(),
        report,
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

fn report_format_name(format: ReportFormat) -> &'static str {
    match format {
        ReportFormat::Html => "html",
        ReportFormat::Markdown => "markdown",
        ReportFormat::Text => "text",
    }
}

fn report_sort_name(sort: ReportSort) -> &'static str {
    match sort {
        ReportSort::Original => "original",
        ReportSort::Name => "name",
        ReportSort::Size => "size",
        ReportSort::Packed => "packed",
        ReportSort::Modified => "modified",
        ReportSort::Crc => "crc",
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
        let report = ReportSettings {
            format: ReportFormat::Text,
            clipboard: true,
            sort: ReportSort::Crc,
            sort_archives_separately: false,
            archives: false,
            usual_files: true,
            archive_name: false,
            headers: false,
            file_size: false,
            packed_size: true,
            file_date: false,
            file_checksum: true,
        };
        let parsed = parse(
            &serialize(
                &pack,
                &extract,
                &recent,
                &opened,
                Preference::Dark,
                ToolbarSize::ExtraLarge,
                &report,
            )
            .expect("toml"),
        )
        .expect("settings");
        assert_eq!(parsed.pack, pack);
        assert_eq!(parsed.extract, extract);
        assert_eq!(parsed.recent, recent);
        assert_eq!(parsed.opened, opened);
        assert_eq!(parsed.theme, Preference::Dark);
        assert_eq!(parsed.toolbar, ToolbarSize::ExtraLarge);
        assert_eq!(parsed.report, report);
    }

    #[test]
    fn toolbar_size_tokens_round_trip() {
        assert_eq!(ToolbarSize::from_token("normal"), ToolbarSize::Normal);
        assert_eq!(ToolbarSize::from_token("large"), ToolbarSize::Large);
        assert_eq!(
            ToolbarSize::from_token("extra_large"),
            ToolbarSize::ExtraLarge
        );
        assert_eq!(ToolbarSize::from_token("nope"), ToolbarSize::Large);
        assert_eq!(ToolbarSize::Normal.as_str(), "normal");
        assert_eq!(ToolbarSize::Large.as_str(), "large");
        assert_eq!(ToolbarSize::Normal.icon_points(), 24.0);
        assert_eq!(ToolbarSize::Large.icon_points(), 48.0);
        assert_eq!(ToolbarSize::ExtraLarge.icon_points(), 64.0);
        assert_eq!(ToolbarSize::Normal.button_size(), egui::vec2(72.0, 48.0));
        assert_eq!(ToolbarSize::Large.button_size(), egui::vec2(80.0, 74.0));
        assert_eq!(
            ToolbarSize::ExtraLarge.button_size(),
            egui::vec2(84.0, 90.0)
        );
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
        assert_eq!(parsed.report, ReportSettings::default());
    }
}
