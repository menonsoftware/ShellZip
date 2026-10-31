//! Lucide icons and the pre-colored toolbar glyphs.
//!
//! Lucide strokes are white so [`paint`] can tint them. Toolbar art from
//! `assets/toolbar/` already has brand colors; [`paint_tool`] draws it as-is
//! and only recolors dark-theme strokes so the marks stay visible.
//!
//! Source: <https://lucide.dev/icons/>. The SVG files and their ISC license
//! live in `assets/lucide/`.
//!
//! Each icon is rasterized once at [`MASTER_POINTS`] and then scaled. A table
//! sizing pass otherwise asks for a new SVG size every frame and the loader
//! draws its failure mark above the file list.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use egui::{Color32, Rect, Sense, Vec2};

type DarkSvgCache = Mutex<HashMap<&'static str, Arc<[u8]>>>;

/// File-list row glyph.
pub const SIZE_ROW: f32 = 16.0;
/// Status-line glyph.
pub const SIZE_STATUS: f32 = 14.0;
/// Sort and address-bar caret.
pub const SIZE_CARET: Vec2 = Vec2::new(9.0, 8.0);

/// One raster size for every icon. Painting scales this texture.
const MASTER_POINTS: f32 = 32.0;

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

/// Pre-colored toolbar and address-bar glyphs from `assets/toolbar/`.
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

    fn has_disabled_art(self) -> bool {
        matches!(
            self,
            Self::Extract | Self::Test | Self::Delete | Self::Sfx | Self::View
        )
    }

    fn bytes(self, enabled: bool) -> &'static [u8] {
        match (self, enabled) {
            (Self::Extract, false) => include_bytes!("../assets/toolbar/extract_disabled.svg"),
            (Self::Test, false) => include_bytes!("../assets/toolbar/test_disabled.svg"),
            (Self::Delete, false) => include_bytes!("../assets/toolbar/delete_disabled.svg"),
            (Self::Open, _) => include_bytes!("../assets/toolbar/open.svg"),
            (Self::Add, _) => include_bytes!("../assets/toolbar/add.svg"),
            (Self::Extract, true) => include_bytes!("../assets/toolbar/extract.svg"),
            (Self::Test, true) => include_bytes!("../assets/toolbar/test.svg"),
            (Self::Delete, true) => include_bytes!("../assets/toolbar/delete.svg"),
            (Self::Info, _) => include_bytes!("../assets/toolbar/info.svg"),
            (Self::About, _) => include_bytes!("../assets/toolbar/about.svg"),
            (Self::Up, _) => include_bytes!("../assets/toolbar/up.svg"),
            (Self::Refresh, _) => include_bytes!("../assets/toolbar/refresh.svg"),
            (Self::Sfx, false) => include_bytes!("../assets/toolbar/sfx_disabled.svg"),
            (Self::Sfx, true) => include_bytes!("../assets/toolbar/sfx.svg"),
            (Self::View, false) => include_bytes!("../assets/toolbar/view_disabled.svg"),
            (Self::View, true) => include_bytes!("../assets/toolbar/view.svg"),
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
    let dark = matches!(ctx.theme(), egui::Theme::Dark);
    for icon in TOOL_ALL {
        pending |= tool_texture(ctx, icon, true, dark).is_none();
        pending |= tool_texture(ctx, icon, false, dark).is_none();
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

/// Draw a pre-colored toolbar glyph. Disabled extract, test, and delete use
/// their own art; the others fade.
pub fn paint_tool(ui: &mut egui::Ui, icon: ToolIcon, enabled: bool, rect: Rect) {
    let dark = matches!(ui.ctx().theme(), egui::Theme::Dark);
    let Some(image) = tool_image(ui.ctx(), icon, enabled, dark, rect.size()) else {
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
    load_bytes(ctx, icon.uri(), icon.bytes())
}

fn tool_image(
    ctx: &egui::Context,
    icon: ToolIcon,
    enabled: bool,
    dark: bool,
    size: Vec2,
) -> Option<egui::Image<'static>> {
    let texture = tool_texture(ctx, icon, enabled, dark)?;
    let tint = if enabled || icon.has_disabled_art() {
        Color32::WHITE
    } else {
        Color32::from_white_alpha(97)
    };
    Some(
        egui::Image::from_texture(texture)
            .tint(tint)
            .fit_to_exact_size(size)
            .sense(Sense::empty())
            .show_loading_spinner(false),
    )
}

fn tool_texture(
    ctx: &egui::Context,
    icon: ToolIcon,
    enabled: bool,
    dark: bool,
) -> Option<egui::load::SizedTexture> {
    let uri = format!(
        "bytes://toolbar/{}/{}/{}.svg",
        icon.key(),
        if enabled { "on" } else { "off" },
        if dark { "dark" } else { "light" },
    );
    load_bytes(ctx, uri, tool_bytes(icon, enabled, dark))
}

fn tool_bytes(icon: ToolIcon, enabled: bool, dark: bool) -> egui::load::Bytes {
    let raw = icon.bytes(enabled);
    if !dark {
        return egui::load::Bytes::Static(raw);
    }
    static CACHE: OnceLock<DarkSvgCache> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let key = tool_cache_key(icon, enabled);
    if let Some(bytes) = map.get(key) {
        return egui::load::Bytes::Shared(Arc::clone(bytes));
    }
    let bytes = Arc::<[u8]>::from(recolor_dark(raw));
    map.insert(key, Arc::clone(&bytes));
    egui::load::Bytes::Shared(bytes)
}

fn tool_cache_key(icon: ToolIcon, enabled: bool) -> &'static str {
    match (icon, enabled) {
        (ToolIcon::Open, true) => "open-on",
        (ToolIcon::Open, false) => "open-off",
        (ToolIcon::Add, true) => "add-on",
        (ToolIcon::Add, false) => "add-off",
        (ToolIcon::Extract, true) => "extract-on",
        (ToolIcon::Extract, false) => "extract-off",
        (ToolIcon::Test, true) => "test-on",
        (ToolIcon::Test, false) => "test-off",
        (ToolIcon::Delete, true) => "delete-on",
        (ToolIcon::Delete, false) => "delete-off",
        (ToolIcon::Info, true) => "info-on",
        (ToolIcon::Info, false) => "info-off",
        (ToolIcon::About, true) => "about-on",
        (ToolIcon::About, false) => "about-off",
        (ToolIcon::Up, true) => "up-on",
        (ToolIcon::Up, false) => "up-off",
        (ToolIcon::Refresh, true) => "refresh-on",
        (ToolIcon::Refresh, false) => "refresh-off",
        (ToolIcon::Sfx, true) => "sfx-on",
        (ToolIcon::Sfx, false) => "sfx-off",
        (ToolIcon::View, true) => "view-on",
        (ToolIcon::View, false) => "view-off",
    }
}

/// Lighten the dark ink so the marks read on a dark toolbar.
fn recolor_dark(svg: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(svg) else {
        return svg.to_vec();
    };
    text.replace("#2B2F36", "#E8ECF0")
        .replace("#8A8F98", "#B4B8C0")
        .into_bytes()
}

fn load_bytes(
    ctx: &egui::Context,
    uri: impl Into<std::borrow::Cow<'static, str>>,
    bytes: impl Into<egui::load::Bytes>,
) -> Option<egui::load::SizedTexture> {
    let pixels = (ctx.pixels_per_point() * MASTER_POINTS).round().max(1.0) as u32;
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
    use super::{ALL, TOOL_ALL, recolor_dark};

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
    fn toolbar_svgs_carry_the_brand_colors() {
        for icon in TOOL_ALL {
            let svg = std::str::from_utf8(icon.bytes(true)).expect("svg is utf-8");
            assert!(
                svg.contains("#2B2F36") || svg.contains("#C8642A"),
                "{} is missing the toolbar palette",
                icon.key()
            );
            if icon.has_disabled_art() {
                let off = std::str::from_utf8(icon.bytes(false)).expect("svg is utf-8");
                assert!(
                    off.contains("#8A8F98"),
                    "{} disabled art is not the gray variant",
                    icon.key()
                );
            }
        }
    }

    #[test]
    fn dark_toolbar_strokes_are_light() {
        let out = String::from_utf8(recolor_dark(b"stroke=\"#2B2F36\" #8A8F98")).expect("utf-8");
        assert!(out.contains("#E8ECF0"));
        assert!(out.contains("#B4B8C0"));
        assert!(!out.contains("#2B2F36"));
        assert!(!out.contains("#8A8F98"));
    }
}
