//! Lucide icons, recolored so egui can tint `stroke="#000000"`.
//!
//! Source: <https://lucide.dev/icons/>. The SVG files and their ISC license
//! live in `assets/lucide/`.

use egui::{Color32, Rect, Sense, Vec2};

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
        }
    }
}

/// Every icon, so the SVG textures can be created before the file table asks for them.
pub const ALL: [Icon; 13] = [
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
];

/// Rasterize the icons once, at the same pixel size the painter requests.
///
/// A table sizing pass otherwise asks for a different SVG size every frame and
/// the loader draws its failure mark instead of the glyph.
pub fn warm(ctx: &egui::Context) {
    let mut pending = false;
    for icon in ALL {
        for points in [16.0, 28.0] {
            pending |= !raster_ready(ctx, icon, points);
        }
    }
    if pending {
        ctx.request_repaint();
    }
}

fn raster_ready(ctx: &egui::Context, icon: Icon, points: f32) -> bool {
    let pixels = (ctx.pixels_per_point() * points).round().max(1.0) as u32;
    let loaded = image(icon, Color32::WHITE, points).source(ctx).load(
        ctx,
        egui::TextureOptions::LINEAR,
        egui::load::SizeHint::Size {
            width: pixels,
            height: pixels,
            maintain_aspect_ratio: false,
        },
    );
    matches!(loaded, Ok(egui::load::TexturePoll::Ready { .. }))
}

/// Draw `icon` at `size` points. The widget does not take clicks, so a table row can.
pub fn show(ui: &mut egui::Ui, icon: Icon, tint: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::empty());
    image(icon, tint, size).paint_at(ui, rect);
}

/// Draw `icon` inside `rect`.
pub fn paint(ui: &mut egui::Ui, icon: Icon, tint: Color32, rect: Rect) {
    let size = rect.width().min(rect.height());
    image(icon, tint, size).paint_at(ui, rect);
}

/// Spin the loader. Callers should keep requesting repaints while work is running.
pub fn show_spinning(ui: &mut egui::Ui, tint: Color32, size: f32) {
    let angle = ui.input(|input| input.time) as f32;
    ui.add(image(Icon::Loader, tint, size).rotate(angle, Vec2::splat(0.5)));
    ui.ctx().request_repaint();
}

fn image(icon: Icon, tint: Color32, size: f32) -> egui::Image<'static> {
    egui::Image::from_bytes(icon.uri(), icon.bytes())
        .tint(tint)
        .fit_to_exact_size(Vec2::splat(size))
        .sense(Sense::empty())
        .show_loading_spinner(false)
}
