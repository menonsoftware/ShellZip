//! Generate a listing report as HTML, Markdown, or UTF-8 text.
//!
//! Rendering is a pure function so it can be tested without a window. Archive
//! listings are filled in by the worker before OK writes the file.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use shellzip_core::format_time;

use crate::chrome;
use crate::icons::Icon;
use crate::settings::{ReportFormat, ReportSettings, ReportSort};
use crate::theme;

/// One row in a report.
#[derive(Clone)]
pub(crate) struct ReportRow {
    /// File or member name.
    pub name: String,
    /// Uncompressed size.
    pub size: u64,
    /// Compressed size.
    pub packed: u64,
    /// Modification time, when known.
    pub modified: Option<SystemTime>,
    /// CRC32, when the archive records one.
    pub crc: Option<u32>,
}

/// One archive and the members to list under it.
#[derive(Clone)]
pub(crate) struct ReportArchive {
    /// Archive path shown when the archive-name flag is on.
    pub name: String,
    /// Members in archive order until a sort is applied.
    pub rows: Vec<ReportRow>,
}

/// The user saved choices, wrote a file, or dismissed the dialog.
pub(crate) enum ReportAction {
    /// Store these choices in settings.
    Save(ReportSettings),
    /// The report was written to this path.
    Wrote(PathBuf),
    /// Cancel or the window close button.
    Close,
}

/// Format, path, and include flags. Rows arrive after the worker lists archives.
pub(crate) struct ReportDialog {
    settings: ReportSettings,
    saved: ReportSettings,
    path: String,
    loose: Vec<ReportRow>,
    archives: Vec<ReportArchive>,
    pending: bool,
    failed: bool,
    notice: Option<String>,
    help: bool,
}

impl ReportDialog {
    /// Open with loose rows already known. `pending` waits for archive listings.
    pub(crate) fn open(
        settings: ReportSettings,
        path: PathBuf,
        loose: Vec<ReportRow>,
        pending: bool,
    ) -> Self {
        Self {
            saved: settings,
            settings,
            path: path.display().to_string(),
            loose,
            archives: Vec::new(),
            pending,
            failed: false,
            notice: None,
            help: false,
        }
    }

    /// Archive blocks from the worker. Listing is finished.
    pub(crate) fn set_archives(&mut self, archives: Vec<ReportArchive>) {
        self.archives = archives;
        self.pending = false;
        self.failed = false;
    }

    /// The worker could not list an archive. OK stays disabled.
    pub(crate) fn fail(&mut self, message: impl Into<String>) {
        self.pending = false;
        self.failed = true;
        self.notice = Some(message.into());
    }

    /// Paint the dialog. Writing the file happens here so a failure can stay open.
    pub(crate) fn show(&mut self, ctx: &egui::Context) -> Option<ReportAction> {
        let mut open = true;
        let mut action = None;
        let previous_format = self.settings.format;
        egui::Window::new("Generate report")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .min_width(520.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(520.0);
                ui.label("Format");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.settings.format, ReportFormat::Html, "HTML");
                    ui.radio_value(
                        &mut self.settings.format,
                        ReportFormat::Markdown,
                        "Markdown",
                    );
                    ui.radio_value(
                        &mut self.settings.format,
                        ReportFormat::Text,
                        "Text (UTF-8)",
                    );
                });
                if self.settings.format != previous_format {
                    self.path = swap_extension(&self.path, self.settings.format);
                }
                ui.add_space(6.0);
                ui.label("Report path");
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let width = (ui.available_width() - 88.0).max(48.0);
                    chrome::path_field(
                        ui,
                        &mut self.path,
                        width,
                        "Report file",
                        "shellzip-report-path",
                    );
                    if ui.button("Browse").clicked()
                        && let Some(path) = browse_report(&self.path, self.settings.format)
                    {
                        self.path = path.display().to_string();
                    }
                });
                ui.checkbox(&mut self.settings.clipboard, "Copy report to clipboard");
                ui.add_space(6.0);
                ui.label("Sort by");
                egui::Grid::new("shellzip-report-sort")
                    .num_columns(3)
                    .spacing(egui::vec2(12.0, 2.0))
                    .show(ui, |ui| {
                        ui.radio_value(
                            &mut self.settings.sort,
                            ReportSort::Original,
                            "Original order",
                        );
                        ui.radio_value(&mut self.settings.sort, ReportSort::Name, "Name");
                        ui.radio_value(&mut self.settings.sort, ReportSort::Size, "Size");
                        ui.end_row();
                        ui.radio_value(&mut self.settings.sort, ReportSort::Packed, "Packed size");
                        ui.radio_value(&mut self.settings.sort, ReportSort::Modified, "Modified");
                        ui.radio_value(&mut self.settings.sort, ReportSort::Crc, "CRC");
                        ui.end_row();
                    });
                ui.checkbox(
                    &mut self.settings.sort_archives_separately,
                    "Sort archives separately",
                );
                ui.add_space(6.0);
                ui.label("Include");
                egui::Grid::new("shellzip-report-include")
                    .num_columns(2)
                    .spacing(egui::vec2(16.0, 2.0))
                    .show(ui, |ui| {
                        ui.checkbox(&mut self.settings.archives, "Archives");
                        ui.checkbox(&mut self.settings.usual_files, "Usual files");
                        ui.end_row();
                        ui.checkbox(&mut self.settings.archive_name, "Archive name");
                        ui.checkbox(&mut self.settings.headers, "Headers and totals");
                        ui.end_row();
                        ui.checkbox(&mut self.settings.file_size, "File size");
                        ui.checkbox(&mut self.settings.packed_size, "Packed size");
                        ui.end_row();
                        ui.checkbox(&mut self.settings.file_date, "File date");
                        ui.checkbox(&mut self.settings.file_checksum, "File checksum");
                        ui.end_row();
                    });
                if self.pending {
                    ui.add_space(4.0);
                    ui.label("Reading archives...");
                }
                if let Some(notice) = &self.notice {
                    ui.add_space(4.0);
                    ui.colored_label(theme::palette(ui.ctx()).danger, notice);
                }
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    let colors = theme::palette(ui.ctx());
                    let dirty = self.settings != self.saved;
                    if chrome::icon_button(ui, Icon::Save, colors.ink, "Save settings", dirty)
                        .clicked()
                    {
                        self.saved = self.settings;
                        action = Some(ReportAction::Save(self.settings));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if chrome::labeled_button(ui, Icon::CircleHelp, colors.open, "Help")
                            .clicked()
                        {
                            self.help = true;
                        }
                        if chrome::labeled_button(ui, Icon::X, colors.ink, "Cancel").clicked() {
                            action = Some(ReportAction::Close);
                        }
                        let can_write = !self.pending && !self.failed;
                        if chrome::labeled_button(ui, Icon::Check, colors.ok, "OK").clicked()
                            && can_write
                        {
                            match write_report(
                                &self.path,
                                &self.loose,
                                &self.archives,
                                &self.settings,
                            ) {
                                Ok(text) => {
                                    if self.settings.clipboard {
                                        ui.ctx().copy_text(text);
                                    }
                                    action =
                                        Some(ReportAction::Wrote(PathBuf::from(self.path.trim())));
                                }
                                Err(message) => self.notice = Some(message),
                            }
                        }
                    });
                });
            });
        help_window(ctx, self);
        if action.is_none() && !open {
            return Some(ReportAction::Close);
        }
        action
    }
}

fn help_window(ctx: &egui::Context, dialog: &mut ReportDialog) {
    if !dialog.help {
        return;
    }
    let mut open = true;
    egui::Window::new("Generate report help")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.label(
                "Include flags choose which archive blocks, loose files, headings, totals, and columns are written.",
            );
        });
    dialog.help = open;
}

fn browse_report(current: &str, format: ReportFormat) -> Option<PathBuf> {
    let path = Path::new(current.trim());
    let mut picker = rfd::FileDialog::new().set_title("Save report");
    if let Some(parent) = path.parent().filter(|parent| parent.is_dir()) {
        picker = picker.set_directory(parent);
    }
    if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
        picker = picker.set_file_name(name);
    }
    let ext = extension(format);
    picker.add_filter("Report", &[ext]).save_file()
}

fn write_report(
    path: &str,
    loose: &[ReportRow],
    archives: &[ReportArchive],
    settings: &ReportSettings,
) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("Choose a report path.".to_string());
    }
    let path = Path::new(trimmed);
    let text = render_report(loose, archives, settings);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    std::fs::write(path, text.as_bytes())
        .map_err(|err| format!("write {}: {err}", path.display()))?;
    Ok(text)
}

/// Suggested path beside `archive`: `stem.htm`, `.md`, or `.txt`.
pub(crate) fn default_report_path(archive: &Path, format: ReportFormat) -> PathBuf {
    let mut name = archive
        .file_stem()
        .filter(|stem| !stem.is_empty())
        .map(|stem| stem.to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from("report"));
    name.push(".");
    name.push(extension(format));
    match archive.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
        _ => PathBuf::from(name),
    }
}

/// File extension for `format`, without a leading dot.
pub(crate) fn extension(format: ReportFormat) -> &'static str {
    match format {
        ReportFormat::Html => "htm",
        ReportFormat::Markdown => "md",
        ReportFormat::Text => "txt",
    }
}

fn swap_extension(path: &str, format: ReportFormat) -> String {
    let file = Path::new(path.trim());
    let current = file.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    if !matches!(current, "htm" | "html" | "md" | "txt") {
        return path.to_string();
    }
    file.with_extension(extension(format)).display().to_string()
}

/// Render `loose` files and `archives` with the include flags and sort.
pub(crate) fn render_report(
    loose: &[ReportRow],
    archives: &[ReportArchive],
    settings: &ReportSettings,
) -> String {
    let sections = sections(loose, archives, settings);
    if sections.is_empty() {
        return match settings.format {
            ReportFormat::Html => html_document("<p>No items included.</p>\n"),
            ReportFormat::Markdown | ReportFormat::Text => "No items included.\n".to_string(),
        };
    }
    match settings.format {
        ReportFormat::Html => render_html(&sections, settings),
        ReportFormat::Markdown => render_markdown(&sections, settings),
        ReportFormat::Text => render_text(&sections, settings),
    }
}

struct Section {
    heading: Option<String>,
    rows: Vec<ReportRow>,
}

#[derive(Clone, Copy)]
enum Column {
    Name,
    Size,
    Packed,
    Modified,
    Crc,
}

impl Column {
    fn header(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Size => "Size",
            Self::Packed => "Packed",
            Self::Modified => "Modified",
            Self::Crc => "CRC",
        }
    }

    fn cell(self, row: &ReportRow) -> String {
        match self {
            Self::Name => row.name.clone(),
            Self::Size => chrome::format_grouped(row.size),
            Self::Packed => chrome::format_grouped(row.packed),
            Self::Modified => format_time(row.modified),
            Self::Crc => match row.crc {
                Some(crc) => format!("{crc:08X}"),
                None => "-".to_string(),
            },
        }
    }
}

fn columns(settings: &ReportSettings) -> Vec<Column> {
    let mut columns = vec![Column::Name];
    if settings.file_size {
        columns.push(Column::Size);
    }
    if settings.packed_size {
        columns.push(Column::Packed);
    }
    if settings.file_date {
        columns.push(Column::Modified);
    }
    if settings.file_checksum {
        columns.push(Column::Crc);
    }
    columns
}

fn sections(
    loose: &[ReportRow],
    archives: &[ReportArchive],
    settings: &ReportSettings,
) -> Vec<Section> {
    let mut out = Vec::new();
    if settings.usual_files {
        let mut rows = loose.to_vec();
        sort_rows(&mut rows, settings.sort);
        if !rows.is_empty() {
            out.push(Section {
                heading: None,
                rows,
            });
        }
    }
    if settings.archives {
        let multiple = archives.len() > 1;
        let merge =
            multiple && !settings.sort_archives_separately && settings.sort != ReportSort::Original;
        if merge {
            let mut rows = Vec::new();
            for archive in archives {
                for row in &archive.rows {
                    let mut copy = row.clone();
                    if settings.archive_name {
                        copy.name = format!("{} / {}", archive.name, copy.name);
                    }
                    rows.push(copy);
                }
            }
            sort_rows(&mut rows, settings.sort);
            if !rows.is_empty() {
                out.push(Section {
                    heading: None,
                    rows,
                });
            }
        } else {
            for archive in archives {
                let mut rows = archive.rows.clone();
                if settings.sort_archives_separately || !multiple {
                    sort_rows(&mut rows, settings.sort);
                }
                let heading = settings.archive_name.then(|| archive.name.clone());
                if heading.is_some() || !rows.is_empty() {
                    out.push(Section { heading, rows });
                }
            }
        }
    }
    out
}

fn sort_rows(rows: &mut [ReportRow], sort: ReportSort) {
    if sort == ReportSort::Original {
        return;
    }
    rows.sort_by(|left, right| {
        let compared = match sort {
            ReportSort::Original => Ordering::Equal,
            ReportSort::Name => left.name.cmp(&right.name),
            ReportSort::Size => left.size.cmp(&right.size),
            ReportSort::Packed => left.packed.cmp(&right.packed),
            ReportSort::Modified => cmp_opt(left.modified, right.modified),
            ReportSort::Crc => cmp_opt(left.crc, right.crc),
        };
        if compared == Ordering::Equal {
            left.name.cmp(&right.name)
        } else {
            compared
        }
    });
}

fn cmp_opt<T: Ord>(left: Option<T>, right: Option<T>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn totals_line(rows: &[ReportRow]) -> String {
    let count = rows.len();
    let bytes = rows
        .iter()
        .fold(0u64, |sum, row| sum.saturating_add(row.size));
    let noun = if count == 1 { "file" } else { "files" };
    format!(
        "Total: {count} {noun}, {} bytes",
        chrome::format_grouped(bytes)
    )
}

fn render_html(sections: &[Section], settings: &ReportSettings) -> String {
    let columns = columns(settings);
    let mut body = String::new();
    for section in sections {
        if let Some(heading) = &section.heading {
            body.push_str("<h2>");
            body.push_str(&escape_html(heading));
            body.push_str("</h2>\n");
        }
        body.push_str("<table>\n");
        if settings.headers {
            body.push_str("<thead><tr>");
            for column in &columns {
                body.push_str("<th>");
                body.push_str(column.header());
                body.push_str("</th>");
            }
            body.push_str("</tr></thead>\n");
        }
        body.push_str("<tbody>\n");
        for row in &section.rows {
            body.push_str("<tr>");
            for column in &columns {
                body.push_str("<td>");
                body.push_str(&escape_html(&column.cell(row)));
                body.push_str("</td>");
            }
            body.push_str("</tr>\n");
        }
        body.push_str("</tbody>\n</table>\n");
        if settings.headers {
            body.push_str("<p>");
            body.push_str(&escape_html(&totals_line(&section.rows)));
            body.push_str("</p>\n");
        }
    }
    html_document(&body)
}

fn render_markdown(sections: &[Section], settings: &ReportSettings) -> String {
    let columns = columns(settings);
    let mut out = String::new();
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if let Some(heading) = &section.heading {
            out.push_str("# ");
            out.push_str(&escape_markdown(heading));
            out.push_str("\n\n");
        }
        out.push('|');
        for column in &columns {
            out.push(' ');
            out.push_str(column.header());
            out.push_str(" |");
        }
        out.push('\n');
        out.push('|');
        for _ in &columns {
            out.push_str(" --- |");
        }
        out.push('\n');
        for row in &section.rows {
            out.push('|');
            for column in &columns {
                out.push(' ');
                out.push_str(&escape_markdown(&column.cell(row)));
                out.push_str(" |");
            }
            out.push('\n');
        }
        if settings.headers {
            out.push('\n');
            out.push_str(&totals_line(&section.rows));
            out.push('\n');
        }
    }
    out
}

fn render_text(sections: &[Section], settings: &ReportSettings) -> String {
    let columns = columns(settings);
    let mut out = String::new();
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if let Some(heading) = &section.heading {
            out.push_str(heading);
            out.push_str("\n\n");
        }
        let mut matrix = Vec::new();
        if settings.headers {
            matrix.push(
                columns
                    .iter()
                    .map(|column| column.header().to_string())
                    .collect::<Vec<_>>(),
            );
        }
        for row in &section.rows {
            matrix.push(
                columns
                    .iter()
                    .map(|column| column.cell(row))
                    .collect::<Vec<_>>(),
            );
        }
        if !matrix.is_empty() {
            let width_of = |index: usize| {
                matrix
                    .iter()
                    .filter_map(|line| line.get(index))
                    .map(|cell| display_width(cell))
                    .max()
                    .unwrap_or(0)
            };
            let widths: Vec<usize> = (0..columns.len()).map(width_of).collect();
            for line in &matrix {
                let last = line.len().saturating_sub(1);
                for (index, cell) in line.iter().enumerate() {
                    if index > 0 {
                        out.push_str("  ");
                    }
                    out.push_str(cell);
                    if index != last {
                        let pad = widths
                            .get(index)
                            .copied()
                            .unwrap_or(0)
                            .saturating_sub(display_width(cell));
                        out.push_str(&" ".repeat(pad));
                    }
                }
                out.push('\n');
            }
        }
        if settings.headers {
            out.push('\n');
            out.push_str(&totals_line(&section.rows));
            out.push('\n');
        }
    }
    out
}

fn html_document(body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>ShellZip report</title>\n</head>\n<body>\n{body}</body>\n</html>\n"
    )
}

fn escape_html(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '|' => out.push_str("\\|"),
            '\n' | '\r' => out.push(' '),
            _ => out.push(ch),
        }
    }
    out
}

fn display_width(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, size: u64, packed: u64) -> ReportRow {
        ReportRow {
            name: name.to_string(),
            size,
            packed,
            modified: None,
            crc: None,
        }
    }

    fn archive(name: &str, rows: Vec<ReportRow>) -> ReportArchive {
        ReportArchive {
            name: name.to_string(),
            rows,
        }
    }

    #[test]
    fn html_with_includes_on_lists_columns_and_escapes() {
        let loose = vec![row("loose.txt", 1, 1)];
        let archives = vec![archive(
            "notes.zip",
            vec![
                ReportRow {
                    name: "b.txt".to_string(),
                    size: 2,
                    packed: 2,
                    modified: None,
                    crc: Some(1),
                },
                ReportRow {
                    name: "a<b>&c.txt".to_string(),
                    size: 9_655_595_819,
                    packed: 10,
                    modified: None,
                    crc: Some(0x00A1_B2C3),
                },
            ],
        )];
        let settings = ReportSettings {
            file_checksum: true,
            sort: ReportSort::Name,
            ..Default::default()
        };
        let html = render_report(&loose, &archives, &settings);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("charset=\"utf-8\""));
        assert!(html.contains("<table>"));
        assert!(html.contains("<h2>notes.zip</h2>"));
        assert!(html.contains("<th>Name</th>"));
        assert!(html.contains("<th>Size</th>"));
        assert!(html.contains("<th>Packed</th>"));
        assert!(html.contains("<th>Modified</th>"));
        assert!(html.contains("<th>CRC</th>"));
        assert!(html.contains("a&lt;b&gt;&amp;c.txt"));
        assert!(html.contains("9,655,595,819"));
        assert!(html.contains("00A1B2C3"));
        assert!(html.contains("loose.txt"));
        assert!(html.contains("Total:"));
        let escaped = html.find("a&lt;b&gt;&amp;c.txt").unwrap();
        let later = html.find("b.txt").unwrap();
        assert!(escaped < later, "name sort inside the archive");
    }

    #[test]
    fn html_with_includes_off_drops_columns_blocks_and_usual_files() {
        let loose = vec![row("loose.txt", 4, 4)];
        let archives = vec![archive("notes.zip", vec![row("inside.txt", 8, 2)])];
        let mut settings = ReportSettings {
            usual_files: false,
            archive_name: false,
            headers: false,
            file_size: false,
            packed_size: false,
            file_date: false,
            file_checksum: false,
            ..Default::default()
        };
        let html = render_report(&loose, &archives, &settings);
        assert!(html.contains("inside.txt"));
        assert!(!html.contains("loose.txt"));
        assert!(!html.contains("notes.zip"));
        assert!(!html.contains("<th>"));
        assert!(!html.contains("Total:"));
        assert!(!html.contains(">8<"));

        settings.archives = false;
        settings.usual_files = false;
        let empty = render_report(&loose, &archives, &settings);
        assert!(empty.contains("No items included."));
        assert!(!empty.contains("inside.txt"));
    }

    #[test]
    fn markdown_with_includes_on_is_a_table() {
        let archives = vec![archive("notes.zip", vec![row("a|b.txt", 12, 9)])];
        let settings = ReportSettings {
            format: ReportFormat::Markdown,
            ..Default::default()
        };
        let markdown = render_report(&[], &archives, &settings);
        assert!(markdown.contains("# notes.zip"));
        assert!(markdown.contains("| Name | Size | Packed | Modified |"));
        assert!(markdown.contains("| --- | --- | --- | --- |"));
        assert!(markdown.contains("a\\|b.txt"));
        assert!(markdown.contains("Total: 1 file, 12 bytes"));
        assert!(!markdown.contains("CRC"));
    }

    #[test]
    fn markdown_with_includes_off_omits_checksum_and_usual_files() {
        let loose = vec![row("loose.txt", 1, 1)];
        let archives = vec![archive("notes.zip", vec![row("inside.txt", 2, 2)])];
        let settings = ReportSettings {
            format: ReportFormat::Markdown,
            usual_files: false,
            file_checksum: false,
            file_size: false,
            packed_size: false,
            file_date: false,
            archive_name: false,
            headers: false,
            ..Default::default()
        };
        let markdown = render_report(&loose, &archives, &settings);
        assert!(markdown.contains("inside.txt"));
        assert!(!markdown.contains("loose.txt"));
        assert!(!markdown.contains("notes.zip"));
        assert!(!markdown.contains("CRC"));
        assert!(!markdown.contains("Size"));
        assert!(!markdown.contains("Total:"));
        assert!(markdown.contains("| Name |"));
    }

    #[test]
    fn text_with_includes_on_aligns_columns() {
        let archives = vec![archive("box.zip", vec![row("a.txt", 3, 100)])];
        let settings = ReportSettings {
            format: ReportFormat::Text,
            file_date: false,
            ..Default::default()
        };
        let text = render_report(&[], &archives, &settings);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.first().copied(), Some("box.zip"));
        let header = lines.iter().find(|line| line.starts_with("Name")).unwrap();
        let data = lines.iter().find(|line| line.starts_with("a.txt")).unwrap();
        assert_eq!(header.find("Size"), data.find('3'));
        assert_eq!(header.find("Packed"), data.find("100"));
        assert!(text.contains("Total: 1 file, 3 bytes"));
        assert!(!text.contains("CRC"));
    }

    #[test]
    fn text_with_includes_off_skips_archive_name_and_totals() {
        let loose = vec![row("loose.txt", 9, 9)];
        let archives = vec![archive("box.zip", vec![row("inside.txt", 3, 1)])];
        let mut settings = ReportSettings {
            format: ReportFormat::Text,
            usual_files: false,
            archive_name: false,
            headers: false,
            file_size: false,
            packed_size: false,
            file_date: false,
            file_checksum: false,
            ..Default::default()
        };
        let text = render_report(&loose, &archives, &settings);
        assert!(text.contains("inside.txt"));
        assert!(!text.contains("loose.txt"));
        assert!(!text.contains("box.zip"));
        assert!(!text.contains("Total:"));
        assert!(!text.contains("Name"));

        settings.archives = false;
        let empty = render_report(&loose, &archives, &settings);
        assert_eq!(empty, "No items included.\n");
    }

    #[test]
    fn sort_archives_separately_keeps_each_block() {
        let archives = vec![
            archive("one.zip", vec![row("b.txt", 2, 2), row("a.txt", 1, 1)]),
            archive("two.zip", vec![row("c.txt", 3, 3)]),
        ];
        let mut settings = ReportSettings {
            sort: ReportSort::Name,
            sort_archives_separately: true,
            file_date: false,
            ..Default::default()
        };
        let html = render_report(&[], &archives, &settings);
        assert_eq!(html.matches("<table>").count(), 2);
        let one = html.find("one.zip").unwrap();
        let two = html.find("two.zip").unwrap();
        let a = html.find("a.txt").unwrap();
        let b = html.find("b.txt").unwrap();
        let c = html.find("c.txt").unwrap();
        assert!(one < a && a < b && b < two && two < c);

        settings.sort_archives_separately = false;
        settings.archive_name = false;
        let merged = render_report(&[], &archives, &settings);
        assert_eq!(merged.matches("<table>").count(), 1);
        assert!(!merged.contains("one.zip"));
        let a = merged.find("a.txt").unwrap();
        let b = merged.find("b.txt").unwrap();
        let c = merged.find("c.txt").unwrap();
        assert!(a < b && b < c);
    }
}
