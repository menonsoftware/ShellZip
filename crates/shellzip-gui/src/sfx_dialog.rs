//! Self-extractor options dialog.
//!
//! Confirms the output path. Background is offered only when unwrapping an
//! existing ShellZip self-extractor; wrapping stays in the foreground.

use std::path::{Path, PathBuf};

use crate::chrome;
use crate::icons::Icon;
use crate::settings::SfxSettings;
use crate::theme;

fn colors(ui: &egui::Ui) -> theme::Palette {
    *theme::palette(ui.ctx())
}

/// Choices confirmed from the dialog.
pub struct SfxRequest {
    /// ZIP or 7z to wrap, or an SFX package to unwrap.
    pub source: PathBuf,
    /// Destination for the self-extractor or stripped archive.
    pub dest: PathBuf,
    /// Hide the progress window and leave the file list usable.
    pub background: bool,
}

/// What the user did with the dialog this frame.
pub enum SfxAction {
    /// OK. The dialog should close and the conversion should start.
    Submit(SfxRequest),
    /// Save settings. The dialog stays open.
    Save(SfxSettings),
    /// Cancel or the window was closed.
    Close,
}

/// Open self-extractor dialog.
pub struct SfxDialog {
    source: PathBuf,
    path: String,
    settings: SfxSettings,
    saved: SfxSettings,
    /// `true` when unwrapping; wrapping keeps the progress window.
    allows_background: bool,
    help: bool,
    notice: Option<String>,
}

impl SfxDialog {
    /// Dialog for converting `source`, with `suggested` as the default output.
    ///
    /// `allows_background` is set when `source` is already a ShellZip SFX.
    pub fn prepare(
        source: &Path,
        suggested: &Path,
        settings: SfxSettings,
        allows_background: bool,
    ) -> Self {
        Self {
            source: source.to_path_buf(),
            path: suggested.display().to_string(),
            settings,
            saved: settings,
            allows_background,
            help: false,
            notice: None,
        }
    }

    fn request(&mut self) -> Option<SfxRequest> {
        let dest = PathBuf::from(self.path.trim());
        if dest.as_os_str().is_empty() {
            self.notice = Some("Choose an output path.".to_string());
            return None;
        }
        if same_path(&self.source, &dest) {
            self.notice = Some("Choose a different path than the open archive.".to_string());
            return None;
        }
        self.notice = None;
        Some(SfxRequest {
            source: self.source.clone(),
            dest,
            background: self.allows_background && self.settings.background,
        })
    }
}

/// Paint the dialog. `Some` means the caller should act. Save does not close it.
pub fn show(ctx: &egui::Context, dialog: &mut SfxDialog) -> Option<SfxAction> {
    let mut action = None;
    let result = chrome::modal(ctx, "shellzip-sfx", "Self-extractor options", |ui| {
        ui.set_min_size(egui::vec2(520.0, 220.0));
        ui.label(
            egui::RichText::new(format!("Archive: {}", dialog.source.display()))
                .color(colors(ui).muted)
                .small(),
        );
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Output").strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut dialog.path)
                    .desired_width(360.0)
                    .hint_text("Path for the self-extractor"),
            );
            if ui.button("Browse…").clicked() {
                browse(dialog);
            }
        });
        if dialog.allows_background {
            ui.add_space(12.0);
            ui.group(|ui| {
                ui.set_min_width(360.0);
                ui.label(egui::RichText::new("Background").strong());
                ui.checkbox(&mut dialog.settings.background, "Start in background");
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "The file list stays usable. Progress stays on the status bar.",
                    )
                    .color(colors(ui).muted)
                    .small(),
                );
            });
        } else {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(
                    "Creating a self-extractor keeps the progress window open until it finishes.",
                )
                .color(colors(ui).muted)
                .small(),
            );
        }
        if let Some(notice) = &dialog.notice {
            ui.add_space(4.0);
            ui.colored_label(colors(ui).danger, notice);
        }
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            if dialog.allows_background {
                let dirty = dialog.settings != dialog.saved;
                if chrome::icon_button(ui, Icon::Save, colors(ui).ink, "Save settings", dirty)
                    .clicked()
                {
                    dialog.saved = dialog.settings;
                    action = Some(SfxAction::Save(dialog.settings));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if chrome::labeled_button(ui, Icon::CircleHelp, colors(ui).open, "Help").clicked() {
                    dialog.help = true;
                }
                if chrome::labeled_button(ui, Icon::X, colors(ui).danger, "Cancel").clicked() {
                    action = Some(SfxAction::Close);
                }
                if chrome::labeled_button(ui, Icon::Check, colors(ui).ok, "OK").clicked()
                    && let Some(request) = dialog.request()
                {
                    action = Some(SfxAction::Submit(request));
                }
            });
        });
    });
    help_window(ctx, dialog);
    if action.is_none() && result.escape {
        return Some(SfxAction::Close);
    }
    action
}

fn browse(dialog: &mut SfxDialog) {
    let suggested = PathBuf::from(dialog.path.trim());
    let mut picker = rfd::FileDialog::new().set_title("Save self-extractor");
    if let Some(parent) = suggested.parent().filter(|parent| parent.is_dir()) {
        picker = picker.set_directory(parent);
    }
    if let Some(name) = suggested.file_name().and_then(|name| name.to_str()) {
        picker = picker.set_file_name(name);
    }
    if let Some(path) = picker.save_file() {
        dialog.path = path.display().to_string();
        dialog.notice = None;
    }
}

fn help_window(ctx: &egui::Context, dialog: &mut SfxDialog) {
    if !dialog.help {
        return;
    }
    let mut open = true;
    egui::Window::new("Self-extractor help")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_max_width(420.0);
            ui.label(
                "Wraps the open ZIP or 7z with the shellzip-sfx stub, or strips an existing ShellZip self-extractor back to its archive.",
            );
            ui.add_space(6.0);
            ui.label(
                "The stub must sit next to the ShellZip executable, or SZIP_SFX_STUB must point at it.",
            );
            ui.add_space(6.0);
            ui.label(
                "Windows cannot run a self-extractor of 4 GiB or larger. Keep very large archives as ZIP or 7z.",
            );
            ui.add_space(6.0);
            ui.label(
                "When unwrapping, Start in background leaves the file list usable and keeps progress on the status bar. Wrapping always shows the progress window.",
            );
        });
    if !open {
        dialog.help = false;
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    if let (Ok(left), Ok(right)) = (left.canonicalize(), right.canonicalize()) {
        return left == right;
    }
    left == right
}
