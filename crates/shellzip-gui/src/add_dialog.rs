//! Add dialog: archive name, update mode, format, compression, and comment.
//!
//! The two pages follow the WinRAR archive dialog. ZIP is the default format.
//! Compression level and the ZIP method live here, not in the Options menu.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};

use shellzip_core::{
    SevenZMethod, SevenZOpts, SfxPayload, UpdateMode, ZipMethod, creatable_formats,
};

use crate::chrome::{self, PackSettings};
use crate::icons::Icon;
use crate::theme;

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
    /// 7z codec, solid blocks, and name encryption.
    pub sevenz: SevenZOpts,
    /// Payload when [`Self::sfx`] is set.
    pub sfx_payload: SfxPayload,
    /// `None` leaves an existing ZIP comment unchanged.
    pub comment: Option<String>,
    /// Hide the progress window and leave the file list usable.
    pub background: bool,
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
    sevenz_method: SevenZMethod,
    solid: bool,
    encrypt_headers: bool,
    sfx: bool,
    password: String,
    comment_path: String,
    comment: String,
    /// Set when the user edits the comment, so an empty box can clear one.
    comment_touched: bool,
    inputs: Vec<PathBuf>,
    background: bool,
    help: bool,
    notice: Option<String>,
    /// Open ZIP or 7z this dialog started on. Format, SFX, and the password
    /// field stay locked while the name field still matches it.
    bound: Option<BoundArchive>,
}

/// Archive the add dialog was opened against.
struct BoundArchive {
    /// Path text placed in the name field.
    name: String,
    /// Password that opened a 7z. Empty when that archive has none.
    ///
    /// Not shown in the dialog. A ZIP update does not use it.
    secret: String,
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
            sevenz_method: options.sevenz_method,
            solid: options.solid,
            encrypt_headers: false,
            sfx: false,
            password: String::new(),
            comment_path: String::new(),
            comment: String::new(),
            comment_touched: false,
            inputs,
            background: options.background,
            help: false,
            notice: None,
            bound: None,
        }
    }

    /// Dialog that adds `inputs` to an existing ZIP or 7z at `archive`.
    ///
    /// Format, SFX, and the password field stay locked while the name field is
    /// still that path. `secret` is the password that opened a 7z. It is not
    /// shown, and it is attached to the request only while the name is unchanged.
    pub fn prepare_into(
        archive: &Path,
        secret: Option<&str>,
        inputs: Vec<PathBuf>,
        options: &PackSettings,
    ) -> Self {
        let parent = archive
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut dialog = Self::prepare(parent, inputs, options);
        dialog.name = archive.display().to_string();
        adopt_extension(&mut dialog);
        dialog.sfx = false;
        dialog.password.clear();
        dialog.bound = Some(BoundArchive {
            name: dialog.name.clone(),
            secret: secret.unwrap_or("").to_string(),
        });
        dialog
    }
}

/// Paint the dialog. `Some` means it should close.
pub fn show(ctx: &egui::Context, dialog: &mut AddDialog) -> Option<AddAction> {
    let mut action = None;
    let result = chrome::modal(ctx, "shellzip-add", "Archive name and parameters", |ui| {
        ui.set_min_size(egui::vec2(520.0, 380.0));
        tabs(ui, dialog);
        ui.separator();
        match dialog.tab {
            Tab::General => general_tab(ui, dialog),
            Tab::Comment => comment_tab(ui, dialog),
        }
        if let Some(notice) = &dialog.notice {
            ui.add_space(4.0);
            ui.colored_label(theme::palette(ui.ctx()).danger, notice);
        }
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let colors = theme::palette(ui.ctx());
                if chrome::labeled_button(ui, Icon::CircleHelp, colors.open, "Help").clicked() {
                    dialog.help = true;
                }
                if chrome::labeled_button(ui, Icon::X, colors.danger, "Cancel").clicked() {
                    action = Some(AddAction::Close);
                }
                if chrome::labeled_button(ui, Icon::Check, colors.ok, "OK").clicked()
                    && let Some(request) = dialog.request()
                {
                    action = Some(AddAction::Submit(request));
                }
            });
        });
    });
    help_window(ctx, dialog);
    if action.is_none() && result.escape {
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
    /// Sentence under the name.
    pub detail: &'static str,
}

/// Ask about one existing name. Escape cancels the job.
pub fn ask_overwrite(ctx: &egui::Context, name: &str, detail: &str) -> Option<AskReply> {
    let mut reply = None;
    let result = chrome::modal(
        ctx,
        "shellzip-ask-overwrite",
        "Confirm file replace",
        |ui| {
            ui.set_min_width(420.0);
            ui.label(egui::RichText::new(name).strong());
            ui.label(detail);
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
            });
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if chrome::labeled_button(ui, Icon::X, theme::palette(ui.ctx()).ink, "Cancel")
                        .clicked()
                    {
                        reply = Some(AskReply::Cancel);
                    }
                });
            });
        },
    );
    if reply.is_none() && result.escape {
        Some(AskReply::Cancel)
    } else {
        reply
    }
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
    if dialog.still_bound() {
        let muted = theme::palette(ui.ctx()).muted;
        ui.label(egui::RichText::new("Files are added to this archive.").color(muted));
        if dialog.kind() == DialogFormat::Zip {
            ui.label(
                egui::RichText::new(
                    "Existing members are kept as they are. New files are stored without a password.",
                )
                .color(muted),
            );
        } else if dialog
            .bound
            .as_ref()
            .is_some_and(|bound| !bound.secret.is_empty())
        {
            ui.label(
                egui::RichText::new(
                    "This archive stays encrypted with the password that opened it.",
                )
                .color(muted),
            );
        }
    }
    ui.add_space(8.0);
    ui.label("Update mode");
    update_combo(ui, dialog);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(240.0);
            ui.label("Archive format");
            ui.add_enabled_ui(!dialog.still_bound(), |ui| {
                format_combo(ui, dialog);
            });
        });
        ui.vertical(|ui| {
            ui.label("Compression method");
            level_combo(ui, dialog);
        });
    });
    ui.add_space(6.0);
    method_row(ui, dialog);
    sevenz_options(ui, dialog);
    ui.add_space(8.0);
    ui.add_enabled_ui(!dialog.still_bound(), |ui| {
        let mut sfx = dialog.sfx;
        if ui.checkbox(&mut sfx, "Create SFX archive").changed() {
            dialog.set_sfx(sfx);
        }
    });
    ui.label(
        egui::RichText::new(
            "An SFX package is a ZIP or 7z archive wrapped in the ShellZip extractor. A password is requested when that package is opened.",
        )
        .color(theme::palette(ui.ctx()).muted),
    );
    ui.add_space(6.0);
    ui.label("Password");
    let can_encrypt = dialog.stores_comment();
    ui.add_enabled_ui(can_encrypt && !dialog.still_bound(), |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut dialog.password)
                .password(true)
                .desired_width(f32::INFINITY)
                .hint_text(if can_encrypt {
                    "Leave empty for no encryption"
                } else {
                    "ZIP and 7z archives only"
                })
                .id_salt("shellzip-archive-password"),
        );
    });
    ui.add_space(6.0);
    ui.checkbox(&mut dialog.background, "Start in background");
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
    if !dialog.stores_comment() {
        ui.label(
            egui::RichText::new("Comments are stored in ZIP and 7z archives.")
                .color(theme::palette(ui.ctx()).muted),
        );
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
    let result = chrome::modal(ctx, "shellzip-add-help", "Add to archive", |ui| {
        ui.set_min_width(420.0);
        for mode in UpdateMode::ALL {
            ui.label(egui::RichText::new(mode.label()).strong());
            ui.label(mode_help(mode));
            ui.add_space(4.0);
        }
        ui.label("The comment is stored in a ZIP or 7z archive, including an SFX package.");
        ui.add_space(8.0);
        ui.separator();
        let mut close = false;
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if chrome::labeled_button(ui, Icon::X, theme::palette(ui.ctx()).ink, "Close")
                    .clicked()
                {
                    close = true;
                }
            });
        });
        close
    });
    if result.inner || result.escape {
        dialog.help = false;
    }
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
    egui::ComboBox::from_id_salt("shellzip-archive-format")
        .selected_text(selected)
        .width(220.0)
        .show_ui(ui, |ui| {
            for (index, kind) in formats.iter().enumerate() {
                if dialog.sfx && kind.extension != "zip" && kind.extension != "7z" {
                    continue;
                }
                if ui
                    .selectable_label(dialog.format == index, kind.label)
                    .clicked()
                    && dialog.format != index
                {
                    dialog.format = index;
                    if !dialog.sfx {
                        dialog.name = with_extension(&dialog.name, kind.extension);
                    }
                }
            }
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
    let compressing = dialog.level > 0;
    match dialog.kind() {
        DialogFormat::Zip => {
            ui.add_enabled_ui(compressing, |ui| {
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
        DialogFormat::SevenZ => {
            ui.add_enabled_ui(compressing, |ui| {
                ui.label("7z method");
                egui::ComboBox::from_id_salt("shellzip-7z-method")
                    .selected_text(dialog.sevenz_method.label())
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for method in [
                            SevenZMethod::Lzma2,
                            SevenZMethod::Lzma,
                            SevenZMethod::Ppmd,
                            SevenZMethod::Bzip2,
                            SevenZMethod::Deflate,
                            SevenZMethod::Zstd,
                        ] {
                            ui.selectable_value(&mut dialog.sevenz_method, method, method.label());
                        }
                    });
            });
        }
        DialogFormat::Other => {}
    }
}

fn sevenz_options(ui: &mut egui::Ui, dialog: &mut AddDialog) {
    let sevenz = dialog.kind() == DialogFormat::SevenZ;
    ui.add_enabled_ui(sevenz, |ui| {
        ui.checkbox(&mut dialog.solid, "Solid archive");
        ui.checkbox(&mut dialog.encrypt_headers, "Encrypt file names");
    });
}

/// Which creatable container the dialog is aimed at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogFormat {
    Zip,
    SevenZ,
    Other,
}

impl AddDialog {
    fn kind(&self) -> DialogFormat {
        match creatable_formats()
            .get(self.format)
            .map(|kind| kind.extension)
        {
            Some("zip") => DialogFormat::Zip,
            Some("7z") => DialogFormat::SevenZ,
            _ => DialogFormat::Other,
        }
    }

    fn stores_comment(&self) -> bool {
        matches!(self.kind(), DialogFormat::Zip | DialogFormat::SevenZ)
    }

    /// The name field is still the archive this dialog was opened against.
    fn still_bound(&self) -> bool {
        self.bound
            .as_ref()
            .is_some_and(|bound| self.name.trim() == bound.name)
    }

    /// Password sent with the request.
    ///
    /// A bound 7z contributes the password that opened it, and only while the
    /// name is unchanged. Every other case uses the password field.
    fn effective_password(&self) -> String {
        if self.still_bound() {
            if self.kind() == DialogFormat::SevenZ {
                return self
                    .bound
                    .as_ref()
                    .map(|bound| bound.secret.clone())
                    .unwrap_or_default();
            }
            return String::new();
        }
        self.password.clone()
    }

    fn set_sfx(&mut self, sfx: bool) {
        self.sfx = sfx;
        if sfx {
            if !matches!(self.kind(), DialogFormat::Zip | DialogFormat::SevenZ) {
                self.format = 0;
            }
            self.update = UpdateMode::Add;
            let extension = if cfg!(windows) { "exe" } else { "sfx" };
            self.name = with_extension(&self.name, extension);
        } else {
            let extension = creatable_formats()
                .get(self.format)
                .map(|kind| kind.extension)
                .unwrap_or("zip");
            self.name = with_extension(&self.name, extension);
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
        if self.comment_touched && !self.comment.is_empty() && !self.stores_comment() {
            self.tab = Tab::Comment;
            self.notice = Some("Comments are stored in ZIP and 7z archives.".to_string());
            return None;
        }
        if self.comment.len() > usize::from(u16::MAX) {
            self.tab = Tab::Comment;
            self.notice = Some("A ZIP comment is limited to 65535 bytes.".to_string());
            return None;
        }
        let password = self.effective_password();
        if !password.is_empty() && !self.stores_comment() {
            self.notice = Some("Passwords encrypt ZIP and 7z archives.".to_string());
            return None;
        }
        if self.encrypt_headers && self.kind() == DialogFormat::SevenZ && password.is_empty() {
            self.notice = Some("Encrypting file names needs a password.".to_string());
            return None;
        }
        if password.as_bytes().contains(&0) {
            self.notice = Some("The password cannot contain a NUL byte.".to_string());
            return None;
        }
        self.notice = None;
        let comment = if self.comment_touched && self.stores_comment() {
            Some(self.comment.clone())
        } else {
            None
        };
        let sfx = self.sfx && !self.still_bound();
        Some(AddRequest {
            dest,
            inputs: self.inputs.clone(),
            level: self.level,
            method: self.method,
            update: if sfx { UpdateMode::Add } else { self.update },
            sfx,
            password,
            sevenz: SevenZOpts {
                method: self.sevenz_method,
                solid: self.solid,
                encrypt_headers: self.encrypt_headers && self.kind() == DialogFormat::SevenZ,
            },
            sfx_payload: if self.kind() == DialogFormat::SevenZ {
                SfxPayload::SevenZ
            } else {
                SfxPayload::Zip
            },
            comment,
            background: self.background,
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
        if !matches!(dialog.kind(), DialogFormat::Zip | DialogFormat::SevenZ) {
            dialog.format = 0;
        }
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
    use std::path::{Path, PathBuf};

    use shellzip_core::{SevenZMethod, ZipMethod};

    use super::{AddDialog, DialogFormat, with_extension};
    use crate::chrome::PackSettings;

    fn pack() -> PackSettings {
        PackSettings {
            level: 3,
            method: ZipMethod::Deflate,
            sevenz_method: SevenZMethod::Lzma2,
            solid: true,
            background: false,
        }
    }

    #[test]
    fn prepare_into_keeps_the_archive_and_hides_its_password() {
        let archive = Path::new("work/notes.7z");
        let mut dialog = AddDialog::prepare_into(
            archive,
            Some("szip-fixture"),
            vec![PathBuf::from("extra.txt")],
            &pack(),
        );
        assert_eq!(dialog.name, archive.display().to_string());
        assert_eq!(dialog.kind(), DialogFormat::SevenZ);
        assert!(!dialog.sfx);
        assert!(dialog.password.is_empty());
        let request = dialog.request().expect("bound request");
        assert_eq!(request.password, "szip-fixture");
        assert!(!request.sfx);
        assert_eq!(request.dest, PathBuf::from(archive.display().to_string()));

        dialog.name = "work/other.zip".to_string();
        let moved = dialog.request().expect("renamed request");
        assert!(
            moved.password.is_empty(),
            "a new name must not keep the 7z password"
        );
        assert_eq!(moved.dest, PathBuf::from("work/other.zip"));

        dialog.name = archive.display().to_string();
        let restored = dialog.request().expect("restored request");
        assert_eq!(restored.password, "szip-fixture");
    }

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
