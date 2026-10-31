//! Menu bar, icon toolbar, address bar, and status line.
//!
//! The commands are the ones ShellZip already performs. Compression is chosen
//! in the add dialog. The menu holds extract options. The toolbar is Add,
//! Extract To, Test, Delete, and Info.

use egui::{Align2, Color32, FontId, Rect, Sense, Vec2};

use crate::explorer::{FolderTree, Jump, JumpTarget, TreeFocus, TreeRow};
use crate::icons::{self, Caret, Icon};
use crate::shell_icons::{self, IconCache, IconKey};
use crate::theme::{
    ADD, DANGER, EXTRACT, FOLDER, HOVER, INK, MUTED, OK, OPEN, PANEL, SELECT, SELECT_TEXT, TEST,
};
use shellzip_core::{Overwrite, SevenZMethod, ZipMethod};

/// Pack choices remembered for the add dialog, and extract choices from the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackSettings {
    /// `-m` level, 0 (store) through 5 (maximum).
    pub level: u8,
    /// ZIP method.
    pub method: ZipMethod,
    /// 7z method. LZMA2 is the default.
    pub sevenz_method: SevenZMethod,
    /// Solid 7z blocks. Default is on.
    pub solid: bool,
    /// What to do when a destination file exists.
    pub overwrite: Overwrite,
    /// Drop archive directories on extract.
    pub flat: bool,
}

/// Menu entries that run a command.
#[derive(Clone, Copy)]
pub enum MenuCommand {
    /// Pick an archive, or open the selected one.
    Open,
    /// Create an archive.
    Add,
    /// Close the window.
    Exit,
    /// Extract the open or selected archive.
    Extract,
    /// Test the open or selected archive.
    Test,
    /// Select every file in the current folder.
    SelectAll,
    /// Clear the selection.
    SelectNone,
    /// Show the about window.
    About,
    /// Delete selected members of the open ZIP.
    Delete,
    /// Show details for the open archive or the selection.
    Info,
}

/// Toolbar buttons, in the same order as the reference layout.
#[derive(Clone, Copy)]
pub enum ToolCommand {
    /// Create an archive.
    Add,
    /// Extract into a chosen folder.
    Extract,
    /// Test archive integrity.
    Test,
    /// Delete selected ZIP members.
    Delete,
    /// Show archive or selection details.
    Info,
}

/// Address bar result.
pub enum AddressEvent {
    /// Up one folder.
    Up,
    /// Enter in the path field.
    Submit,
    /// A parent chosen from the dropdown.
    Jump(JumpTarget),
}

/// File, Commands, Options, and Help.
pub fn menu_bar(ui: &mut egui::Ui, options: &mut PackSettings) -> Option<MenuCommand> {
    let mut command = None;
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                command = Some(MenuCommand::Open);
            }
            if ui.button("Add to archive...").clicked() {
                command = Some(MenuCommand::Add);
            }
            ui.separator();
            if ui.button("Exit").clicked() {
                command = Some(MenuCommand::Exit);
            }
        });
        ui.menu_button("Commands", |ui| {
            if ui.button("Extract to...").clicked() {
                command = Some(MenuCommand::Extract);
            }
            if ui.button("Test").clicked() {
                command = Some(MenuCommand::Test);
            }
            if ui.button("Delete").clicked() {
                command = Some(MenuCommand::Delete);
            }
            if ui.button("Info").clicked() {
                command = Some(MenuCommand::Info);
            }
            ui.separator();
            if ui.button("Select all").clicked() {
                command = Some(MenuCommand::SelectAll);
            }
            if ui.button("Select none").clicked() {
                command = Some(MenuCommand::SelectNone);
            }
        });
        options_menu(ui, options);
        ui.menu_button("Help", |ui| {
            if ui.button("About ShellZip").clicked() {
                command = Some(MenuCommand::About);
            }
        });
    });
    command
}

/// Large icons with captions, using the Lucide set already shipped with the window.
pub fn toolbar(
    ui: &mut egui::Ui,
    busy: bool,
    archive_ready: bool,
    can_delete: bool,
) -> Option<ToolCommand> {
    let mut command = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if tool(ui, Icon::Plus, ADD, "Add", "Create an archive", !busy) {
            command = Some(ToolCommand::Add);
        }
        if tool(
            ui,
            Icon::FolderDown,
            EXTRACT,
            "Extract To",
            "Extract into a folder",
            !busy && archive_ready,
        ) {
            command = Some(ToolCommand::Extract);
        }
        if tool(
            ui,
            Icon::CircleCheck,
            TEST,
            "Test",
            "Test the archive",
            !busy && archive_ready,
        ) {
            command = Some(ToolCommand::Test);
        }
        if tool(
            ui,
            Icon::Trash,
            DANGER,
            "Delete",
            "Delete selected ZIP members",
            can_delete,
        ) {
            command = Some(ToolCommand::Delete);
        }
        if tool(
            ui,
            Icon::Info,
            OPEN,
            "Info",
            "Details of this folder or archive",
            !busy,
        ) {
            command = Some(ToolCommand::Info);
        }
    });
    command
}

/// Clickable parent folders. The current location is the last crumb.
pub fn breadcrumbs(ui: &mut egui::Ui, jumps: &[Jump]) -> Option<AddressEvent> {
    if !jumps
        .iter()
        .any(|jump| matches!(jump.target, JumpTarget::Prefix(_)))
    {
        return None;
    }
    let mut event = None;
    let last = jumps.len().saturating_sub(1);
    ui.horizontal_wrapped(|ui| {
        for (index, jump) in jumps.iter().enumerate() {
            if index > 0 {
                ui.label(egui::RichText::new(">").color(MUTED));
            }
            let label = crumb_label(jump);
            if index == last {
                ui.strong(label);
            } else if ui.button(label).clicked() {
                event = Some(AddressEvent::Jump(jump.target.clone()));
            }
        }
    });
    event
}

fn crumb_label(jump: &Jump) -> String {
    match &jump.target {
        JumpTarget::Prefix(prefix) if !prefix.is_empty() => prefix
            .rsplit('/')
            .next()
            .unwrap_or(prefix.as_str())
            .to_string(),
        _ => std::path::Path::new(&jump.label)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| jump.label.clone()),
    }
}

/// Up button, editable path, and a folder-tree dropdown.
pub fn address_bar(
    ui: &mut egui::Ui,
    text: &mut String,
    can_up: bool,
    tree: &mut FolderTree,
    focus: &TreeFocus,
    cache: &mut IconCache,
) -> Option<AddressEvent> {
    let mut event = None;
    let bar = global_rect(ui);
    ui.horizontal(|ui| {
        ui.add_enabled_ui(can_up, |ui| {
            if icon_button(ui, Icon::ArrowUp, "Up one level").clicked() {
                event = Some(AddressEvent::Up);
            }
        });
        let width = (ui.available_width() - 28.0).max(48.0);
        let response = ui.add(
            egui::TextEdit::singleline(text)
                .desired_width(width)
                .id_salt("shellzip-address"),
        );
        if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            event = Some(AddressEvent::Submit);
        }
        if let Some(jump) = parent_menu(ui, tree, focus, cache, bar) {
            event = Some(jump);
        }
    });
    event
}

/// What the status line wants the window to do.
pub struct StatusEvent {
    /// The error dismiss button was pressed.
    pub dismiss_error: bool,
    /// The cancel button was pressed.
    pub cancel: bool,
}

/// Totals on the right. Progress, errors, and job notes stay on the left.
pub fn status_bar(
    ui: &mut egui::Ui,
    summary: &str,
    busy: bool,
    progress: (String, String, u64, u64),
    error: Option<&str>,
    note: &str,
) -> StatusEvent {
    let mut dismiss = false;
    let mut cancel = false;
    ui.horizontal(|ui| {
        if busy {
            icons::show_spinning(ui, INK, icons::SIZE_STATUS);
            let (label, entry, done, total) = progress;
            let fraction = if total == 0 {
                0.0
            } else {
                (done as f32 / total as f32).clamp(0.0, 1.0)
            };
            ui.add(egui::ProgressBar::new(fraction).desired_width(160.0));
            let text = if entry.is_empty() { label } else { entry };
            ui.label(egui::RichText::new(text).color(MUTED));
            if ui.button("Cancel").clicked() {
                cancel = true;
            }
        } else if let Some(message) = error {
            icons::show(ui, Icon::TriangleAlert, DANGER, icons::SIZE_STATUS);
            ui.colored_label(DANGER, message);
            if dismiss_button(ui).clicked() {
                dismiss = true;
            }
        } else if !note.is_empty() {
            if note.starts_with("OK") {
                icons::show(ui, Icon::CircleCheck, OK, icons::SIZE_STATUS);
            } else {
                icons::show(ui, Icon::Archive, MUTED, icons::SIZE_STATUS);
            }
            ui.label(egui::RichText::new(note).color(MUTED));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(summary);
        });
    });
    StatusEvent {
        dismiss_error: dismiss,
        cancel,
    }
}

/// Details window. `text` is already formatted for display.
pub fn info_window(ctx: &egui::Context, text: &mut Option<String>) {
    let mut open = text.is_some();
    if let Some(body) = text.clone() {
        egui::Window::new("Info")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                ui.label(body);
            });
    }
    if !open {
        *text = None;
    }
}

/// Confirm deletion of ZIP members. Returns true when the user confirms.
pub fn confirm_delete(ctx: &egui::Context, pending: &mut Option<usize>) -> bool {
    let Some(count) = *pending else {
        return false;
    };
    let mut open = true;
    let mut confirmed = false;
    let mut dismiss = false;
    egui::Window::new("Delete")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(format!("Delete {count} selected members from this ZIP?"));
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    confirmed = true;
                }
                if ui.button("Cancel").clicked() {
                    dismiss = true;
                }
            });
        });
    if confirmed || dismiss {
        open = false;
    }
    if confirmed || !open {
        *pending = None;
    }
    confirmed
}

/// Short about window. The Lucide credit lives here so the status line stays a total.
pub fn about(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("About ShellZip")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("Open, list, test, extract, and create ZIP and tar archives.");
            ui.add_space(8.0);
            ui.label("Author: Ajay Menon");
            ui.add_space(8.0);
            ui.hyperlink_to("Icons by Lucide", "https://lucide.dev/icons/");
        });
}

/// Gray chrome strip.
pub fn bar_frame(x: i8, y: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(PANEL)
        .inner_margin(egui::Margin::symmetric(x, y))
}

fn options_menu(ui: &mut egui::Ui, options: &mut PackSettings) {
    egui::containers::menu::MenuButton::from_button(egui::Button::new("Options"))
        .config(
            egui::containers::menu::MenuConfig::default()
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
        )
        .ui(ui, |ui| {
            ui.set_min_width(220.0);
            ui.label("When a file exists");
            ui.selectable_value(&mut options.overwrite, Overwrite::Never, "Skip existing");
            ui.selectable_value(
                &mut options.overwrite,
                Overwrite::Always,
                "Replace existing",
            );
            ui.selectable_value(&mut options.overwrite, Overwrite::Rename, "Rename");
            ui.separator();
            ui.checkbox(&mut options.flat, "Flat extract");
        });
}

fn tool(
    ui: &mut egui::Ui,
    icon: Icon,
    tint: Color32,
    label: &str,
    tip: &str,
    enabled: bool,
) -> bool {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(74.0, 58.0), sense);
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, 3.0, HOVER);
    }
    let tint = if enabled {
        tint
    } else {
        tint.gamma_multiply(0.35)
    };
    let icon_rect = Rect::from_center_size(
        rect.center_top() + egui::vec2(0.0, 20.0),
        Vec2::splat(icons::SIZE_TOOL),
    );
    icons::paint(ui, icon, tint, icon_rect);
    let text = if enabled { INK } else { MUTED };
    ui.painter().text(
        rect.center_bottom() + egui::vec2(0.0, -5.0),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(11.0),
        text,
    );
    enabled && response.on_hover_text(tip).clicked()
}

fn dismiss_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(icons::SIZE_ROW), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, HOVER);
    }
    icons::paint(ui, Icon::X, DANGER, rect);
    response.on_hover_text("Dismiss")
}

fn icon_button(ui: &mut egui::Ui, icon: Icon, tip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(28.0, 24.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, HOVER);
    }
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icons::SIZE_ROW));
    icons::paint(ui, icon, INK, icon_rect);
    response.on_hover_text(tip)
}

/// Painted caret. The default font has no ▾ glyph, which was drawing as an empty box
/// on the address bar above the file list.
///
/// `bar` is the address row in global coordinates. The menu hangs under that
/// row, uses its full width, and shrinks to the expanded folder list.
fn parent_menu(
    ui: &mut egui::Ui,
    tree: &mut FolderTree,
    focus: &TreeFocus,
    cache: &mut IconCache,
    bar: Rect,
) -> Option<AddressEvent> {
    let response = caret_button(ui, "Folders");
    if response.clicked() {
        tree.reveal(focus);
    }
    let room = (ui.ctx().screen_rect().bottom() - bar.bottom()).max(120.0);
    let mut event = None;
    egui::Popup::menu(&response)
        .anchor(bar)
        .align(egui::RectAlign::BOTTOM_START)
        .align_alternatives(&[])
        .width(bar.width())
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let inset = ui.spacing().menu_margin.sum().x + ui.visuals().window_stroke.width * 2.0;
            ui.set_min_width((bar.width() - inset).max(48.0));
            ui.spacing_mut().item_spacing.y = 0.0;
            tree.reveal(focus);
            if let Some(message) = tree.loader_error() {
                ui.label(egui::RichText::new(message).color(DANGER));
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(room)
                .show(ui, |ui| {
                    for row in tree.rows(focus) {
                        if let Some(target) = tree_row(ui, tree, cache, &row) {
                            event = Some(AddressEvent::Jump(target));
                            ui.close();
                        }
                    }
                });
        });
    event
}

fn global_rect(ui: &egui::Ui) -> Rect {
    let rect = ui.max_rect();
    ui.ctx()
        .layer_transform_to_global(ui.layer_id())
        .map(|to_global| to_global * rect)
        .unwrap_or(rect)
}

fn tree_row(
    ui: &mut egui::Ui,
    tree: &mut FolderTree,
    cache: &mut IconCache,
    row: &TreeRow,
) -> Option<JumpTarget> {
    let width = ui.available_width();
    let mut navigate = None;
    let mut toggle = false;
    ui.allocate_ui_with_layout(
        egui::vec2(width, 22.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let rect = ui.max_rect();
            let text = if row.current { SELECT_TEXT } else { INK };
            if row.current {
                ui.painter().rect_filled(rect, 0.0, SELECT);
            } else if ui.rect_contains_pointer(rect) {
                ui.painter().rect_filled(rect, 0.0, HOVER);
            }
            ui.add_space(4.0 + f32::from(row.depth) * 14.0);
            if row.expandable {
                let (caret_rect, caret_response) =
                    ui.allocate_exact_size(egui::vec2(14.0, 18.0), Sense::click());
                let mark = egui::Rect::from_center_size(caret_rect.center(), egui::vec2(8.0, 6.0));
                icons::paint_caret(
                    ui,
                    if row.expanded {
                        Caret::Down
                    } else {
                        Caret::Right
                    },
                    text,
                    mark,
                );
                if caret_response.clicked() {
                    toggle = true;
                }
            } else {
                ui.add_space(14.0);
            }
            let fallback = match &row.icon {
                IconKey::Extension(_) => Icon::File,
                IconKey::Computer => Icon::Archive,
                _ => Icon::Folder,
            };
            let tint = if row.current {
                SELECT_TEXT
            } else if matches!(fallback, Icon::Folder) {
                FOLDER
            } else {
                MUTED
            };
            if !shell_icons::paint(ui, cache, &row.icon, icons::SIZE_ROW) {
                icons::show(ui, fallback, tint, icons::SIZE_ROW);
            }
            ui.add_space(4.0);
            let label = ui.add(
                egui::Label::new(egui::RichText::new(&row.label).color(text)).sense(Sense::click()),
            );
            if label.clicked() && !toggle {
                if let Some(target) = row.target.clone() {
                    navigate = Some(target);
                } else {
                    toggle = true;
                }
            }
        },
    );
    if toggle {
        tree.toggle(&row.id);
    }
    navigate
}

fn caret_button(ui: &mut egui::Ui, tip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, HOVER);
    }
    let caret = Rect::from_center_size(rect.center(), Vec2::new(9.0, 6.0));
    icons::paint_caret(ui, Caret::Down, INK, caret);
    response.on_hover_text(tip)
}
