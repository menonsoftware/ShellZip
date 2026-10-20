//! Lucide icons, recolored so egui can tint `stroke="#000000"`.
//!
//! Source: <https://lucide.dev/icons/>. The SVG files and their ISC license
//! live in `assets/lucide/`.

use egui::{Color32, Vec2};

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
        }
    }
}

/// Draw `icon` at `size` points.
pub fn show(ui: &mut egui::Ui, icon: Icon, tint: Color32, size: f32) {
    ui.add(image(icon, tint, size));
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
}
