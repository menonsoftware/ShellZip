//! Platform shell pictures. Win32 and AppKit stay inside the icon crates.

use super::{IconKey, RgbaIcon};

/// Raster `key` at `pixels` on a side. `None` means the list keeps Lucide.
pub fn raster(key: &IconKey, pixels: u32) -> Option<RgbaIcon> {
    desktop::raster(key, pixels)
}

#[cfg(any(windows, target_os = "macos"))]
mod desktop {
    use super::{IconKey, RgbaIcon, decode_png_icon, icon_edge, icon_edge_u16};
    use std::path::Path;

    pub fn raster(key: &IconKey, pixels: u32) -> Option<RgbaIcon> {
        match key {
            IconKey::Extension(ext) => extension_icon(ext, pixels),
            IconKey::Folder => folder_icon(pixels),
            IconKey::Path(path) => path_icon(path, pixels),
            IconKey::Computer => computer_icon(pixels),
        }
    }

    fn extension_icon(ext: &str, pixels: u32) -> Option<RgbaIcon> {
        let name = if ext.is_empty() {
            "file".to_string()
        } else {
            format!("file.{ext}")
        };
        let png = systemicons::get_icon(&name, icon_edge(pixels)).ok()?;
        decode_png_icon(&png)
    }

    fn path_icon(path: &Path, pixels: u32) -> Option<RgbaIcon> {
        let icon = file_icon_provider::get_file_icon(path, icon_edge_u16(pixels)).ok()?;
        packed(icon)
    }

    fn folder_icon(pixels: u32) -> Option<RgbaIcon> {
        path_icon(&sample_dir()?, pixels)
    }

    /// This PC. `imageres.dll` extract index 104 is resource 109
    /// (`SIID_DESKTOPPC`). `windows-icons` takes that index plus one.
    #[cfg(windows)]
    fn computer_icon(_pixels: u32) -> Option<RgbaIcon> {
        let spec = windows_icons::DllIcon::new().with_imageres(105);
        let image = windows_icons::get_icon_by_dll(spec).ok()?;
        packed_rgba(image)
    }

    #[cfg(target_os = "macos")]
    fn computer_icon(_pixels: u32) -> Option<RgbaIcon> {
        None
    }

    /// A normal directory, so the shell returns the yellow folder.
    ///
    /// Desktop's picture is a monitor. Home can be a person or a house.
    /// Either one is the wrong stand-in for every folder in the tree.
    fn sample_dir() -> Option<std::path::PathBuf> {
        let temp = std::env::temp_dir();
        if temp.is_dir() {
            return Some(temp);
        }
        let dirs = directories::UserDirs::new()?;
        let home = dirs.home_dir();
        if home.is_dir() {
            return Some(home.to_path_buf());
        }
        dirs.desktop_dir()
            .filter(|path| path.is_dir())
            .map(Path::to_path_buf)
    }

    fn packed(icon: file_icon_provider::Icon) -> Option<RgbaIcon> {
        let expected = (icon.width as usize)
            .checked_mul(icon.height as usize)?
            .checked_mul(4)?;
        if icon.width == 0 || icon.height == 0 || icon.pixels.len() != expected {
            return None;
        }
        Some(RgbaIcon {
            width: icon.width,
            height: icon.height,
            pixels: icon.pixels,
        })
    }

    #[cfg(windows)]
    fn packed_rgba(image: image::RgbaImage) -> Option<RgbaIcon> {
        let width = image.width();
        let height = image.height();
        let pixels = image.into_raw();
        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if width == 0 || height == 0 || pixels.len() != expected {
            return None;
        }
        Some(RgbaIcon {
            width,
            height,
            pixels,
        })
    }
}

#[cfg(target_os = "linux")]
mod desktop {
    use super::{IconKey, RgbaIcon, decode_png_icon, icon_edge_u16};
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    pub fn raster(key: &IconKey, pixels: u32) -> Option<RgbaIcon> {
        let names = names_for(key);
        let path = find_icon(&names, pixels)?;
        decode_file(&path, pixels)
    }

    fn names_for(key: &IconKey) -> Vec<String> {
        match key {
            IconKey::Folder => vec!["folder".to_string(), "inode-directory".to_string()],
            IconKey::Extension(ext) => mime_names(ext),
            IconKey::Path(path) => path_names(path),
            IconKey::Computer => vec![
                "computer".to_string(),
                "computer-symbolic".to_string(),
                "drive-harddisk".to_string(),
            ],
        }
    }

    fn path_names(path: &Path) -> Vec<String> {
        if is_desktop(path) {
            return vec![
                "user-desktop".to_string(),
                "desktop".to_string(),
                "folder".to_string(),
            ];
        }
        if is_home(path) {
            return vec![
                "user-home".to_string(),
                "folder-home".to_string(),
                "folder".to_string(),
            ];
        }
        if is_volume(path) {
            return vec![
                "drive-harddisk".to_string(),
                "drive-harddisk-symbolic".to_string(),
                "folder".to_string(),
            ];
        }
        if path.is_dir() || path.extension().is_none() {
            return vec!["folder".to_string(), "inode-directory".to_string()];
        }
        let ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        mime_names(&ext)
    }

    fn is_desktop(path: &Path) -> bool {
        directories::UserDirs::new()
            .and_then(|dirs| dirs.desktop_dir().map(Path::to_path_buf))
            .is_some_and(|desktop| desktop == path)
    }

    fn is_home(path: &Path) -> bool {
        directories::UserDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .is_some_and(|home| home == path)
    }

    fn is_volume(path: &Path) -> bool {
        if path == Path::new("/") {
            return true;
        }
        let Some(parent) = path.parent() else {
            return false;
        };
        (parent == Path::new("/mnt")
            || parent == Path::new("/media")
            || parent == Path::new("/run/media")
            || parent == Path::new("/Volumes"))
            && !is_home(path)
            && !is_desktop(path)
    }

    fn mime_names(ext: &str) -> Vec<String> {
        let db = mime_db();
        let file = if ext.is_empty() {
            "file".to_string()
        } else {
            format!("file.{ext}")
        };
        let mut names = Vec::new();
        if let Some(mime) = db.get_mime_types_from_file_name(&file).into_iter().next() {
            names.extend(db.lookup_icon_names(&mime));
            if let Some(generic) = db.lookup_generic_icon_name(&mime) {
                names.push(generic);
            }
        }
        if names.is_empty() {
            names.push("application-octet-stream".to_string());
            names.push("text-x-generic".to_string());
        }
        names
    }

    fn mime_db() -> &'static xdg_mime::SharedMimeInfo {
        static DB: OnceLock<xdg_mime::SharedMimeInfo> = OnceLock::new();
        DB.get_or_init(xdg_mime::SharedMimeInfo::new)
    }

    fn find_icon(names: &[String], pixels: u32) -> Option<PathBuf> {
        let size = icon_edge_u16(pixels);
        let theme = icon_theme();
        for name in names {
            if let Some(path) = freedesktop_icons::lookup(name)
                .with_size(size)
                .with_theme(&theme)
                .with_cache()
                .find()
            {
                return Some(path);
            }
            if let Some(path) = freedesktop_icons::lookup(name)
                .with_size(size)
                .with_cache()
                .find()
            {
                return Some(path);
            }
        }
        None
    }

    fn icon_theme() -> String {
        static THEME: OnceLock<String> = OnceLock::new();
        THEME
            .get_or_init(|| {
                for relative in ["gtk-4.0/settings.ini", "gtk-3.0/settings.ini"] {
                    let Some(base) = directories::BaseDirs::new() else {
                        continue;
                    };
                    if let Some(name) = theme_from_ini(&base.config_dir().join(relative)) {
                        return name;
                    }
                }
                "Adwaita".to_string()
            })
            .clone()
    }

    fn theme_from_ini(path: &Path) -> Option<String> {
        let text = std::fs::read_to_string(path).ok()?;
        for line in text.lines() {
            let line = line.trim();
            let Some(value) = line.strip_prefix("gtk-icon-theme-name") else {
                continue;
            };
            let value = value
                .trim_start_matches([' ', '='])
                .trim()
                .trim_matches('"');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }

    fn decode_file(path: &Path, pixels: u32) -> Option<RgbaIcon> {
        let bytes = std::fs::read(path).ok()?;
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("svg" | "svgz") => decode_svg(&bytes, pixels),
            _ => decode_png_icon(&bytes).or_else(|| decode_svg(&bytes, pixels)),
        }
    }

    fn decode_svg(bytes: &[u8], pixels: u32) -> Option<RgbaIcon> {
        let options = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(bytes, &options).ok()?;
        let size = tree.size();
        let width = size.width();
        let height = size.height();
        if width <= 0.0 || height <= 0.0 {
            return None;
        }
        let edge = pixels.max(1);
        let mut pixmap = resvg::tiny_skia::Pixmap::new(edge, edge)?;
        {
            let mut view = pixmap.as_mut();
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::from_scale(edge as f32 / width, edge as f32 / height),
                &mut view,
            );
        }
        let rgba = pixmap.take_demultiplied();
        let expected = (edge as usize).checked_mul(edge as usize)?.checked_mul(4)?;
        if rgba.len() != expected {
            return None;
        }
        Some(RgbaIcon {
            width: edge,
            height: edge,
            pixels: rgba,
        })
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod desktop {
    use super::{IconKey, RgbaIcon};

    pub fn raster(_key: &IconKey, _pixels: u32) -> Option<RgbaIcon> {
        None
    }
}

pub(super) fn decode_png_icon(bytes: &[u8]) -> Option<RgbaIcon> {
    super::decode_png(bytes)
}

#[cfg(any(windows, target_os = "macos"))]
fn icon_edge(pixels: u32) -> i32 {
    i32::from(icon_edge_u16(pixels))
}

#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
fn icon_edge_u16(pixels: u32) -> u16 {
    let edge = pixels.clamp(16, 256);
    u16::try_from(edge).unwrap_or(16)
}

#[cfg(windows)]
#[cfg(test)]
mod tests {
    use super::raster;
    use crate::shell_icons::IconKey;

    #[test]
    fn text_file_extension_has_a_shell_icon() {
        let icon = raster(&IconKey::Extension("txt".to_string()), 16);
        assert!(
            icon.is_some_and(|icon| icon.width > 0 && !icon.pixels.is_empty()),
            "file.txt should resolve through SHGetFileInfo"
        );
    }

    #[test]
    fn folder_icon_is_yellow_not_the_desktop_monitor() {
        let icon = raster(&IconKey::Folder, 32).expect("folder icon");
        let mut yellow = 0;
        let mut blue = 0;
        for px in icon.pixels.as_chunks::<4>().0 {
            if px[3] < 32 {
                continue;
            }
            let (r, g, b) = (u16::from(px[0]), u16::from(px[1]), u16::from(px[2]));
            if r > 180 && g > 140 && b < 120 {
                yellow += 1;
            }
            if b > 140 && b > r + 40 && b > g {
                blue += 1;
            }
        }
        assert!(
            yellow > blue,
            "the shared folder picture should be the yellow shell folder, not the blue desktop monitor (yellow={yellow} blue={blue})"
        );
    }

    #[test]
    fn folder_and_drive_have_shell_icons() {
        let folder = raster(&IconKey::Folder, 16);
        assert!(
            folder.is_some_and(|icon| icon.width > 0 && !icon.pixels.is_empty()),
            "a real directory should yield a shell folder icon"
        );
        let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
        let drive = std::path::PathBuf::from(format!("{drive}\\"));
        let icon = raster(&IconKey::Path(drive), 32).expect("drive icon");
        let folder = raster(&IconKey::Folder, 32).expect("folder icon");
        assert_ne!(
            icon.pixels, folder.pixels,
            "a drive must not reuse the yellow folder"
        );
        let computer = raster(&IconKey::Computer, 32).expect("This PC icon");
        assert!(
            blue_pixels(&computer) > yellow_pixels(&computer),
            "This PC should be the shell monitor, not the yellow folder"
        );
        if let Some(desktop) = directories::UserDirs::new()
            .and_then(|dirs| dirs.desktop_dir().map(std::path::Path::to_path_buf))
            .filter(|path| path.is_dir())
        {
            let desktop = raster(&IconKey::Path(desktop), 32).expect("desktop icon");
            assert_ne!(
                desktop.pixels, folder.pixels,
                "Desktop must not reuse the yellow folder"
            );
        }
    }

    fn yellow_pixels(icon: &crate::shell_icons::RgbaIcon) -> u32 {
        color_count(icon, true)
    }

    fn blue_pixels(icon: &crate::shell_icons::RgbaIcon) -> u32 {
        color_count(icon, false)
    }

    fn color_count(icon: &crate::shell_icons::RgbaIcon, yellow: bool) -> u32 {
        let mut count = 0;
        for px in icon.pixels.as_chunks::<4>().0 {
            if px[3] < 32 {
                continue;
            }
            let (r, g, b) = (u16::from(px[0]), u16::from(px[1]), u16::from(px[2]));
            let hit = if yellow {
                r > 180 && g > 140 && b < 120
            } else {
                b > 140 && b > r + 40 && b > g
            };
            if hit {
                count += 1;
            }
        }
        count
    }
}
