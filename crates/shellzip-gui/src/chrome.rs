//! Menu bar, icon toolbar, address bar, and status line.
//!
//! The commands are the ones ShellZip already performs. The menu holds the
//! pack and extract options; the toolbar is only Add, Extract To, Test, and Open.

use egui::{Align2, Color32, FontId, Rect, Sense, Vec2};

use crate::explorer::{Jump, JumpTarget};
use crate::icons::{self, Caret, Icon};
use crate::theme::{ADD, DANGER, EXTRACT, HOVER, INK, MUTED, OK, OPEN, PANEL, TEST};
use shellzip_core::{Overwrite, ZipMethod};

/// Pack and extract choices edited from the Options menu.
#[derive(Clone, Copy)]
pub struct PackSettings {
    /// `-m` level, 0 (store) through 5 (maximum).
    pub level: u8,
    /// ZIP method.
    pub method: ZipMethod,
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
    /// Open an archive.
    Open,
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
pub fn toolbar(ui: &mut egui::Ui, busy: bool, archive_ready: bool) -> Option<ToolCommand> {
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
        if tool(ui, Icon::FolderOpen, OPEN, "Open", "Open an archive", !busy) {
            command = Some(ToolCommand::Open);
        }
    });
    command
}

/// Up button, editable path, and a dropdown of parent folders.
pub fn address_bar(
    ui: &mut egui::Ui,
    text: &mut String,
    can_up: bool,
    jumps: &[Jump],
) -> Option<AddressEvent> {
    let mut event = None;
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
        if let Some(jump) = parent_menu(ui, jumps) {
            event = Some(jump);
        }
    });
    event
}

/// Totals on the right. Progress, errors, and job notes stay on the left.
///
/// Returns true when the error's Dismiss button was pressed.
pub fn status_bar(
    ui: &mut egui::Ui,
    summary: &str,
    busy: bool,
    progress: (String, String, u64, u64),
    error: Option<&str>,
    note: &str,
) -> bool {
    let mut dismiss = false;
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
    dismiss
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
            ui.label("Compression level");
            let level = level_name(options.level);
            ui.add(egui::Slider::new(&mut options.level, 0..=5).text(level));
            ui.separator();
            ui.label("Method");
            ui.selectable_value(&mut options.method, ZipMethod::Deflate, "Deflate");
            ui.selectable_value(&mut options.method, ZipMethod::Zstd, "Zstd");
            ui.separator();
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

fn level_name(level: u8) -> &'static str {
    match level {
        0 => "store",
        1 => "fastest",
        2 => "fast",
        3 => "normal",
        4 => "good",
        _ => "maximum",
    }
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
fn parent_menu(ui: &mut egui::Ui, jumps: &[Jump]) -> Option<AddressEvent> {
    let response = caret_button(ui, "Parent folders");
    let mut event = None;
    egui::Popup::menu(&response).show(|ui| {
        if jumps.is_empty() {
            ui.label(egui::RichText::new("No parent folders").color(MUTED));
            return;
        }
        for jump in jumps {
            if ui.button(&jump.label).clicked() {
                event = Some(AddressEvent::Jump(jump.target.clone()));
            }
        }
    });
    event
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
