//! Extraction path and options dialog.
//!
//! The General page matches the WinRAR extract dialog: a destination, update
//! and overwrite choices, and a folder tree. Advanced and Options hold the
//! path mode, file time, and archive deletion choices ShellZip can honor.

use std::path::{Path, PathBuf};

use shellzip_core::{ExtractUpdate, Overwrite};

use crate::chrome;
use crate::explorer::{FolderTree, JumpTarget, TreeFocus};
use crate::icons::Icon;
use crate::settings::{DeleteArchive, ExtractSettings, PathMode};
use crate::shell_icons::IconCache;
use crate::theme;

fn colors(ui: &egui::Ui) -> theme::Palette {
    *theme::palette(ui.ctx())
}

/// Every option group uses this inner width, on every page.
const GROUP_INNER: f32 = 320.0;

/// Inner width of the extract dialog. The path field and folder tree stay inside it.
const DIALOG_WIDTH: f32 = 760.0;

/// Tab body height. General, Advanced, and Options share it so the footer stays put.
const TAB_BODY_HEIGHT: f32 = 380.0;

/// Which page of the extract dialog is visible.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    /// Destination, update mode, overwrite mode, and the folder tree.
    General,
    /// File time and path mode.
    Advanced,
    /// Deleting the archive after extraction.
    Options,
}

/// Choices confirmed from the dialog.
pub struct ExtractRequest {
    /// Archive to unpack.
    pub archive: PathBuf,
    /// Destination directory. It is created when missing.
    pub output: PathBuf,
    /// Replace, update, or freshen.
    pub update: ExtractUpdate,
    /// Collision policy.
    pub overwrite: Overwrite,
    /// Drop archive directories.
    pub flat: bool,
    /// Open archive folder removed from the front of each name.
    pub strip_prefix: String,
    /// Keep a partial file when decoding fails.
    pub keep_broken: bool,
    /// Restore modification times.
    pub restore_mtime: bool,
    /// Open the destination in Explorer afterwards.
    pub show_in_explorer: bool,
    /// Delete the archive after a clean full extract.
    pub delete: DeleteArchive,
    /// Move a deleted archive to the recycle bin.
    pub recycle: bool,
    /// Hide the progress window and leave the file list usable.
    pub background: bool,
    /// Member names, or every member when `None`.
    pub names: Option<Vec<String>>,
}

/// What the user did with the dialog this frame.
pub enum ExtractAction {
    /// OK. The dialog should close and the extract should start.
    Submit(ExtractRequest),
    /// Save settings. The dialog stays open.
    Save(ExtractSettings),
    /// Cancel or the window was closed.
    Close,
}

/// Open extract dialog.
pub struct ExtractDialog {
    tab: Tab,
    path: String,
    settings: ExtractSettings,
    saved: ExtractSettings,
    archive: PathBuf,
    names: Option<Vec<String>>,
    /// Archive folder currently open, used by relative paths.
    prefix: String,
    tree: FolderTree,
    focus: TreeFocus,
    /// Folder last chosen in the tree.
    selected: Option<PathBuf>,
    /// The tree was changed more recently than the path field.
    tree_touched: bool,
    help: bool,
    notice: Option<String>,
    /// Name prompt after **New folder**.
    new_folder: Option<NewFolderPrompt>,
}

struct NewFolderPrompt {
    parent: PathBuf,
    name: String,
    error: Option<String>,
}

impl ExtractDialog {
    /// Dialog for unpacking `archive`. `prefix` is the open archive folder.
    pub fn prepare(
        archive: &Path,
        names: Option<Vec<String>>,
        prefix: &str,
        settings: ExtractSettings,
    ) -> Self {
        let dest = suggested_destination(archive);
        let focus = TreeFocus::folder(&nearest_existing(&dest));
        let mut tree = FolderTree::new();
        tree.reveal(&focus);
        Self {
            tab: Tab::General,
            path: dest.display().to_string(),
            settings,
            saved: settings,
            archive: archive.to_path_buf(),
            names,
            prefix: prefix.trim_matches('/').to_string(),
            tree,
            focus,
            selected: None,
            tree_touched: false,
            help: false,
            notice: None,
            new_folder: None,
        }
    }

    fn request(&mut self) -> Option<ExtractRequest> {
        let text = self.path.trim();
        if text.is_empty() {
            self.notice = Some("Enter a destination path.".to_string());
            return None;
        }
        let output = PathBuf::from(text);
        if output.is_file() {
            self.notice = Some("The destination is a file. Choose a folder.".to_string());
            return None;
        }
        self.notice = None;
        let (flat, strip_prefix) = match self.settings.paths {
            PathMode::None => (true, String::new()),
            PathMode::Full => (false, String::new()),
            PathMode::Relative => (false, self.prefix.clone()),
        };
        Some(ExtractRequest {
            archive: self.archive.clone(),
            output,
            update: self.settings.update,
            overwrite: self.settings.overwrite,
            flat,
            strip_prefix,
            keep_broken: self.settings.keep_broken,
            restore_mtime: self.settings.restore_mtime,
            show_in_explorer: self.settings.show_in_explorer,
            delete: self.settings.delete,
            recycle: self.settings.recycle,
            background: self.settings.background,
            names: self.names.clone(),
        })
    }
}

/// Paint the dialog. `Some` means the caller should act. Save does not close it.
pub fn show(
    ctx: &egui::Context,
    dialog: &mut ExtractDialog,
    cache: &mut IconCache,
) -> Option<ExtractAction> {
    dialog.tree.poll();
    if dialog.tree.is_loading() {
        ctx.request_repaint();
    }
    let mut open = true;
    let mut action = None;
    egui::Window::new("Extraction path and options")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_width(DIALOG_WIDTH)
        .min_width(DIALOG_WIDTH)
        .max_width(DIALOG_WIDTH)
        .min_height(480.0)
        .default_height(480.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_width(DIALOG_WIDTH);
            ui.set_min_height(480.0);
            tabs(ui, dialog);
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), TAB_BODY_HEIGHT),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_height(TAB_BODY_HEIGHT);
                    match dialog.tab {
                        Tab::General => general_tab(ui, dialog, cache),
                        Tab::Advanced => advanced_tab(ui, dialog),
                        Tab::Options => options_tab(ui, dialog),
                    }
                    if let Some(notice) = &dialog.notice {
                        ui.add_space(4.0);
                        ui.colored_label(colors(ui).danger, notice);
                    }
                },
            );
            ui.add_space(8.0);
            ui.separator();
            ui.horizontal(|ui| {
                let dirty = dialog.settings != dialog.saved;
                if chrome::icon_button(ui, Icon::Save, colors(ui).ink, "Save settings", dirty)
                    .clicked()
                {
                    dialog.saved = dialog.settings;
                    action = Some(ExtractAction::Save(dialog.settings));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if chrome::labeled_button(ui, Icon::CircleHelp, colors(ui).open, "Help")
                        .clicked()
                    {
                        dialog.help = true;
                    }
                    if chrome::labeled_button(ui, Icon::X, colors(ui).danger, "Cancel").clicked() {
                        action = Some(ExtractAction::Close);
                    }
                    if chrome::labeled_button(ui, Icon::Check, colors(ui).ok, "OK").clicked()
                        && let Some(request) = dialog.request()
                    {
                        action = Some(ExtractAction::Submit(request));
                    }
                });
            });
        });
    help_window(ctx, dialog);
    new_folder_window(ctx, dialog);
    if !open {
        return Some(ExtractAction::Close);
    }
    action
}

/// Ask before deleting an unpacked archive. `true` means delete.
pub fn confirm_delete_archive(ctx: &egui::Context, path: &Path, open: &mut bool) -> bool {
    let mut confirmed = false;
    let mut dismiss = false;
    egui::Window::new("Delete archive")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.label("The archive was extracted. Delete it?");
            ui.label(egui::RichText::new(path.display().to_string()).strong());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if chrome::icon_button(ui, Icon::Trash, colors(ui).danger, "Delete", true).clicked()
                {
                    confirmed = true;
                }
                if chrome::icon_button(ui, Icon::X, colors(ui).ink, "Cancel", true).clicked() {
                    dismiss = true;
                }
            });
        });
    if confirmed || dismiss {
        *open = false;
    }
    confirmed
}

fn tabs(ui: &mut egui::Ui, dialog: &mut ExtractDialog) {
    ui.horizontal(|ui| {
        for (tab, label) in [
            (Tab::General, "General"),
            (Tab::Advanced, "Advanced"),
            (Tab::Options, "Options"),
        ] {
            if ui.selectable_label(dialog.tab == tab, label).clicked() {
                dialog.tab = tab;
            }
        }
    });
}

fn general_tab(ui: &mut egui::Ui, dialog: &mut ExtractDialog, cache: &mut IconCache) {
    ui.label("Destination path (will be created if does not exist)");
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let reserved = chrome::WELL_SIZE * 3.0 + ui.spacing().item_spacing.x * 2.0;
        let width = (ui.available_width() - reserved).max(48.0);
        let response = chrome::path_field(
            ui,
            &mut dialog.path,
            width,
            "Folder",
            "shellzip-extract-path",
        );
        if response.changed() {
            dialog.tree_touched = false;
        }
        if chrome::well_button(ui, Icon::FolderSearch, "Display", true).clicked() {
            display_path(dialog);
        }
        if chrome::well_button(ui, Icon::FolderPlus, "New folder", true).clicked() {
            begin_new_folder(dialog);
        }
        let busy = dialog.tree.is_loading();
        if chrome::well_button(ui, Icon::Refresh, "Refresh folders", !busy).clicked() {
            dialog.tree.refresh();
        }
    });
    ui.add_space(4.0);
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
        ui.vertical(|ui| {
            option_group(ui, "Update mode", |ui| {
                ui.radio_value(
                    &mut dialog.settings.update,
                    ExtractUpdate::Replace,
                    "Extract and replace files",
                );
                ui.radio_value(
                    &mut dialog.settings.update,
                    ExtractUpdate::Update,
                    "Extract and update files",
                );
                ui.radio_value(
                    &mut dialog.settings.update,
                    ExtractUpdate::Freshen,
                    "Freshen existing files only",
                );
            });
            option_group(ui, "Overwrite mode", |ui| {
                ui.radio_value(
                    &mut dialog.settings.overwrite,
                    Overwrite::Ask,
                    "Ask before overwrite",
                );
                ui.radio_value(
                    &mut dialog.settings.overwrite,
                    Overwrite::Always,
                    "Overwrite without prompt",
                );
                ui.radio_value(
                    &mut dialog.settings.overwrite,
                    Overwrite::Never,
                    "Skip existing files",
                );
                ui.radio_value(
                    &mut dialog.settings.overwrite,
                    Overwrite::Rename,
                    "Rename automatically",
                );
            });
            option_group(ui, "Miscellaneous", |ui| {
                ui.checkbox(&mut dialog.settings.keep_broken, "Keep broken files");
                ui.checkbox(
                    &mut dialog.settings.show_in_explorer,
                    "Display files in Explorer",
                );
            });
        });
        ui.vertical(|ui| {
            folder_tree(ui, dialog, cache);
        });
    });
}

fn advanced_tab(ui: &mut egui::Ui, dialog: &mut ExtractDialog) {
    option_group(ui, "File time", |ui| {
        ui.checkbox(&mut dialog.settings.restore_mtime, "Set modification time");
        ui.checkbox(&mut dialog.settings.restore_created, "Set creation time");
        ui.checkbox(
            &mut dialog.settings.restore_accessed,
            "Set last access time",
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Creation and last access time are remembered. Archives only provide the modification time, so those two are not applied.",
            )
            .color(colors(ui).muted)
            .small(),
        );
    });
    ui.add_space(8.0);
    option_group(ui, "File paths", |ui| {
        ui.radio_value(
            &mut dialog.settings.paths,
            PathMode::Relative,
            "Extract relative paths",
        );
        ui.radio_value(
            &mut dialog.settings.paths,
            PathMode::Full,
            "Extract full paths",
        );
        ui.radio_value(
            &mut dialog.settings.paths,
            PathMode::None,
            "Do not extract pathnames",
        );
    });
}

fn options_tab(ui: &mut egui::Ui, dialog: &mut ExtractDialog) {
    option_group(ui, "Delete archive", |ui| {
        ui.radio_value(&mut dialog.settings.delete, DeleteArchive::Never, "Never");
        ui.radio_value(
            &mut dialog.settings.delete,
            DeleteArchive::Ask,
            "Ask for confirmation",
        );
        ui.radio_value(&mut dialog.settings.delete, DeleteArchive::Always, "Always");
        ui.checkbox(&mut dialog.settings.recycle, "Move to Recycle Bin");
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "The archive is deleted only after every member is extracted with no warnings.",
            )
            .color(colors(ui).muted)
            .small(),
        );
    });
    ui.add_space(8.0);
    option_group(ui, "Background", |ui| {
        ui.checkbox(&mut dialog.settings.background, "Start in background");
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("The file list stays usable. Progress stays on the status bar.")
                .color(colors(ui).muted)
                .small(),
        );
    });
}

fn option_group(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    ui.group(|ui| {
        ui.set_min_width(GROUP_INNER);
        ui.set_max_width(GROUP_INNER);
        ui.label(egui::RichText::new(title).strong());
        body(ui);
    });
}

fn folder_tree(ui: &mut egui::Ui, dialog: &mut ExtractDialog, cache: &mut IconCache) {
    if let Some(message) = dialog.tree.loader_error() {
        ui.colored_label(colors(ui).danger, message);
    }
    let tree_width = DIALOG_WIDTH - GROUP_INNER - 24.0;
    ui.set_width(tree_width);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(320.0)
        .show(ui, |ui| {
            let row_width = ui.available_width().clamp(240.0, tree_width);
            ui.set_min_width(row_width);
            ui.set_min_height(280.0);
            let rows = dialog.tree.rows(&dialog.focus);
            for row in rows {
                if let Some(JumpTarget::Folder(path)) =
                    chrome::tree_row(ui, &mut dialog.tree, cache, &row)
                {
                    dialog.path = path.display().to_string();
                    dialog.selected = Some(path);
                    dialog.tree_touched = true;
                    dialog.notice = None;
                }
            }
        });
}

fn display_path(dialog: &mut ExtractDialog) {
    let typed = PathBuf::from(dialog.path.trim());
    if typed.as_os_str().is_empty() {
        dialog.notice = Some("Enter a destination path.".to_string());
        return;
    }
    let found = nearest_existing(&typed);
    dialog.focus = TreeFocus::folder(&found);
    dialog.tree.reveal(&dialog.focus);
    dialog.selected = Some(found);
    dialog.tree_touched = true;
    dialog.notice = None;
}

fn begin_new_folder(dialog: &mut ExtractDialog) {
    let parent = new_folder_parent(dialog);
    if !parent.is_dir() {
        dialog.notice = Some(format!("{} is not a folder.", parent.display()));
        return;
    }
    dialog.notice = None;
    let name = suggested_new_folder_name(&parent);
    dialog.new_folder = Some(NewFolderPrompt {
        parent,
        name,
        error: None,
    });
}

fn new_folder_window(ctx: &egui::Context, dialog: &mut ExtractDialog) {
    let Some(prompt) = dialog.new_folder.as_mut() else {
        return;
    };
    let mut open = true;
    let mut confirmed = false;
    let mut dismiss = false;
    let parent_label = prompt.parent.display().to_string();
    egui::Window::new("New folder")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            ui.label("Folder name");
            let response = ui.add(
                egui::TextEdit::singleline(&mut prompt.name)
                    .desired_width(f32::INFINITY)
                    .hint_text("Name"),
            );
            if response.changed() {
                prompt.error = None;
            }
            if ui.input(|input| input.key_pressed(egui::Key::Enter)) {
                confirmed = true;
            }
            ui.label(
                egui::RichText::new(format!("Create in {parent_label}"))
                    .color(colors(ui).muted)
                    .small(),
            );
            if let Some(error) = &prompt.error {
                ui.colored_label(colors(ui).danger, error);
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if chrome::icon_button(ui, Icon::Check, colors(ui).ok, "Create", true).clicked() {
                    confirmed = true;
                }
                if chrome::icon_button(ui, Icon::X, colors(ui).ink, "Cancel", true).clicked() {
                    dismiss = true;
                }
            });
        });
    if dismiss || !open {
        dialog.new_folder = None;
        return;
    }
    if confirmed {
        match confirm_new_folder(dialog) {
            Ok(()) => dialog.new_folder = None,
            Err(message) => {
                if let Some(prompt) = dialog.new_folder.as_mut() {
                    prompt.error = Some(message);
                }
            }
        }
    }
}

fn confirm_new_folder(dialog: &mut ExtractDialog) -> Result<(), String> {
    let prompt = dialog
        .new_folder
        .as_ref()
        .ok_or_else(|| "New folder prompt is missing.".to_string())?;
    let name = validate_folder_name(&prompt.name)?;
    let created = prompt.parent.join(name);
    if created.exists() {
        return Err(format!("{} already exists.", created.display()));
    }
    std::fs::create_dir(&created).map_err(|err| format!("create {}: {err}", created.display()))?;
    select_created_folder(dialog, &created);
    Ok(())
}

fn select_created_folder(dialog: &mut ExtractDialog, created: &Path) {
    if let Some(parent) = created.parent() {
        dialog.tree.forget(parent);
    }
    dialog.path = created.display().to_string();
    dialog.focus = TreeFocus::folder(created);
    dialog.tree.reveal(&dialog.focus);
    dialog.selected = Some(created.to_path_buf());
    dialog.tree_touched = true;
    dialog.notice = None;
}

const RESERVED_FOLDER_NAMES: &[&str] = &["CON", "PRN", "AUX", "NUL"];

fn validate_folder_name(raw: &str) -> Result<String, String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err("Enter a folder name.".to_string());
    }
    if name != name.trim_end() || name.ends_with('.') {
        return Err("Folder names cannot end with a space or period.".to_string());
    }
    if name.chars().any(char::is_control) {
        return Err("The name contains invalid characters.".to_string());
    }
    if name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("The name cannot contain \\ / : * ? \" < > |.".to_string());
    }
    if name == "." || name == ".." {
        return Err("That name is not allowed.".to_string());
    }
    if folder_name_is_reserved(name) {
        return Err("That name is reserved by the system.".to_string());
    }
    Ok(name.to_string())
}

fn folder_name_is_reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name);
    let upper = stem.to_ascii_uppercase();
    if RESERVED_FOLDER_NAMES.contains(&upper.as_str()) {
        return true;
    }
    let bytes = upper.as_bytes();
    bytes.len() == 4
        && (upper.starts_with("COM") || upper.starts_with("LPT"))
        && bytes[3].is_ascii_digit()
        && bytes[3] != b'0'
}

fn suggested_new_folder_name(parent: &Path) -> String {
    unused_folder(parent)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "New Folder".to_string())
}

fn new_folder_parent(dialog: &ExtractDialog) -> PathBuf {
    if dialog.tree_touched
        && let Some(selected) = &dialog.selected
        && selected.is_dir()
    {
        return selected.clone();
    }
    let typed = PathBuf::from(dialog.path.trim());
    if typed.is_dir() {
        return typed;
    }
    if let Some(parent) = typed.parent()
        && parent.is_dir()
    {
        return parent.to_path_buf();
    }
    typed
}

fn unused_folder(parent: &Path) -> PathBuf {
    let first = parent.join("New Folder");
    if !first.exists() {
        return first;
    }
    for index in 2..10_000 {
        let candidate = parent.join(format!("New Folder ({index})"));
        if !candidate.exists() {
            return candidate;
        }
    }
    parent.join(format!("New Folder ({})", std::process::id()))
}

fn nearest_existing(path: &Path) -> PathBuf {
    let mut cursor = path.to_path_buf();
    loop {
        if cursor.is_dir() {
            return cursor;
        }
        if !cursor.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Folder named after the archive, beside the archive.
pub fn suggested_destination(archive: &Path) -> PathBuf {
    let stem = archive
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "extracted".to_string());
    match archive.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(stem),
        _ => PathBuf::from(stem),
    }
}

fn help_window(ctx: &egui::Context, dialog: &mut ExtractDialog) {
    if !dialog.help {
        return;
    }
    let mut open = true;
    egui::Window::new("Extraction path and options help")
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(460.0);
            ui.label(egui::RichText::new("General").strong());
            ui.label("The destination is created if it does not exist. Display finds that folder in the tree. New folder asks for a name, creates it under the folder you selected or typed last, and switches the destination to it.");
            ui.label("Extract and replace writes every selected file. Extract and update also writes missing files, and replaces a file only when the archive copy is newer. Freshen only replaces an older file that is already there.");
            ui.label("Ask before overwrite prompts for each existing file. Rename writes name (n).ext beside it. Keep broken files leaves a partial file when decoding fails. Display files in Explorer opens the destination when the extract finishes.");
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Advanced").strong());
            ui.label("Set modification time writes the archive time onto each file. Relative paths drop the archive folder you are browsing. Full paths keep the archive path. Do not extract pathnames writes every file into the destination.");
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Options").strong());
            ui.label("Delete archive runs only after every member extracts with no warnings. Move to Recycle Bin sends it there instead of removing it permanently.");
            ui.label("Save settings remembers these choices, except the destination path, for the next time.");
            ui.label("Start in background leaves the file list usable and keeps progress on the status bar.");
        });
    dialog.help = open;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_uses_the_archive_name_without_its_extension() {
        let archive = Path::new(r"C:\Users\Administrator\Downloads\Programs\App.rar");
        let dest = suggested_destination(archive);
        assert_eq!(
            dest,
            PathBuf::from(r"C:\Users\Administrator\Downloads\Programs\App")
        );
    }

    #[test]
    fn validate_folder_name_accepts_simple_names() {
        assert_eq!(validate_folder_name("Projects").unwrap(), "Projects");
        assert_eq!(validate_folder_name("  Notes  ").unwrap(), "Notes");
    }

    #[test]
    fn validate_folder_name_rejects_invalid_names() {
        assert!(validate_folder_name("").is_err());
        assert!(validate_folder_name("bad/name").is_err());
        assert!(validate_folder_name("CON").is_err());
        assert!(validate_folder_name("name.").is_err());
    }
}
