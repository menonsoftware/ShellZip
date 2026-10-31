//! Archive information, comment, and self-extractor pages.
//!
//! The summary is loaded on the worker. This module only paints it.

use std::path::Path;

use shellzip_core::ArchiveSummary;

use crate::chrome;
use crate::icons::Icon;
use crate::theme;

/// Which page is visible.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Info,
    Comment,
    Sfx,
}

/// Centered archive summary. OK and Cancel both close it.
pub(crate) struct InfoDialog {
    filename: String,
    summary: ArchiveSummary,
    tab: Tab,
    help: bool,
}

/// The user dismissed the dialog.
pub(crate) enum InfoAction {
    /// OK, Cancel, or the window close button.
    Close,
}

impl InfoDialog {
    /// `path` supplies the filename in the title. `summary` is shown as-is.
    pub(crate) fn new(path: &Path, summary: ArchiveSummary) -> Self {
        Self {
            filename: file_name(path),
            summary,
            tab: Tab::Info,
            help: false,
        }
    }

    /// Paint the dialog.
    pub(crate) fn show(&mut self, ctx: &egui::Context) -> Option<InfoAction> {
        let mut open = true;
        let mut action = None;
        let title = format!("Archive {}", self.filename);
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .min_width(560.0)
            .min_height(420.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(560.0);
                ui.set_min_height(380.0);
                tabs(ui, self);
                ui.separator();
                match self.tab {
                    Tab::Info => info_tab(ui, &self.summary),
                    Tab::Comment => comment_tab(ui, &self.summary),
                    Tab::Sfx => sfx_tab(ui, &self.summary),
                }
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let colors = theme::palette(ui.ctx());
                        if chrome::labeled_button(ui, Icon::CircleHelp, colors.open, "Help")
                            .clicked()
                        {
                            self.help = true;
                        }
                        if chrome::labeled_button(ui, Icon::X, colors.ink, "Cancel").clicked() {
                            action = Some(InfoAction::Close);
                        }
                        if chrome::labeled_button(ui, Icon::Check, colors.ok, "OK").clicked() {
                            action = Some(InfoAction::Close);
                        }
                    });
                });
            });
        help_window(ctx, self);
        if action.is_none() && !open {
            return Some(InfoAction::Close);
        }
        action
    }
}

fn tabs(ui: &mut egui::Ui, dialog: &mut InfoDialog) {
    ui.horizontal(|ui| {
        for (tab, label) in [
            (Tab::Info, "Info"),
            (Tab::Comment, "Comment"),
            (Tab::Sfx, "SFX"),
        ] {
            if ui.selectable_label(dialog.tab == tab, label).clicked() {
                dialog.tab = tab;
            }
        }
    });
}

fn info_tab(ui: &mut egui::Ui, summary: &ArchiveSummary) {
    ui.horizontal_top(|ui| {
        ratio_column(ui, summary.ratio_percent);
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(&summary.kind).strong().size(16.0));
            ui.add_space(6.0);
            egui::Grid::new("shellzip-archive-info")
                .num_columns(2)
                .spacing(egui::vec2(16.0, 4.0))
                .show(ui, |ui| {
                    for (label, value) in info_rows(summary) {
                        ui.label(egui::RichText::new(label).color(theme::palette(ui.ctx()).muted));
                        ui.label(value);
                        ui.end_row();
                    }
                });
        });
    });
}

fn info_rows(summary: &ArchiveSummary) -> [(&'static str, String); 10] {
    [
        ("Version to extract", summary.version.clone()),
        ("Host OS", summary.host_os.clone()),
        ("Total folders", chrome::format_grouped(summary.folders)),
        ("Total files", chrome::format_grouped(summary.files)),
        ("Total size", chrome::format_grouped(summary.total_size)),
        ("Packed size", chrome::format_grouped(summary.packed_size)),
        ("Ratio", format!("{}%", summary.ratio_percent)),
        ("SFX module size", chrome::format_grouped(summary.sfx_bytes)),
        (
            "Main comment",
            presence(summary.comment.is_some()).to_string(),
        ),
        ("Encryption", presence(summary.encrypted).to_string()),
    ]
}

fn ratio_column(ui: &mut egui::Ui, percent: u64) {
    ui.vertical(|ui| {
        let colors = theme::palette(ui.ctx());
        let (rect, _) = ui.allocate_exact_size(egui::vec2(32.0, 200.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 2.0, colors.field);
        painter.rect_stroke(
            rect,
            2.0,
            egui::Stroke::new(1.0_f32, colors.line),
            egui::StrokeKind::Inside,
        );
        let fraction = (percent as f32 / 100.0).clamp(0.0, 1.0);
        let fill_height = rect.height() * fraction;
        if fill_height > 0.0 {
            let fill = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - fill_height),
                rect.right_bottom(),
            );
            // `open` is the theme accent used for the Info glyph and links.
            painter.rect_filled(fill, 2.0, colors.open);
        }
        ui.add_space(4.0);
        ui.label(format!("{percent}%"));
    });
}

fn comment_tab(ui: &mut egui::Ui, summary: &ArchiveSummary) {
    match &summary.comment {
        Some(text) => {
            egui::ScrollArea::vertical()
                .id_salt("shellzip-archive-comment")
                .max_height(240.0)
                .show(ui, |ui| {
                    ui.label(text);
                });
        }
        None => {
            ui.label("Absent");
        }
    }
}

fn sfx_tab(ui: &mut egui::Ui, summary: &ArchiveSummary) {
    ui.label(format!(
        "SFX module size: {}",
        chrome::format_grouped(summary.sfx_bytes)
    ));
    if summary.sfx_bytes == 0 {
        ui.label("This file is not a self-extractor.");
    }
}

fn help_window(ctx: &egui::Context, dialog: &mut InfoDialog) {
    if !dialog.help {
        return;
    }
    let mut open = true;
    let sentence = match dialog.tab {
        Tab::Info => "Packed size over total size, read from the archive headers.",
        Tab::Comment => "The archive comment, or Absent when the format stores none.",
        Tab::Sfx => {
            "The self-extractor stub size, or a note that this file is not a self-extractor."
        }
    };
    egui::Window::new("Archive information help")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.label(sentence);
        });
    dialog.help = open;
}

fn presence(present: bool) -> &'static str {
    if present { "Present" } else { "Absent" }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}
