//! Menu bar, icon toolbar, address bar, and status line.
//!
//! The commands are the ones ShellZip already performs. Compression is chosen
//! in the add dialog. Extract options are chosen in the extract dialog. The
//! toolbar is Open, Add, Extract To, Test, Delete, SFX, Info, and About.

use std::path::Path;

use egui::{Align2, Color32, FontId, Rect, Sense, Vec2};

use crate::explorer::{FolderTree, Jump, JumpTarget, TreeFocus, TreeRow};
use crate::icons::{self, Icon, ToolIcon};
use crate::settings::ToolbarSize;
use crate::shell_icons::{self, IconCache, IconKey};
use crate::theme::{self, Preference};
use shellzip_core::{SevenZMethod, ZipMethod};

/// Height of the address field and the well buttons beside it.
pub(crate) const WELL_SIZE: f32 = 26.0;

/// Result of [`modal`]. Backdrop clicks do not close the dialog.
pub struct ModalResult<T> {
    /// Value returned from the content closure.
    pub inner: T,
    /// Escape on the topmost modal while no popup is open.
    pub escape: bool,
}

/// Centered modal with a dimmed backdrop that blocks the shell.
///
/// Escape closes when this is the topmost modal and no popup is open. A click
/// on the backdrop is ignored so the user must use a button or Escape.
pub fn modal<R>(
    ctx: &egui::Context,
    id: impl Into<egui::Id>,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> ModalResult<R> {
    let response = egui::Modal::new(id.into()).show(ctx, |ui| {
        ui.label(egui::RichText::new(title).strong());
        ui.separator();
        add_contents(ui)
    });
    let escape = response.is_top_modal
        && !response.any_popup_open
        && response
            .response
            .ctx
            .input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
    ModalResult {
        inner: response.inner,
        escape,
    }
}

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
    /// Start an add with the progress window hidden.
    pub background: bool,
}

/// Menu entries that run a command.
#[derive(Clone, Copy)]
pub enum MenuCommand {
    /// Pick an archive, or open the selected one.
    Open,
    /// Create an archive, or add files to the open ZIP or 7z.
    Add,
    /// Close the window.
    Exit,
    /// Extract the open or selected archive.
    Extract,
    /// Test the open or selected archive.
    Test,
    /// Show one selected file in the window.
    View,
    /// Select every file in the current folder.
    SelectAll,
    /// Clear the selection.
    SelectNone,
    /// Show the about window.
    About,
    /// Delete selected members of the open ZIP or 7z.
    Delete,
    /// Rename one selected file or folder.
    Rename,
    /// Show archive header details.
    Info,
    /// Write an HTML, Markdown, or text listing.
    Report,
    /// Light, dark, or follow the operating system.
    Theme(Preference),
    /// Open a path from the File menu recent list.
    OpenRecent(usize),
    /// Forget remembered archives and destinations.
    ClearHistory,
    /// Toggle every row except `..`.
    InvertSelection,
    /// Large (32) or extra-large (48) toolbar glyphs.
    ToolbarSize(ToolbarSize),
}

/// Toolbar buttons, in the same order as the reference layout.
#[derive(Clone, Copy)]
pub enum ToolCommand {
    /// Pick an archive, or open the selected one.
    Open,
    /// Create an archive, or add files to the open ZIP or 7z.
    Add,
    /// Extract into a chosen folder.
    Extract,
    /// Test archive integrity.
    Test,
    /// Delete selected ZIP members.
    Delete,
    /// Show archive or selection details.
    Info,
    /// Show the about window.
    About,
    /// Wrap the open ZIP or 7z as a self-extractor.
    Sfx,
    /// Show one selected file.
    View,
}

/// Which toolbar buttons are live this frame.
pub struct ToolbarState {
    /// An extract, add, test, or other job is running.
    pub operation: bool,
    /// The file list is locked by a foreground job or listing.
    pub locked: bool,
    /// An archive is open, or a supported archive is selected.
    pub archive_ready: bool,
    /// Selected ZIP members can be deleted.
    pub can_delete: bool,
    /// Exactly one file is selected for viewing.
    pub can_view: bool,
    /// The file pane is inside a ZIP or 7z.
    pub into_archive: bool,
}

/// Address bar result.
pub enum AddressEvent {
    /// Up one folder.
    Up,
    /// Enter in the path field.
    Submit,
    /// Reload the folder or archive that is open.
    Refresh,
    /// A parent chosen from the dropdown.
    Jump(JumpTarget),
}

/// File, Commands, Options, and Help.
///
/// `into_archive` is set when a ZIP or 7z is open, so Add names that archive.
/// `opened` is recent archives, newest first.
pub fn menu_bar(
    ui: &mut egui::Ui,
    theme: Preference,
    toolbar: ToolbarSize,
    into_archive: bool,
    opened: &[String],
) -> Option<MenuCommand> {
    let mut command = None;
    let add_label = if into_archive {
        "Add to this archive..."
    } else {
        "Add to archive..."
    };
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open...").clicked() {
                command = Some(MenuCommand::Open);
            }
            if ui.button(add_label).clicked() {
                command = Some(MenuCommand::Add);
            }
            if !opened.is_empty() {
                ui.separator();
                for (index, path) in opened.iter().enumerate() {
                    let name = Path::new(path)
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .filter(|name| !name.is_empty())
                        .unwrap_or_else(|| path.clone());
                    if ui.button(name).on_hover_text(path).clicked() {
                        command = Some(MenuCommand::OpenRecent(index));
                    }
                }
            }
            ui.separator();
            ui.add_enabled_ui(!opened.is_empty(), |ui| {
                if ui
                    .button("Clear history")
                    .on_hover_text("Forget recently opened archives and extract destinations")
                    .clicked()
                {
                    command = Some(MenuCommand::ClearHistory);
                }
            });
            ui.separator();
            if ui.button("Exit").clicked() {
                command = Some(MenuCommand::Exit);
            }
        });
        ui.menu_button("Commands", |ui| {
            if ui.button("Extract to...").clicked() {
                command = Some(MenuCommand::Extract);
            }
            if ui.button("Test archived files").clicked() {
                command = Some(MenuCommand::Test);
            }
            if ui.button("View file").clicked() {
                command = Some(MenuCommand::View);
            }
            if ui.button("Delete files").clicked() {
                command = Some(MenuCommand::Delete);
            }
            if ui.button("Rename file").clicked() {
                command = Some(MenuCommand::Rename);
            }
            if ui.button("Show information...").clicked() {
                command = Some(MenuCommand::Info);
            }
            if ui.button("Generate report...").clicked() {
                command = Some(MenuCommand::Report);
            }
            ui.separator();
            if ui.button("Select all").clicked() {
                command = Some(MenuCommand::SelectAll);
            }
            if ui.button("Select none").clicked() {
                command = Some(MenuCommand::SelectNone);
            }
            if ui.button("Invert selection").clicked() {
                command = Some(MenuCommand::InvertSelection);
            }
        });
        ui.menu_button("Options", |ui| {
            for (choice, label) in [
                (Preference::System, "System"),
                (Preference::Light, "Light"),
                (Preference::Dark, "Dark"),
            ] {
                if ui.selectable_label(theme == choice, label).clicked() {
                    choice.install(ui.ctx());
                    command = Some(MenuCommand::Theme(choice));
                }
            }
            ui.separator();
            ui.menu_button("Toolbar icons", |ui| {
                for (choice, label) in [
                    (ToolbarSize::Normal, "Normal"),
                    (ToolbarSize::Large, "Large"),
                    (ToolbarSize::ExtraLarge, "Extra large"),
                ] {
                    if ui.selectable_label(toolbar == choice, label).clicked() {
                        command = Some(MenuCommand::ToolbarSize(choice));
                    }
                }
            });
        });
        ui.menu_button("Help", |ui| {
            if ui.button("About ShellZip").clicked() {
                command = Some(MenuCommand::About);
            }
        });
    });
    command
}

/// Large icons with captions, using the brand toolbar glyphs.
///
/// `into_archive` changes the Add tip when a ZIP or 7z is open.
pub fn toolbar(ui: &mut egui::Ui, state: &ToolbarState, size: ToolbarSize) -> Option<ToolCommand> {
    let mut command = None;
    let add_tip = if state.into_archive {
        "Add files to this archive"
    } else {
        "Create an archive"
    };
    let idle = !state.operation && !state.locked;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if tool(ui, ToolIcon::Open, "Open", "Open an archive", idle, size) {
            command = Some(ToolCommand::Open);
        }
        if tool(ui, ToolIcon::Add, "Add", add_tip, idle, size) {
            command = Some(ToolCommand::Add);
        }
        if tool(
            ui,
            ToolIcon::Extract,
            "Extract To",
            "Extract into a folder",
            idle && state.archive_ready,
            size,
        ) {
            command = Some(ToolCommand::Extract);
        }
        if tool(
            ui,
            ToolIcon::Test,
            "Test",
            "Test the archive",
            idle && state.archive_ready,
            size,
        ) {
            command = Some(ToolCommand::Test);
        }
        if tool(
            ui,
            ToolIcon::View,
            "View",
            "View the selected file",
            state.can_view,
            size,
        ) {
            command = Some(ToolCommand::View);
        }
        if tool(
            ui,
            ToolIcon::Delete,
            "Delete",
            "Delete selected ZIP members",
            state.can_delete,
            size,
        ) {
            command = Some(ToolCommand::Delete);
        }
        if tool(
            ui,
            ToolIcon::Sfx,
            "SFX",
            "Create a self-extracting archive",
            idle && state.into_archive,
            size,
        ) {
            command = Some(ToolCommand::Sfx);
        }
        if tool(
            ui,
            ToolIcon::Info,
            "Info",
            "Details of this folder or archive",
            !state.locked,
            size,
        ) {
            command = Some(ToolCommand::Info);
        }
        if tool(
            ui,
            ToolIcon::About,
            "About",
            "About ShellZip",
            !state.locked,
            size,
        ) {
            command = Some(ToolCommand::About);
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
    let muted = theme::palette(ui.ctx()).muted;
    ui.horizontal_wrapped(|ui| {
        for (index, jump) in jumps.iter().enumerate() {
            if index > 0 {
                ui.label(egui::RichText::new(">").color(muted));
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

/// Up button, path field, folder-tree dropdown, and refresh.
pub fn address_bar(
    ui: &mut egui::Ui,
    text: &mut String,
    can_up: bool,
    can_refresh: bool,
    tree: &mut FolderTree,
    focus: &TreeFocus,
    cache: &mut IconCache,
) -> Option<AddressEvent> {
    let mut event = None;
    let bar = global_rect(ui);
    ui.horizontal(|ui| {
        if tool_icon_button(ui, ToolIcon::Up, "Up one level", can_up).clicked() {
            event = Some(AddressEvent::Up);
        }
        let spacing = ui.spacing().item_spacing.x;
        let reserved = WELL_SIZE * 2.0 + spacing * 2.0;
        let width = (ui.available_width() - reserved).max(48.0);
        let response = path_field(ui, text, width, "", "shellzip-address");
        if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            event = Some(AddressEvent::Submit);
        }
        if let Some(jump) = parent_menu(ui, tree, focus, cache, bar) {
            event = Some(jump);
        }
        if well_tool_button(ui, ToolIcon::Refresh, "Refresh", can_refresh).clicked() {
            event = Some(AddressEvent::Refresh);
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
    /// Pause or continue the running extract or add.
    pub toggle_pause: bool,
    /// Show the progress window again.
    pub details: bool,
}

/// Compact progress drawn while a job runs, including one sent to the background.
pub struct RunningStatus {
    /// Current file or operation label.
    pub text: String,
    /// Total-bar fraction, `0.0` when the total is unknown.
    pub fraction: f32,
    /// Animate the bar because the total size is unknown.
    pub indeterminate: bool,
    /// The worker is paused.
    pub paused: bool,
    /// Show Pause or Continue. Extract and add only.
    pub pause: bool,
    /// Show Details. A background extract or add only.
    pub details: bool,
}

/// Totals on the right. Progress, errors, and job notes stay on the left.
pub fn status_bar(
    ui: &mut egui::Ui,
    summary: &str,
    running: Option<&RunningStatus>,
    error: Option<&str>,
    note: &str,
) -> StatusEvent {
    let mut dismiss = false;
    let mut cancel = false;
    let mut toggle_pause = false;
    let mut details = false;
    let colors = theme::palette(ui.ctx());
    ui.horizontal(|ui| {
        if let Some(job) = running {
            icons::show_spinning(ui, colors.ink, icons::SIZE_STATUS);
            ui.add(
                egui::ProgressBar::new(job.fraction)
                    .fill(theme::RUST)
                    .animate(job.indeterminate)
                    .desired_width(160.0),
            );
            ui.label(egui::RichText::new(&job.text).color(colors.muted));
            if job.pause {
                let label = if job.paused { "Continue" } else { "Pause" };
                if ui.button(label).clicked() {
                    toggle_pause = true;
                }
            }
            if job.details && ui.button("Details").clicked() {
                details = true;
            }
            if ui.button("Cancel").clicked() {
                cancel = true;
            }
        } else if let Some(message) = error {
            icons::show(ui, Icon::TriangleAlert, colors.danger, icons::SIZE_STATUS);
            ui.colored_label(colors.danger, message);
            if dismiss_button(ui).clicked() {
                dismiss = true;
            }
        } else if !note.is_empty() {
            if note.starts_with("OK") {
                icons::show(ui, Icon::CircleCheck, colors.ok, icons::SIZE_STATUS);
            } else {
                icons::show(ui, Icon::Archive, colors.muted, icons::SIZE_STATUS);
            }
            ui.label(egui::RichText::new(note).color(colors.muted));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(summary);
        });
    });
    if running.is_some()
        && let Some(message) = error
    {
        ui.horizontal(|ui| {
            icons::show(ui, Icon::TriangleAlert, colors.danger, icons::SIZE_STATUS);
            ui.colored_label(colors.danger, message);
            if dismiss_button(ui).clicked() {
                dismiss = true;
            }
        });
    }
    StatusEvent {
        dismiss_error: dismiss,
        cancel,
        toggle_pause,
        details,
    }
}

/// How the delete confirmation ended.
pub enum DeleteChoice {
    /// Delete the listed names.
    Delete,
    /// Cancel or the window close button.
    Cancel,
    /// The window is still open.
    Wait,
}

/// Confirm deletion of archive members. `names` are scrolled above the buttons.
pub fn confirm_delete(ctx: &egui::Context, names: &[String]) -> DeleteChoice {
    let mut choice = DeleteChoice::Wait;
    let count = names.len();
    let result = modal(ctx, "shellzip-delete", "Delete", |ui| {
        ui.set_min_width(440.0);
        ui.set_min_height(320.0);
        ui.label(format!(
            "Delete {count} selected members from this archive?"
        ));
        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .id_salt("shellzip-delete-names")
            .max_height(180.0)
            .show(ui, |ui| {
                for name in names {
                    ui.label(name);
                }
            });
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let colors = theme::palette(ui.ctx());
                if labeled_button(ui, Icon::X, colors.ink, "Cancel").clicked() {
                    choice = DeleteChoice::Cancel;
                }
                if labeled_button(ui, Icon::Trash, colors.danger, "Delete").clicked() {
                    choice = DeleteChoice::Delete;
                }
            });
        });
    });
    if result.escape && matches!(choice, DeleteChoice::Wait) {
        DeleteChoice::Cancel
    } else {
        choice
    }
}

/// Integer with thousands separators, such as `9,655,595,819`.
pub(crate) fn format_grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// Chrome strip in the active palette.
pub fn bar_frame(ctx: &egui::Context, x: i8, y: i8) -> egui::Frame {
    egui::Frame::new()
        .fill(theme::palette(ctx).panel)
        .inner_margin(egui::Margin::symmetric(x, y))
}

fn tool(
    ui: &mut egui::Ui,
    icon: ToolIcon,
    label: &str,
    tip: &str,
    enabled: bool,
    size: ToolbarSize,
) -> bool {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let colors = theme::palette(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(size.button_size(), sense);
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, 3.0, colors.hover);
    }
    let glyph = size.icon_points();
    let icon_rect = Rect::from_center_size(
        rect.center_top() + egui::vec2(0.0, 4.0 + glyph * 0.5),
        Vec2::splat(glyph),
    );
    icons::paint_tool(ui, icon, enabled, icon_rect);
    let text = if enabled { colors.ink } else { colors.muted };
    ui.painter().text(
        rect.center_bottom() + egui::vec2(0.0, -3.0),
        Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(size.label_size()),
        text,
    );
    enabled && response.on_hover_text(tip).clicked()
}

fn dismiss_button(ui: &mut egui::Ui) -> egui::Response {
    let colors = theme::palette(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(icons::SIZE_ROW), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, colors.hover);
    }
    icons::paint(ui, Icon::X, colors.danger, rect);
    response.on_hover_text("Dismiss")
}

/// Single-line path field. Height matches [`WELL_SIZE`].
pub(crate) fn path_field(
    ui: &mut egui::Ui,
    text: &mut String,
    width: f32,
    hint: &str,
    salt: &'static str,
) -> egui::Response {
    ui.add_sized(
        [width, WELL_SIZE],
        egui::TextEdit::singleline(text)
            .desired_width(width)
            .hint_text(hint)
            .margin(egui::Margin::symmetric(6, 4))
            .id_salt(salt),
    )
}

/// Dialog footer button: a Lucide glyph and a label.
pub fn labeled_button(ui: &mut egui::Ui, icon: Icon, tint: Color32, label: &str) -> egui::Response {
    match icons::image(ui.ctx(), icon, tint, icons::SIZE_STATUS) {
        Some(image) => ui.add(egui::Button::image_and_text(image, label)),
        None => ui.button(label),
    }
}

pub(crate) fn icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    tint: Color32,
    tip: &str,
    enabled: bool,
) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let colors = theme::palette(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), sense);
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, 2.0, colors.hover);
    }
    let color = if enabled { tint } else { colors.muted };
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icons::SIZE_ROW));
    icons::paint(ui, icon, color, icon_rect);
    response.on_hover_text(tip)
}

fn tool_icon_button(ui: &mut egui::Ui, icon: ToolIcon, tip: &str, enabled: bool) -> egui::Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let colors = theme::palette(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), sense);
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, 2.0, colors.hover);
    }
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icons::SIZE_TOOL_WELL));
    icons::paint_tool(ui, icon, enabled, icon_rect);
    response.on_hover_text(tip)
}

/// Painted caret. The default font has no ▾ glyph, which was drawing as an empty box
/// on the address bar above the file list.
///
/// `bar` is the address row in global coordinates. The menu hangs under that
/// row at the address-bar width. Its height is the 7-Zip path-combo drop
/// height, clamped so the list stays on screen above the status bar.
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
    let height = dropdown_height(ui.ctx().screen_rect().bottom() - bar.bottom());
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
                ui.label(egui::RichText::new(message).color(theme::palette(ui.ctx()).danger));
            }
            // `allocate_ui` fixes the popup height. A scroll area alone shrinks
            // to the three open rows, which is not the path-combo drop height.
            let width = ui.available_width().min(bar.width());
            ui.allocate_ui_with_layout(
                egui::vec2(width, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(height)
                        .show(ui, |ui| {
                            for row in tree.rows(focus) {
                                if let Some(target) = tree_row(ui, tree, cache, &row) {
                                    event = Some(AddressEvent::Jump(target));
                                    ui.close();
                                }
                            }
                        });
                },
            );
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

pub(crate) fn tree_row(
    ui: &mut egui::Ui,
    tree: &mut FolderTree,
    cache: &mut IconCache,
    row: &TreeRow,
) -> Option<JumpTarget> {
    let width = ui.available_width();
    let mut navigate = None;
    let mut toggle = false;
    let colors = theme::palette(ui.ctx());
    ui.allocate_ui_with_layout(
        egui::vec2(width, 22.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let rect = ui.max_rect();
            let text = if row.current {
                colors.select_text
            } else {
                colors.ink
            };
            if row.current {
                ui.painter().rect_filled(rect, 0.0, colors.select);
            } else if ui.rect_contains_pointer(rect) {
                ui.painter().rect_filled(rect, 0.0, colors.hover);
            }
            ui.add_space(4.0 + f32::from(row.depth) * 14.0);
            if row.expandable {
                let (mark_rect, mark_response) =
                    ui.allocate_exact_size(egui::vec2(16.0, 18.0), Sense::click());
                icons::paint_expander(ui, row.expanded, text, mark_rect);
                if mark_response.clicked() {
                    toggle = true;
                }
            } else {
                ui.add_space(16.0);
            }
            // Shell pictures keep their own colors on the selection bar.
            // A miss falls back to the yellow folder, or the file glyph for an extension.
            let fallback = match &row.icon {
                IconKey::Extension(_) => Icon::File,
                _ => Icon::Folder,
            };
            let tint = if matches!(fallback, Icon::Folder) {
                colors.folder
            } else if row.current {
                colors.select_text
            } else {
                colors.muted
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

/// Dropped-list height for the address menu.
///
/// 7-Zip creates the path combo 620px tall. Win32 treats that as the edit
/// field plus the list, so the list itself stays at 620 until the space under
/// the address bar (above the status line) is shorter.
fn dropdown_height(space_below: f32) -> f32 {
    const PREFERRED: f32 = 620.0;
    const STATUS: f32 = 32.0;
    let room = (space_below - STATUS).max(22.0);
    PREFERRED.min(room)
}

fn caret_button(ui: &mut egui::Ui, tip: &str) -> egui::Response {
    well_button(ui, Icon::ChevronDown, tip, true)
}

/// Bordered chrome button. The address field, refresh, and tree drop-down share it.
pub(crate) fn well_button(
    ui: &mut egui::Ui,
    icon: Icon,
    tip: &str,
    enabled: bool,
) -> egui::Response {
    let colors = theme::palette(ui.ctx());
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(WELL_SIZE), sense);
    let fill = if enabled && response.hovered() {
        colors.hover
    } else {
        colors.field
    };
    ui.painter().rect_filled(rect, 2.0, fill);
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0_f32, colors.line),
        egui::StrokeKind::Inside,
    );
    let tint = if enabled { colors.ink } else { colors.muted };
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(14.0));
    icons::paint(ui, icon, tint, icon_rect);
    response.on_hover_text(tip)
}

fn well_tool_button(ui: &mut egui::Ui, icon: ToolIcon, tip: &str, enabled: bool) -> egui::Response {
    let colors = theme::palette(ui.ctx());
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(WELL_SIZE), sense);
    let fill = if enabled && response.hovered() {
        colors.hover
    } else {
        colors.field
    };
    ui.painter().rect_filled(rect, 2.0, fill);
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0_f32, colors.line),
        egui::StrokeKind::Inside,
    );
    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icons::SIZE_TOOL_WELL));
    icons::paint_tool(ui, icon, enabled, icon_rect);
    response.on_hover_text(tip)
}

#[cfg(test)]
mod tests {
    use super::{dropdown_height, format_grouped};

    #[test]
    fn dropdown_keeps_the_sevenzip_height_until_the_window_runs_out() {
        assert_eq!(dropdown_height(2000.0), 620.0);
        assert_eq!(dropdown_height(620.0 + 32.0), 620.0);
        assert_eq!(dropdown_height(400.0), 368.0);
        assert_eq!(dropdown_height(40.0), 22.0);
    }

    #[test]
    fn format_grouped_inserts_thousands_separators() {
        assert_eq!(format_grouped(0), "0");
        assert_eq!(format_grouped(999), "999");
        assert_eq!(format_grouped(1000), "1,000");
        assert_eq!(format_grouped(9_655_595_819), "9,655,595,819");
    }
}
