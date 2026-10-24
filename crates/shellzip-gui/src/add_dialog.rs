//! Add dialog: archive name, update mode, format, compression, and comment.
//!
//! The two pages follow the WinRAR archive dialog. ZIP is the default format.
//! Compression level and the ZIP method live here, not in the Options menu.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};

use shellzip_core::{UpdateMode, ZipMethod, creatable_formats};

use crate::chrome::PackSettings;
use crate::theme::{DANGER, MUTED};

/// Which page of the add dialog is visible.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    /// Name, format, compression, and update mode.
    General,
    /// Archive comment.
    Comment,
}

/// Choices confirmed from the dialog. The window turns this into a pack job.
pub struct AddRequest {
    /// Destination path, including the extension.
    pub dest: PathBuf,
    /// Files and folders to pack.
    pub inputs: Vec<PathBuf>,
    /// Compression level, 0 through 5.
    pub level: u8,
    /// ZIP method. Ignored for other formats.
    pub method: ZipMethod,
    /// How an existing ZIP is merged.
    pub update: UpdateMode,
    /// Wrap the ZIP as a self-extractor.
    pub sfx: bool,
    /// Empty means no password.
    pub password: String,
    /// `None` leaves an existing ZIP comment unchanged.
    pub comment: Option<String>,
}

/// What the user did with the dialog this frame.
pub enum AddAction {
    /// OK. The dialog should close.
    Submit(AddRequest),
    /// Cancel or the window was closed.
    Close,
}

/// Open add dialog.
pub struct AddDialog {
    tab: Tab,
    name: String,
    format: usize,
    update: UpdateMode,
    level: u8,
    method: ZipMethod,
    sfx: bool,
    password: String,
    comment_path: String,
    comment: String,
    /// Set when the user edits the comment, so an empty box can clear one.
    comment_touched: bool,
    inputs: Vec<PathBuf>,
    help: bool,
    notice: Option<String>,
}

impl AddDialog {
    /// Dialog for packing `inputs`, starting as a ZIP in `dir`.
    pub fn prepare(dir: &Path, inputs: Vec<PathBuf>, options: &PackSettings) -> Self {
        Self {
            tab: Tab::General,
            name: suggested_name(dir, &inputs),
            format: 0,
            update: UpdateMode::Add,
            level: options.level.min(5),
            method: options.method,
            sfx: false,
            password: String::new(),
            comment_path: String::new(),
            comment: String::new(),
            comment_touched: false,
            inputs,
            help: false,
            notice: None,
        }
    }
}

/// Paint the dialog. `Some` means it should close.
pub fn show(ctx: &egui::Context, dialog: &mut AddDialog) -> Option<AddAction> {
    let mut open = true;
    let mut action = None;
    egui::Window::new("Archive name and parameters")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(560.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_size(egui::vec2(520.0, 380.0));
            tabs(ui, dialog);
            ui.separator();
            match dialog.tab {
                Tab::General => general_tab(ui, dialog),
                Tab::Comment => comment_tab(ui, dialog),
            }
            if let Some(notice) = &dialog.notice {
                ui.add_space(4.0);
                ui.colored_label(DANGER, notice);
            }
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("OK").clicked()
                    && let Some(request) = dialog.request()
                {
                    action = Some(AddAction::Submit(request));
                }
                if ui.button("Cancel").clicked() {
                    action = Some(AddAction::Close);
                }
                if ui.button("Help").clicked() {
                    dialog.help = true;
                }
            });
        });
    help_window(ctx, dialog);
    if !open {
        return Some(AddAction::Close);
    }
    action
}

/// Answer for one existing archive member.
#[derive(Clone, Copy)]
pub enum AskReply {
    /// Replace this member.
    Replace,
    /// Replace this member and every later collision.
    ReplaceAll,
    /// Leave this member.
    Skip,
    /// Leave this member and every later collision.
    SkipAll,
    /// Store the source under a new name.
    Rename,
    /// Stop the pack.
    Cancel,
}

/// Worker questions and the reply channel, owned by the window.
pub struct AskPrompt {
    /// Member names the worker wants a decision for.
    pub names: Receiver<String>,
    /// Decisions sent back to the worker.
    pub replies: Sender<AskReply>,
    /// Name currently on screen.
    pub pending: Option<String>,
}

/// Ask about one existing member. Closing the window cancels the pack.
pub fn ask_overwrite(ctx: &egui::Context, name: &str) -> Option<AskReply> {
    let mut open = true;
    let mut reply = None;
    egui::Window::new("Confirm file replace")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.label(egui::RichText::new(name).strong());
            ui.label("This name is already in the archive.");
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Replace").clicked() {
                    reply = Some(AskReply::Replace);
                }
                if ui.button("Replace all").clicked() {
                    reply = Some(AskReply::ReplaceAll);
                }
                if ui.button("Skip").clicked() {
                    reply = Some(AskReply::Skip);
                }
                if ui.button("Skip all").clicked() {
                    reply = Some(AskReply::SkipAll);
                }
                if ui.button("Rename").clicked() {
                    reply = Some(AskReply::Rename);
                }
                if ui.button("Cancel").clicked() {
                    reply = Some(AskReply::Cancel);
                }
            });
        });
    if !open { Some(AskReply::Cancel) } else { reply }
}

fn tabs(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(dialog.tab == Tab::General, "General")
            .clicked()
        {
            dialog.tab = Tab::General;
        }
        if ui
            .selectable_label(dialog.tab == Tab::Comment, "Comment")
            .clicked()
        {
            dialog.tab = Tab::Comment;
        }
    });
}

fn general_tab(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    ui.horizontal(|ui| {
        ui.label("Archive name");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Browse...").clicked() {
                browse_archive(dialog);
            }
        });
    });
    ui.add(
        egui::TextEdit::singleline(&mut dialog.name)
            .desired_width(f32::INFINITY)
            .id_salt("shellzip-archive-name"),
    );
    ui.add_space(8.0);
    ui.label("Update mode");
    update_combo(ui, dialog);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(240.0);
            ui.label("Archive format");
            format_combo(ui, dialog);
        });
        ui.vertical(|ui| {
            ui.label("Compression method");
            level_combo(ui, dialog);
        });
    });
    ui.add_space(6.0);
    method_row(ui, dialog);
    ui.add_space(8.0);
    let mut sfx = dialog.sfx;
    if ui.checkbox(&mut sfx, "Create SFX archive").changed() {
        dialog.set_sfx(sfx);
    }
    ui.label(
        egui::RichText::new(
            "An SFX package is a ZIP wrapped in the ShellZip extractor. A password is requested when that package is opened.",
        )
        .color(MUTED),
    );
    ui.add_space(6.0);
    ui.label("Password");
    let zip = dialog.stores_zip();
    ui.add_enabled_ui(zip, |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut dialog.password)
                .password(true)
                .desired_width(f32::INFINITY)
                .hint_text(if zip {
                    "Leave empty for no encryption"
                } else {
                    "ZIP archives only"
                })
                .id_salt("shellzip-archive-password"),
        );
    });
}

fn comment_tab(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    ui.horizontal(|ui| {
        ui.label("Load a comment from the file");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Browse...").clicked() {
                browse_comment(dialog);
            }
        });
    });
    ui.add(
        egui::TextEdit::singleline(&mut dialog.comment_path)
            .desired_width(f32::INFINITY)
            .interactive(false)
            .id_salt("shellzip-comment-path"),
    );
    ui.add_space(8.0);
    ui.label("Enter a comment manually");
    if !dialog.stores_zip() {
        ui.label(egui::RichText::new("Comments are stored in ZIP archives.").color(MUTED));
    }
    let response = ui.add_sized(
        egui::vec2(ui.available_width(), 180.0),
        egui::TextEdit::multiline(&mut dialog.comment).id_salt("shellzip-comment"),
    );
    if response.changed() {
        dialog.comment_touched = true;
    }
}

fn help_window(ctx: &egui::Context, dialog: &mut AddDialog) {
    if !dialog.help {
        return;
    }
    let mut open = true;
    egui::Window::new("Add to archive")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            for mode in UpdateMode::ALL {
                ui.label(egui::RichText::new(mode.label()).strong());
                ui.label(mode_help(mode));
                ui.add_space(4.0);
            }
            ui.label("The comment is stored in a ZIP archive, including an SFX package.");
        });
    dialog.help = open;
}

fn update_combo(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    ui.add_enabled_ui(!dialog.sfx, |ui| {
        egui::ComboBox::from_id_salt("shellzip-update-mode")
            .selected_text(dialog.update.label())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for mode in UpdateMode::ALL {
                    ui.selectable_value(&mut dialog.update, mode, mode.label());
                }
            });
    });
}

fn format_combo(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    let formats = creatable_formats();
    let selected = formats
        .get(dialog.format)
        .map(|kind| kind.label)
        .unwrap_or("ZIP");
    ui.add_enabled_ui(!dialog.sfx, |ui| {
        egui::ComboBox::from_id_salt("shellzip-archive-format")
            .selected_text(selected)
            .width(220.0)
            .show_ui(ui, |ui| {
                for (index, kind) in formats.iter().enumerate() {
                    if ui
                        .selectable_label(dialog.format == index, kind.label)
                        .clicked()
                        && dialog.format != index
                    {
                        dialog.format = index;
                        dialog.name = with_extension(&dialog.name, kind.extension);
                    }
                }
            });
    });
}

fn level_combo(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    egui::ComboBox::from_id_salt("shellzip-compression")
        .selected_text(level_name(dialog.level))
        .width(180.0)
        .show_ui(ui, |ui| {
            for level in 0..=5 {
                ui.selectable_value(&mut dialog.level, level, level_name(level));
            }
        });
}

fn method_row(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    let zip = dialog.stores_zip() && dialog.level > 0;
    ui.add_enabled_ui(zip, |ui| {
        ui.label("ZIP method");
        egui::ComboBox::from_id_salt("shellzip-zip-method")
            .selected_text(method_name(dialog.method))
            .width(180.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut dialog.method, ZipMethod::Deflate, "Deflate");
                ui.selectable_value(&mut dialog.method, ZipMethod::Zstd, "Zstd");
            });
    });
}

impl AddDialog {
    fn stores_zip(&self) -> bool {
        self.sfx
            || creatable_formats()
                .get(self.format)
                .is_some_and(|kind| kind.extension == "zip")
    }

    fn set_sfx(&mut self, sfx: bool) {
        self.sfx = sfx;
        if sfx {
            self.format = 0;
            self.update = UpdateMode::Add;
            let extension = if cfg!(windows) { "exe" } else { "sfx" };
            self.name = with_extension(&self.name, extension);
        } else {
            self.name = with_extension(&self.name, "zip");
        }
        self.notice = None;
    }

    fn request(&mut self) -> Option<AddRequest> {
        let name = self.name.trim();
        if name.is_empty() {
            self.notice = Some("Enter an archive name.".to_string());
            return None;
        }
        let dest = PathBuf::from(name);
        if dest.is_dir() {
            self.notice = Some("Choose a file name, not a folder.".to_string());
            return None;
        }
        if self.comment_touched && !self.comment.is_empty() && !self.stores_zip() {
            self.tab = Tab::Comment;
            self.notice = Some("Comments are stored in ZIP archives.".to_string());
            return None;
        }
        if self.comment.len() > usize::from(u16::MAX) {
            self.tab = Tab::Comment;
            self.notice = Some("A ZIP comment is limited to 65535 bytes.".to_string());
            return None;
        }
        if !self.password.is_empty() && !self.stores_zip() {
            self.notice = Some("Passwords encrypt ZIP archives.".to_string());
            return None;
        }
        if self.password.as_bytes().contains(&0) {
            self.notice = Some("The password cannot contain a NUL byte.".to_string());
            return None;
        }
        self.notice = None;
        let comment = if self.comment_touched && self.stores_zip() {
            Some(self.comment.clone())
        } else {
            None
        };
        Some(AddRequest {
            dest,
            inputs: self.inputs.clone(),
            level: self.level,
            method: self.method,
            update: if self.sfx {
                UpdateMode::Add
            } else {
                self.update
            },
            sfx: self.sfx,
            password: self.password.clone(),
            comment,
        })
    }
}

fn browse_archive(dialog: &mut AddDialog) {
    let current = PathBuf::from(&dialog.name);
    let mut picker = rfd::FileDialog::new();
    if let Some(parent) = current.parent().filter(|parent| parent.is_dir()) {
        picker = picker.set_directory(parent);
    }
    if let Some(file_name) = current.file_name().and_then(|name| name.to_str()) {
        picker = picker.set_file_name(file_name);
    }
    let Some(path) = picker.save_file() else {
        return;
    };
    dialog.name = path.display().to_string();
    dialog.notice = None;
    adopt_extension(dialog);
}

fn browse_comment(dialog: &mut AddDialog) {
    let Some(path) = rfd::FileDialog::new().pick_file() else {
        return;
    };
    match read_comment_file(&path) {
        Ok(text) => {
            dialog.comment = text;
            dialog.comment_path = path.display().to_string();
            dialog.comment_touched = true;
            dialog.notice = None;
        }
        Err(message) => dialog.notice = Some(message),
    }
}

fn adopt_extension(dialog: &mut AddDialog) {
    let name = dialog.name.to_ascii_lowercase();
    if name.ends_with(".exe") || name.ends_with(".sfx") {
        dialog.sfx = true;
        dialog.format = 0;
        dialog.update = UpdateMode::Add;
        return;
    }
    dialog.sfx = false;
    for (index, kind) in creatable_formats().iter().enumerate() {
        if name.ends_with(&format!(".{}", kind.extension)) {
            dialog.format = index;
            return;
        }
    }
}

fn read_comment_file(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path)
        .map_err(|err| format!("cannot read {}: {err}", path.display()))?;
    // Stop at the ZIP comment limit from the bytes actually read. A size check
    // before the read would miss a file that grows in between.
    let mut bytes = Vec::new();
    file.take(u64::from(u16::MAX) + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("cannot read {}: {err}", path.display()))?;
    if bytes.len() > usize::from(u16::MAX) {
        return Err("Comment files are limited to 65535 bytes.".to_string());
    }
    String::from_utf8(bytes).map_err(|_| "The comment file is not UTF-8.".to_string())
}

fn suggested_name(dir: &Path, inputs: &[PathBuf]) -> String {
    let stem = if inputs.len() == 1 {
        inputs[0]
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty())
    } else {
        None
    };
    let stem = stem.unwrap_or_else(|| {
        dir.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Archive".to_string())
    });
    dir.join(format!("{stem}.zip")).display().to_string()
}

/// Replace a known archive extension, keeping the directory.
pub(crate) fn with_extension(path: &str, extension: &str) -> String {
    let path = Path::new(path);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Archive".to_string());
    let mut stem = strip_archive_extension(&name);
    if stem.is_empty() {
        stem = "Archive".to_string();
    }
    let file = format!("{stem}.{extension}");
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(file).display().to_string(),
        _ => file,
    }
}

fn strip_archive_extension(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    for extension in [
        "tar.gz", "tar.zst", "tar.bz2", "tar.xz", "tgz", "tzst", "tbz2", "txz", "zip", "tar", "gz",
        "zst", "xz", "bz2", "7z", "rar", "exe", "sfx",
    ] {
        let suffix = format!(".{extension}");
        if lower.ends_with(&suffix) && name.len() > suffix.len() {
            return name[..name.len() - suffix.len()].to_string();
        }
    }
    name.to_string()
}

fn level_name(level: u8) -> &'static str {
    match level {
        0 => "Store",
        1 => "Fastest",
        2 => "Fast",
        3 => "Normal",
        4 => "Good",
        _ => "Best",
    }
}

fn method_name(method: ZipMethod) -> &'static str {
    match method {
        ZipMethod::Deflate => "Deflate",
        ZipMethod::Zstd => "Zstd",
    }
}

fn mode_help(mode: UpdateMode) -> &'static str {
    match mode {
        UpdateMode::Add => "Add every selected file and replace a member that has the same name.",
        UpdateMode::Update => {
            "Add new files, and replace a member only when the selected file is newer."
        }
        UpdateMode::Freshen => {
            "Replace a member only when it already exists and the selected file is newer."
        }
        UpdateMode::Ask => "Ask before replacing a member that already exists.",
        UpdateMode::Skip => {
            "Add files that are not in the archive, and leave existing members as they are."
        }
        UpdateMode::Sync => {
            "Add and update the selection, then remove archive members that were not selected."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::with_extension;

    #[test]
    fn extension_change_keeps_the_directory_and_stem() {
        let zip = with_extension("work/notes.tar.gz", "zip");
        let path = std::path::Path::new(&zip);
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("notes.zip")
        );
        let wrapped = with_extension("notes.zip", "tar.gz");
        assert_eq!(
            std::path::Path::new(&wrapped)
                .file_name()
                .and_then(|name| name.to_str()),
            Some("notes.tar.gz")
        );
        assert_eq!(
            std::path::Path::new(&with_extension("bundle.exe", "zip"))
                .file_name()
                .and_then(|name| name.to_str()),
            Some("bundle.zip")
        );
    }

    #[test]
    fn comment_file_stops_at_the_zip_limit() {
        let dir = std::env::temp_dir().join(format!(
            "shellzip-comment-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir(&dir).unwrap();
        let ok = dir.join("ok.txt");
        let big = dir.join("big.txt");
        std::fs::write(&ok, vec![b'a'; usize::from(u16::MAX)]).unwrap();
        std::fs::write(&big, vec![b'a'; usize::from(u16::MAX) + 1]).unwrap();
        assert_eq!(
            super::read_comment_file(&ok).unwrap().len(),
            usize::from(u16::MAX)
        );
        let err = super::read_comment_file(&big).unwrap_err();
        assert!(err.contains("65535"), "{err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
