//! Lucide icons. egui tints by multiplying, so a black shape stays black.
//! The folder file is white and picks up [`crate::theme::FOLDER`].
//!
//! Source: <https://lucide.dev/icons/>. The SVG files and their ISC license
//! live in `assets/lucide/`.
//!
//! Each icon is rasterized once at [`MASTER_POINTS`] and then scaled. A table
//! sizing pass otherwise asks for a new SVG size every frame and the loader
//! draws its failure mark above the file list.

use egui::{Color32, Rect, Sense, Vec2};

/// File-list row glyph.
pub const SIZE_ROW: f32 = 16.0;
/// Toolbar glyph.
pub const SIZE_TOOL: f32 = 28.0;
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
}

/// Up or down triangle, painted so we do not depend on font glyphs.
#[derive(Clone, Copy)]
pub enum Caret {
    /// Sort ascending, or the address-bar parent list.
    Up,
    /// Sort descending, or an expanded tree node.
    Down,
    /// A collapsed tree node.
    Right,
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
        }
    }
}

/// Every icon, so the SVG textures can be created before the file table asks for them.
pub const ALL: [Icon; 20] = [
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
    if pending {
        ctx.request_repaint();
    }
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
        Caret::Right => vec![
            rect.left_top(),
            rect.left_bottom(),
            egui::pos2(rect.right(), rect.center().y),
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
    let pixels = (ctx.pixels_per_point() * MASTER_POINTS).round().max(1.0) as u32;
    match egui::Image::from_bytes(icon.uri(), icon.bytes())
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
    use super::Icon;

    #[test]
    fn folder_svg_is_white_so_the_yellow_tint_shows() {
        let svg = std::str::from_utf8(Icon::Folder.bytes()).expect("folder svg is utf-8");
        assert!(svg.contains("fill=\"#ffffff\""));
        assert!(svg.contains("stroke=\"#ffffff\""));
        assert!(!svg.contains("#000000"));
    }
}
