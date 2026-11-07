//! Lucide icons and the pre-colored toolbar glyphs.
//!
//! Lucide strokes are white so [`paint`] can tint them. Toolbar art comes from
//! `grx/toolbar_icons/png/` as fixed-size PNGs; [`paint_tool`] draws them as-is.
//!
//! Source: <https://lucide.dev/icons/>. The SVG files and their ISC license
//! live in `assets/lucide/`.
//!
//! Lucide icons are rasterized once at [`MASTER_POINTS`] and then scaled. A table
//! sizing pass otherwise asks for a new SVG size every frame and the loader
//! draws its failure mark above the file list.

use egui::{Color32, Rect, Sense, Vec2};

/// File-list row glyph.
pub const SIZE_ROW: f32 = 16.0;
/// Status-line glyph.
pub const SIZE_STATUS: f32 = 14.0;
/// Sort and address-bar caret.
pub const SIZE_CARET: Vec2 = Vec2::new(9.0, 8.0);
/// Address-bar Up / Refresh glyph.
pub const SIZE_TOOL_WELL: f32 = 16.0;

/// One raster size for every Lucide icon. Painting scales this texture.
const MASTER_POINTS: f32 = 32.0;

/// Available toolbar PNG edge lengths, ascending.
const TOOL_PNG_SIZES: [u32; 5] = [16, 24, 32, 48, 64];

/// Icons used by the window.
#[derive(Clone, Copy)]
pub enum Icon {
    /// Open an archive.
    FolderOpen,
    /// Extract into a folder.
    FolderDown,
    /// Test the archive.
    CircleCheck,
    /// Create an archive.
    Plus,
    /// Regular file.
    File,
    /// Directory.
    Folder,
    /// Empty-state archive.
    FileArchive,
    /// Error.
    TriangleAlert,
    /// Archive path.
    Archive,
    /// Symbolic link.
    Link,
    /// Work in progress.
    Loader,
    /// Dismiss.
    X,
    /// Leave the current folder.
    ArrowUp,
    /// Delete the selected archive members.
    Trash,
    /// Details of the selection or archive.
    Info,
    /// Confirm.
    Check,
    /// Remember the current choices.
    Save,
    /// Create a directory.
    FolderPlus,
    /// Find a folder in the tree.
    FolderSearch,
    /// Explain the dialog.
    CircleHelp,
    /// Reload the current folder or archive.
    Refresh,
    /// Open a drop-down list.
    ChevronDown,
}

/// Up or down triangle, painted so we do not depend on font glyphs.
#[derive(Clone, Copy)]
pub enum Caret {
    /// Sort ascending, or the address-bar parent list.
    Up,
    /// Sort descending.
    Down,
}

/// Pre-colored toolbar and address-bar glyphs from `grx/toolbar_icons/png/`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToolIcon {
    /// Open an archive.
    Open,
    /// Create an archive, or add files to one.
    Add,
    /// Extract into a folder.
    Extract,
    /// Test archive integrity.
    Test,
    /// Delete selected members.
    Delete,
    /// Archive or selection details.
    Info,
    /// About ShellZip.
    About,
    /// Leave the current folder.
    Up,
    /// Reload the current folder or archive.
    Refresh,
    /// Wrap the open ZIP or 7z as a self-extractor.
    Sfx,
    /// Show one selected file.
    View,
}

impl Icon {
    fn uri(self) -> &'static str {
        match self {
            Self::FolderOpen => "bytes://lucide/folder-open.svg",
            Self::FolderDown => "bytes://lucide/folder-down.svg",
            Self::CircleCheck => "bytes://lucide/circle-check.svg",
            Self::Plus => "bytes://lucide/plus.svg",
            Self::File => "bytes://lucide/file.svg",
            Self::Folder => "bytes://lucide/folder.svg",
            Self::FileArchive => "bytes://lucide/file-archive.svg",
            Self::TriangleAlert => "bytes://lucide/triangle-alert.svg",
            Self::Archive => "bytes://lucide/archive.svg",
            Self::Link => "bytes://lucide/link.svg",
            Self::Loader => "bytes://lucide/loader-circle.svg",
            Self::X => "bytes://lucide/x.svg",
            Self::ArrowUp => "bytes://lucide/arrow-up.svg",
            Self::Trash => "bytes://lucide/trash-2.svg",
            Self::Info => "bytes://lucide/info.svg",
            Self::Check => "bytes://lucide/check.svg",
            Self::Save => "bytes://lucide/save.svg",
            Self::FolderPlus => "bytes://lucide/folder-plus.svg",
            Self::FolderSearch => "bytes://lucide/folder-search.svg",
            Self::CircleHelp => "bytes://lucide/circle-help.svg",
            Self::Refresh => "bytes://lucide/refresh-cw.svg",
            Self::ChevronDown => "bytes://lucide/chevron-down.svg",
        }
    }

    fn bytes(self) -> &'static [u8] {
        match self {
            Self::FolderOpen => include_bytes!("../assets/lucide/folder-open.svg"),
            Self::FolderDown => include_bytes!("../assets/lucide/folder-down.svg"),
            Self::CircleCheck => include_bytes!("../assets/lucide/circle-check.svg"),
            Self::Plus => include_bytes!("../assets/lucide/plus.svg"),
            Self::File => include_bytes!("../assets/lucide/file.svg"),
            Self::Folder => include_bytes!("../assets/lucide/folder.svg"),
            Self::FileArchive => include_bytes!("../assets/lucide/file-archive.svg"),
            Self::TriangleAlert => include_bytes!("../assets/lucide/triangle-alert.svg"),
            Self::Archive => include_bytes!("../assets/lucide/archive.svg"),
            Self::Link => include_bytes!("../assets/lucide/link.svg"),
            Self::Loader => include_bytes!("../assets/lucide/loader-circle.svg"),
            Self::X => include_bytes!("../assets/lucide/x.svg"),
            Self::ArrowUp => include_bytes!("../assets/lucide/arrow-up.svg"),
            Self::Trash => include_bytes!("../assets/lucide/trash-2.svg"),
            Self::Info => include_bytes!("../assets/lucide/info.svg"),
            Self::Check => include_bytes!("../assets/lucide/check.svg"),
            Self::Save => include_bytes!("../assets/lucide/save.svg"),
            Self::FolderPlus => include_bytes!("../assets/lucide/folder-plus.svg"),
            Self::FolderSearch => include_bytes!("../assets/lucide/folder-search.svg"),
            Self::CircleHelp => include_bytes!("../assets/lucide/circle-help.svg"),
            Self::Refresh => include_bytes!("../assets/lucide/refresh-cw.svg"),
            Self::ChevronDown => include_bytes!("../assets/lucide/chevron-down.svg"),
        }
    }
}

impl ToolIcon {
    fn key(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Add => "add",
            Self::Extract => "extract",
            Self::Test => "test",
            Self::Delete => "delete",
            Self::Info => "info",
            Self::About => "about",
            Self::Up => "up",
            Self::Refresh => "refresh",
            Self::Sfx => "sfx",
            Self::View => "view",
        }
    }

    fn png_bytes(self, enabled: bool, px: u32) -> &'static [u8] {
        match (self, enabled, px) {
            (Self::Open, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_16.png")
            }
            (Self::Open, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_disabled_16.png")
            }
            (Self::Open, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_24.png")
            }
            (Self::Open, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_disabled_24.png")
            }
            (Self::Open, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_32.png")
            }
            (Self::Open, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_disabled_32.png")
            }
            (Self::Open, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_48.png")
            }
            (Self::Open, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_disabled_48.png")
            }
            (Self::Open, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_64.png")
            }
            (Self::Open, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/open_disabled_64.png")
            }

            (Self::Add, true, 16) => include_bytes!("../../../grx/toolbar_icons/png/add_16.png"),
            (Self::Add, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/add_disabled_16.png")
            }
            (Self::Add, true, 24) => include_bytes!("../../../grx/toolbar_icons/png/add_24.png"),
            (Self::Add, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/add_disabled_24.png")
            }
            (Self::Add, true, 32) => include_bytes!("../../../grx/toolbar_icons/png/add_32.png"),
            (Self::Add, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/add_disabled_32.png")
            }
            (Self::Add, true, 48) => include_bytes!("../../../grx/toolbar_icons/png/add_48.png"),
            (Self::Add, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/add_disabled_48.png")
            }
            (Self::Add, true, _) => include_bytes!("../../../grx/toolbar_icons/png/add_64.png"),
            (Self::Add, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/add_disabled_64.png")
            }

            (Self::Extract, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_16.png")
            }
            (Self::Extract, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_disabled_16.png")
            }
            (Self::Extract, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_24.png")
            }
            (Self::Extract, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_disabled_24.png")
            }
            (Self::Extract, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_32.png")
            }
            (Self::Extract, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_disabled_32.png")
            }
            (Self::Extract, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_48.png")
            }
            (Self::Extract, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_disabled_48.png")
            }
            (Self::Extract, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_64.png")
            }
            (Self::Extract, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/extract_disabled_64.png")
            }

            (Self::Test, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_16.png")
            }
            (Self::Test, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_disabled_16.png")
            }
            (Self::Test, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_24.png")
            }
            (Self::Test, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_disabled_24.png")
            }
            (Self::Test, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_32.png")
            }
            (Self::Test, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_disabled_32.png")
            }
            (Self::Test, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_48.png")
            }
            (Self::Test, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_disabled_48.png")
            }
            (Self::Test, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_64.png")
            }
            (Self::Test, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/test_disabled_64.png")
            }

            (Self::Delete, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_16.png")
            }
            (Self::Delete, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_disabled_16.png")
            }
            (Self::Delete, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_24.png")
            }
            (Self::Delete, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_disabled_24.png")
            }
            (Self::Delete, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_32.png")
            }
            (Self::Delete, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_disabled_32.png")
            }
            (Self::Delete, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_48.png")
            }
            (Self::Delete, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_disabled_48.png")
            }
            (Self::Delete, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_64.png")
            }
            (Self::Delete, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/delete_disabled_64.png")
            }

            (Self::Info, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_16.png")
            }
            (Self::Info, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_disabled_16.png")
            }
            (Self::Info, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_24.png")
            }
            (Self::Info, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_disabled_24.png")
            }
            (Self::Info, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_32.png")
            }
            (Self::Info, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_disabled_32.png")
            }
            (Self::Info, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_48.png")
            }
            (Self::Info, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_disabled_48.png")
            }
            (Self::Info, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_64.png")
            }
            (Self::Info, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/info_disabled_64.png")
            }

            (Self::About, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_16.png")
            }
            (Self::About, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_disabled_16.png")
            }
            (Self::About, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_24.png")
            }
            (Self::About, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_disabled_24.png")
            }
            (Self::About, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_32.png")
            }
            (Self::About, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_disabled_32.png")
            }
            (Self::About, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_48.png")
            }
            (Self::About, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_disabled_48.png")
            }
            (Self::About, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_64.png")
            }
            (Self::About, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/about_disabled_64.png")
            }

            (Self::Up, true, 16) => include_bytes!("../../../grx/toolbar_icons/png/up_16.png"),
            (Self::Up, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/up_disabled_16.png")
            }
            (Self::Up, true, 24) => include_bytes!("../../../grx/toolbar_icons/png/up_24.png"),
            (Self::Up, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/up_disabled_24.png")
            }
            (Self::Up, true, 32) => include_bytes!("../../../grx/toolbar_icons/png/up_32.png"),
            (Self::Up, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/up_disabled_32.png")
            }
            (Self::Up, true, 48) => include_bytes!("../../../grx/toolbar_icons/png/up_48.png"),
            (Self::Up, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/up_disabled_48.png")
            }
            (Self::Up, true, _) => include_bytes!("../../../grx/toolbar_icons/png/up_64.png"),
            (Self::Up, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/up_disabled_64.png")
            }

            (Self::Refresh, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_16.png")
            }
            (Self::Refresh, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_disabled_16.png")
            }
            (Self::Refresh, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_24.png")
            }
            (Self::Refresh, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_disabled_24.png")
            }
            (Self::Refresh, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_32.png")
            }
            (Self::Refresh, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_disabled_32.png")
            }
            (Self::Refresh, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_48.png")
            }
            (Self::Refresh, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_disabled_48.png")
            }
            (Self::Refresh, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_64.png")
            }
            (Self::Refresh, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/refresh_disabled_64.png")
            }

            (Self::Sfx, true, 16) => include_bytes!("../../../grx/toolbar_icons/png/sfx_16.png"),
            (Self::Sfx, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/sfx_disabled_16.png")
            }
            (Self::Sfx, true, 24) => include_bytes!("../../../grx/toolbar_icons/png/sfx_24.png"),
            (Self::Sfx, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/sfx_disabled_24.png")
            }
            (Self::Sfx, true, 32) => include_bytes!("../../../grx/toolbar_icons/png/sfx_32.png"),
            (Self::Sfx, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/sfx_disabled_32.png")
            }
            (Self::Sfx, true, 48) => include_bytes!("../../../grx/toolbar_icons/png/sfx_48.png"),
            (Self::Sfx, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/sfx_disabled_48.png")
            }
            (Self::Sfx, true, _) => include_bytes!("../../../grx/toolbar_icons/png/sfx_64.png"),
            (Self::Sfx, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/sfx_disabled_64.png")
            }

            (Self::View, true, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_16.png")
            }
            (Self::View, false, 16) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_disabled_16.png")
            }
            (Self::View, true, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_24.png")
            }
            (Self::View, false, 24) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_disabled_24.png")
            }
            (Self::View, true, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_32.png")
            }
            (Self::View, false, 32) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_disabled_32.png")
            }
            (Self::View, true, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_48.png")
            }
            (Self::View, false, 48) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_disabled_48.png")
            }
            (Self::View, true, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_64.png")
            }
            (Self::View, false, _) => {
                include_bytes!("../../../grx/toolbar_icons/png/view_disabled_64.png")
            }
        }
    }
}

/// Every icon, so the SVG textures can be created before the file table asks for them.
pub const ALL: [Icon; 22] = [
    Icon::FolderOpen,
    Icon::FolderDown,
    Icon::CircleCheck,
    Icon::Plus,
    Icon::File,
    Icon::Folder,
    Icon::FileArchive,
    Icon::TriangleAlert,
    Icon::Archive,
    Icon::Link,
    Icon::Loader,
    Icon::X,
    Icon::ArrowUp,
    Icon::Trash,
    Icon::Info,
    Icon::Check,
    Icon::Save,
    Icon::FolderPlus,
    Icon::FolderSearch,
    Icon::CircleHelp,
    Icon::Refresh,
    Icon::ChevronDown,
];

/// Every toolbar glyph, so [`warm`] can rasterize them with the Lucide set.
pub const TOOL_ALL: [ToolIcon; 11] = [
    ToolIcon::Open,
    ToolIcon::Add,
    ToolIcon::Extract,
    ToolIcon::Test,
    ToolIcon::Delete,
    ToolIcon::Info,
    ToolIcon::About,
    ToolIcon::Up,
    ToolIcon::Refresh,
    ToolIcon::Sfx,
    ToolIcon::View,
];

/// Rasterize the icons once, at the master pixel size.
///
/// Returns immediately when a texture is still loading; the next frame paints
/// the glyph instead of the loader failure mark.
pub fn warm(ctx: &egui::Context) {
    let mut pending = false;
    for icon in ALL {
        pending |= sized_texture(ctx, icon).is_none();
    }
    let need = (ctx.pixels_per_point() * 64.0).round().max(1.0) as u32;
    let px = pick_tool_png_size(need);
    for icon in TOOL_ALL {
        pending |= tool_texture(ctx, icon, true, px).is_none();
        pending |= tool_texture(ctx, icon, false, px).is_none();
        pending |= tool_texture(ctx, icon, true, 16).is_none();
        pending |= tool_texture(ctx, icon, false, 16).is_none();
    }
    if pending {
        ctx.request_repaint();
    }
}

/// Tinted glyph for `Button::image_and_text`.
pub fn image(
    ctx: &egui::Context,
    icon: Icon,
    tint: Color32,
    size: f32,
) -> Option<egui::Image<'static>> {
    tinted(ctx, icon, tint, Vec2::splat(size))
}

/// Draw `icon` at `size` points. The widget does not take clicks, so a table row can.
pub fn show(ui: &mut egui::Ui, icon: Icon, tint: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::empty());
    paint(ui, icon, tint, rect);
}

/// Draw `icon` inside `rect`. Skips the frame when the texture is not ready.
pub fn paint(ui: &mut egui::Ui, icon: Icon, tint: Color32, rect: Rect) {
    let Some(image) = tinted(ui.ctx(), icon, tint, rect.size()) else {
        return;
    };
    image.paint_at(ui, rect);
}

/// Draw a pre-colored toolbar glyph. Disabled buttons use their gray PNG.
pub fn paint_tool(ui: &mut egui::Ui, icon: ToolIcon, enabled: bool, rect: Rect) {
    let Some(image) = tool_image(ui.ctx(), icon, enabled, rect.size()) else {
        return;
    };
    image.paint_at(ui, rect);
}

/// Spin the loader. Callers should keep requesting repaints while work is running.
pub fn show_spinning(ui: &mut egui::Ui, tint: Color32, size: f32) {
    let angle = ui.input(|input| input.time) as f32;
    if let Some(image) = tinted(ui.ctx(), Icon::Loader, tint, Vec2::splat(size)) {
        ui.add(image.rotate(angle, Vec2::splat(0.5)));
    } else {
        ui.allocate_exact_size(Vec2::splat(size), Sense::empty());
    }
    ui.ctx().request_repaint();
}

/// Allocate [`SIZE_CARET`] and paint a triangle.
pub fn caret(ui: &mut egui::Ui, caret: Caret, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(SIZE_CARET, Sense::empty());
    paint_caret(ui, caret, color, rect);
}

/// Classic tree box: a square with `+` when collapsed and `-` when open.
pub fn paint_expander(ui: &mut egui::Ui, expanded: bool, color: Color32, rect: Rect) {
    let box_rect = Rect::from_center_size(rect.center(), Vec2::splat(11.0));
    let stroke = egui::Stroke::new(1.0_f32, color);
    ui.painter()
        .rect_stroke(box_rect, 1.0, stroke, egui::StrokeKind::Inside);
    let mid = box_rect.center();
    let arm = 3.0;
    ui.painter().line_segment(
        [mid - egui::vec2(arm, 0.0), mid + egui::vec2(arm, 0.0)],
        stroke,
    );
    if !expanded {
        ui.painter().line_segment(
            [mid - egui::vec2(0.0, arm), mid + egui::vec2(0.0, arm)],
            stroke,
        );
    }
}

/// Draw a filled triangle in `rect`.
pub fn paint_caret(ui: &mut egui::Ui, caret: Caret, color: Color32, rect: Rect) {
    let points = match caret {
        Caret::Up => vec![
            egui::pos2(rect.center().x, rect.top()),
            rect.right_bottom(),
            rect.left_bottom(),
        ],
        Caret::Down => vec![
            rect.left_top(),
            rect.right_top(),
            egui::pos2(rect.center().x, rect.bottom()),
        ],
    };
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

fn tinted(
    ctx: &egui::Context,
    icon: Icon,
    tint: Color32,
    size: Vec2,
) -> Option<egui::Image<'static>> {
    let texture = sized_texture(ctx, icon)?;
    Some(
        egui::Image::from_texture(texture)
            .tint(tint)
            .fit_to_exact_size(size)
            .sense(Sense::empty())
            .show_loading_spinner(false),
    )
}

fn sized_texture(ctx: &egui::Context, icon: Icon) -> Option<egui::load::SizedTexture> {
    load_bytes(ctx, icon.uri(), icon.bytes(), MASTER_POINTS)
}

fn tool_image(
    ctx: &egui::Context,
    icon: ToolIcon,
    enabled: bool,
    size: Vec2,
) -> Option<egui::Image<'static>> {
    let need = (ctx.pixels_per_point() * size.x.max(size.y))
        .round()
        .max(1.0) as u32;
    let px = pick_tool_png_size(need);
    let texture = tool_texture(ctx, icon, enabled, px)?;
    Some(
        egui::Image::from_texture(texture)
            .fit_to_exact_size(size)
            .sense(Sense::empty())
            .show_loading_spinner(false),
    )
}

fn tool_texture(
    ctx: &egui::Context,
    icon: ToolIcon,
    enabled: bool,
    px: u32,
) -> Option<egui::load::SizedTexture> {
    let uri = format!(
        "bytes://toolbar/{}/{}/{}.png",
        icon.key(),
        if enabled { "on" } else { "off" },
        px,
    );
    load_bytes(ctx, uri, icon.png_bytes(enabled, px), px as f32)
}

/// Smallest shipped PNG edge that covers `need` pixels, capped at 64.
fn pick_tool_png_size(need: u32) -> u32 {
    for size in TOOL_PNG_SIZES {
        if size >= need {
            return size;
        }
    }
    64
}

fn load_bytes(
    ctx: &egui::Context,
    uri: impl Into<std::borrow::Cow<'static, str>>,
    bytes: impl Into<egui::load::Bytes>,
    points: f32,
) -> Option<egui::load::SizedTexture> {
    let pixels = (ctx.pixels_per_point() * points).round().max(1.0) as u32;
    match egui::Image::from_bytes(uri, bytes)
        .sense(Sense::empty())
        .show_loading_spinner(false)
        .source(ctx)
        .load(
            ctx,
            egui::TextureOptions::LINEAR,
            egui::load::SizeHint::Size {
                width: pixels,
                height: pixels,
                maintain_aspect_ratio: false,
            },
        ) {
        Ok(egui::load::TexturePoll::Ready { texture }) => Some(texture),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ALL, TOOL_ALL, TOOL_PNG_SIZES, pick_tool_png_size};

    #[test]
    fn lucide_strokes_are_white_so_tints_show() {
        for icon in ALL {
            let svg = std::str::from_utf8(icon.bytes()).expect("svg is utf-8");
            assert!(
                !svg.contains("#000000"),
                "{} still has a black stroke",
                icon.uri()
            );
            assert!(
                svg.contains("#ffffff"),
                "{} has no white stroke for tinting",
                icon.uri()
            );
        }
    }

    #[test]
    fn toolbar_pngs_are_valid_for_each_size() {
        for icon in TOOL_ALL {
            for &px in &TOOL_PNG_SIZES {
                for enabled in [true, false] {
                    let bytes = icon.png_bytes(enabled, px);
                    assert!(
                        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
                        "{} {} {} is not a PNG",
                        icon.key(),
                        if enabled { "on" } else { "off" },
                        px
                    );
                    assert!(
                        bytes.len() > 32,
                        "{} {} {} is too small",
                        icon.key(),
                        if enabled { "on" } else { "off" },
                        px
                    );
                }
            }
        }
    }

    #[test]
    fn pick_tool_png_size_covers_need() {
        assert_eq!(pick_tool_png_size(1), 16);
        assert_eq!(pick_tool_png_size(16), 16);
        assert_eq!(pick_tool_png_size(17), 24);
        assert_eq!(pick_tool_png_size(48), 48);
        assert_eq!(pick_tool_png_size(49), 64);
        assert_eq!(pick_tool_png_size(200), 64);
    }
}
