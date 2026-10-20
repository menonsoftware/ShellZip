//! Toolbar, member table, and status line.

use std::collections::HashSet;

use egui_extras::{Column, TableBuilder};
use shellzip_core::{Entry, EntryKind, format_bytes, format_time};

use crate::icons::{self, Icon};
use crate::theme::{DANGER, INK, MUTED, OK};

/// A labeled toolbar control. The icon sits beside the button.
pub fn tool(ui: &mut egui::Ui, icon: Icon, label: &str) -> bool {
    ui.horizontal(|ui| {
        icons::show(ui, icon, INK, 16.0);
        ui.button(label).clicked()
    })
    .inner
}

/// Member table. Clicking the name toggles selection.
pub fn file_list(ui: &mut egui::Ui, entries: &[Entry], selected: &mut HashSet<String>) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(28.0))
        .column(Column::remainder().at_least(240.0))
        .column(Column::exact(90.0))
        .column(Column::exact(90.0))
        .column(Column::exact(80.0))
        .column(Column::exact(140.0))
        .header(22.0, |mut header| {
            header.col(|ui| {
                ui.label("");
            });
            header.col(|ui| {
                ui.strong("Name");
            });
            header.col(|ui| {
                ui.strong("Size");
            });
            header.col(|ui| {
                ui.strong("Packed");
            });
            header.col(|ui| {
                ui.strong("Method");
            });
            header.col(|ui| {
                ui.strong("Modified");
            });
        })
        .body(|body| {
            body.rows(22.0, entries.len(), |mut row| {
                let entry = &entries[row.index()];
                let path = entry.path.clone();
                let on = selected.contains(&path);
                row.col(|ui| icons::show(ui, kind_icon(entry.kind), MUTED, 14.0));
                row.col(|ui| {
                    let label = display_name(entry);
                    if ui.selectable_label(on, label).clicked() {
                        if on {
                            selected.remove(&path);
                        } else {
                            selected.insert(path);
                        }
                    }
                });
                row.col(|ui| {
                    ui.label(
                        entry
                            .size
                            .map(format_bytes)
                            .unwrap_or_else(|| "-".to_string()),
                    );
                });
                row.col(|ui| {
                    ui.label(
                        entry
                            .packed_size
                            .map(format_bytes)
                            .unwrap_or_else(|| "-".to_string()),
                    );
                });
                row.col(|ui| {
                    ui.label(entry.method.clone().unwrap_or_else(|| "-".to_string()));
                });
                row.col(|ui| {
                    ui.label(format_time(entry.modified));
                });
            });
        });
}

/// Centered hint shown before an archive is open.
pub fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(72.0);
        icons::show(ui, Icon::FileArchive, MUTED, 48.0);
        ui.add_space(12.0);
        ui.label("Open a ZIP, tar, gzip, or zstd archive");
        ui.label(egui::RichText::new("or drop one here").color(MUTED));
    });
}

/// Error line with a dismiss control.
pub fn error_banner(ui: &mut egui::Ui, message: &str) -> bool {
    let mut dismiss = false;
    ui.horizontal(|ui| {
        icons::show(ui, Icon::TriangleAlert, DANGER, 16.0);
        ui.colored_label(DANGER, message);
        if ui.button("Dismiss").clicked() {
            dismiss = true;
        }
        icons::show(ui, Icon::X, DANGER, 14.0);
    });
    dismiss
}

/// Bottom line: format, counts, progress, and the icon credit.
pub fn status(
    ui: &mut egui::Ui,
    format: &str,
    count: usize,
    selected: usize,
    busy: bool,
    summary: &str,
    progress: (String, String, u64, u64),
) {
    ui.horizontal(|ui| {
        if busy {
            icons::show_spinning(ui, INK, 14.0);
        } else if summary.starts_with("OK") {
            icons::show(ui, Icon::CircleCheck, OK, 14.0);
        } else {
            icons::show(ui, Icon::Archive, MUTED, 14.0);
        }
        if !format.is_empty() {
            ui.label(format);
            ui.label(egui::RichText::new(format!("{count} entries")).color(MUTED));
            if selected > 0 {
                ui.label(egui::RichText::new(format!("{selected} selected")).color(MUTED));
            }
        }
        let (label, entry, done, total) = progress;
        if busy {
            let fraction = if total == 0 {
                0.0
            } else {
                (done as f32 / total as f32).clamp(0.0, 1.0)
            };
            ui.add(egui::ProgressBar::new(fraction).desired_width(160.0));
            let text = if entry.is_empty() { label } else { entry };
            ui.label(egui::RichText::new(text).color(MUTED));
        } else if !summary.is_empty() {
            ui.label(summary);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.hyperlink_to(
                egui::RichText::new("Lucide").color(MUTED),
                "https://lucide.dev/icons/",
            );
        });
    });
}

fn kind_icon(kind: EntryKind) -> Icon {
    match kind {
        EntryKind::Dir => Icon::Folder,
        EntryKind::Symlink => Icon::Link,
        EntryKind::File | EntryKind::Other => Icon::File,
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
