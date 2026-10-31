//! About ShellZip: the project license, and every third-party crate in this build.

use egui_extras::{Column, TableBuilder};

use crate::credits::{self, Credit};
use crate::theme;

/// MIT License for the ShellZip source. Third-party components are listed separately.
const MIT_LICENSE: &str = "\
Copyright (c) 2026 Ajay Menon

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the \"Software\"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.";

/// Paragraph 2 of the UnRAR license. It has to travel with the extraction sources.
const UNRAR_NOTICE: &str = "\
UnRAR source code may be used in any software to handle RAR archives without \
limitations free of charge, but cannot be used to develop RAR (WinRAR) \
compatible archiver and to re-create RAR compression algorithm, which is \
proprietary. Distribution of modified UnRAR source code in separate form or \
as a part of other software is permitted, provided that full text of this \
paragraph, starting from \"UnRAR source code\" words, is included in license, \
or in documentation if license is not available, and in source code comments \
of resulting package.";

/// About window. `open` is cleared by the title-bar close button or Close.
pub fn show(ctx: &egui::Context, open: &mut bool) {
    let mut close = false;
    egui::Window::new("About ShellZip")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_size([640.0, 560.0])
        .min_width(520.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(520.0);
            identity(ui);
            ui.add_space(8.0);
            license(ui);
            ui.add_space(8.0);
            notices(ui);
            ui.add_space(8.0);
            ui.separator();
            let rows = credits::credits();
            ui.label(format!(
                "Third-party packages in this build ({})",
                rows.len()
            ));
            ui.add_space(4.0);
            package_table(ui, rows);
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    close = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Icons by Lucide (MIT)", "https://lucide.dev/icons/");
                });
            });
        });
    if close {
        *open = false;
    }
}

fn identity(ui: &mut egui::Ui) {
    let colors = theme::palette(ui.ctx());
    ui.label(egui::RichText::new("ShellZip").strong().size(22.0));
    ui.label(
        egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION"))).color(colors.muted),
    );
    ui.add_space(6.0);
    ui.label(
        "Open, list, test, extract, and create ZIP, 7z, and tar archives. \
         RAR archives can be listed and extracted.",
    );
    ui.add_space(6.0);
    ui.label("Ajay Menon");
}

fn license(ui: &mut egui::Ui) {
    ui.label(
        "ShellZip is licensed under the MIT License. \
         You may also use it under the Apache License, Version 2.0.",
    );
    ui.label(
        "That license covers the ShellZip source. The packages below keep their own licenses.",
    );
    egui::CollapsingHeader::new("MIT License")
        .id_salt("shellzip-mit")
        .show(ui, |ui| {
            ui.label(MIT_LICENSE);
        });
}

fn notices(ui: &mut egui::Ui) {
    ui.label(
        "RAR listing and extraction include the UnRAR sources. \
         Those sources are not covered by the MIT License.",
    );
    egui::CollapsingHeader::new("UnRAR license")
        .id_salt("shellzip-unrar")
        .show(ui, |ui| {
            ui.label(UNRAR_NOTICE);
        });
    ui.label(
        "Interface fonts in epaint_default_fonts stay under the SIL Open Font License 1.1 \
         and the Ubuntu Font Licence. option-ext is MPL-2.0.",
    );
}

fn package_table(ui: &mut egui::Ui, rows: &[Credit]) {
    TableBuilder::new(ui)
        .id_salt("shellzip-about-credits")
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::initial(180.0).at_least(120.0).clip(true))
        .column(Column::initial(72.0).at_least(64.0).clip(true))
        .column(Column::remainder().at_least(180.0).clip(true))
        .min_scrolled_height(220.0)
        .max_scroll_height(280.0)
        .header(22.0, |mut header| {
            header.col(|ui| {
                ui.strong("Package");
            });
            header.col(|ui| {
                ui.strong("Version");
            });
            header.col(|ui| {
                ui.strong("License");
            });
        })
        .body(|body| {
            body.rows(20.0, rows.len(), |mut row| {
                let credit = &rows[row.index()];
                row.col(|ui| {
                    ui.label(credit.name);
                });
                row.col(|ui| {
                    ui.label(credit.version);
                });
                row.col(|ui| {
                    ui.label(credit.license).on_hover_text(credit.license);
                });
            });
        });
}
