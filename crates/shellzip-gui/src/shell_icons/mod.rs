//! Operating-system icons for the file list and the address-bar tree.
//!
//! Lookups run on a background thread. The window paints a Lucide glyph until
//! the raster arrives, then keeps the texture. A failed lookup stays on Lucide.

mod platform;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use egui::{Color32, Pos2, Rect, Sense, TextureHandle, Vec2};

use crate::explorer::{Row, RowKind, Target};

/// Which shell picture to load.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IconKey {
    /// One directory picture shared by `..`, folders, and archive folders.
    Folder,
    /// Lowercase extension without the dot. Empty means the name has none.
    Extension(String),
    /// A real path. Drives, Desktop, and `.exe` / `.lnk` files use this.
    Path(PathBuf),
    /// The virtual computer node (This PC).
    Computer,
}

/// Straight RGBA, top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaIcon {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes.
    pub pixels: Vec<u8>,
}

struct Request {
    key: IconKey,
    pixels: u32,
}

struct Raster {
    key: IconKey,
    pixels: u32,
    image: Option<RgbaIcon>,
}

/// Textures already decoded, plus the thread that fills them.
pub struct IconCache {
    ready: HashMap<(IconKey, u32), TextureHandle>,
    pending: HashSet<(IconKey, u32)>,
    failed: HashSet<(IconKey, u32)>,
    next_id: u64,
    tx: Sender<Request>,
    rx: Receiver<Raster>,
}

impl IconCache {
    /// Shell provider. Misses stay on the Lucide fallback.
    pub fn new() -> Self {
        Self::spawn(platform::raster)
    }

    /// `provider` is called on the icon thread.
    pub fn spawn(provider: fn(&IconKey, u32) -> Option<RgbaIcon>) -> Self {
        let (tx, request_rx) = mpsc::channel::<Request>();
        let (result_tx, rx) = mpsc::channel::<Raster>();
        let started = std::thread::Builder::new()
            .name("shellzip-icons".to_string())
            .spawn(move || {
                while let Ok(request) = request_rx.recv() {
                    let image = provider(&request.key, request.pixels);
                    if result_tx
                        .send(Raster {
                            key: request.key,
                            pixels: request.pixels,
                            image,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .is_ok();
        if !started {
            // The request receiver was dropped with the closure. Sends fail and
            // lookups are recorded as failed, which keeps the Lucide glyph.
        }
        Self {
            ready: HashMap::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            next_id: 0,
            tx,
            rx,
        }
    }

    /// True while a lookup has not returned.
    pub fn is_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Texture for `key` at `pixels`, requesting it on a miss.
    pub fn texture(
        &mut self,
        ctx: &egui::Context,
        key: &IconKey,
        pixels: u32,
    ) -> Option<egui::TextureId> {
        self.poll(ctx);
        let id = (key.clone(), pixels);
        if let Some(texture) = self.ready.get(&id) {
            return Some(texture.id());
        }
        if self.failed.contains(&id) {
            return None;
        }
        if self.pending.insert(id) {
            let _ = self.tx.send(Request {
                key: key.clone(),
                pixels,
            });
        }
        ctx.request_repaint();
        None
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(raster) = self.rx.try_recv() {
            let id = (raster.key, raster.pixels);
            self.pending.remove(&id);
            match raster.image.and_then(validated) {
                Some(image) => {
                    let color = egui::ColorImage::from_rgba_unmultiplied(
                        [image.width as usize, image.height as usize],
                        &image.pixels,
                    );
                    self.next_id += 1;
                    let texture = ctx.load_texture(
                        format!("shell-icon-{}", self.next_id),
                        color,
                        egui::TextureOptions::LINEAR,
                    );
                    self.ready.insert(id, texture);
                }
                None => {
                    self.failed.insert(id);
                }
            }
        }
    }
}

fn validated(image: RgbaIcon) -> Option<RgbaIcon> {
    let expected = (image.width as usize)
        .checked_mul(image.height as usize)?
        .checked_mul(4)?;
    if image.width == 0 || image.height == 0 || image.pixels.len() != expected {
        None
    } else {
        Some(image)
    }
}

/// Decode a PNG into straight RGBA.
pub(crate) fn decode_png(bytes: &[u8]) -> Option<RgbaIcon> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).ok()?;
    let rgba = image.to_rgba8();
    Some(RgbaIcon {
        width: rgba.width(),
        height: rgba.height(),
        pixels: rgba.into_raw(),
    })
}

/// Pixel edge for a `points`-tall icon on this monitor.
pub fn pixel_size(ctx: &egui::Context, points: f32) -> u32 {
    (ctx.pixels_per_point() * points).round().max(1.0) as u32
}

/// Draw the shell icon. Returns false when the caller should paint Lucide instead.
///
/// The texture is not tinted: a selected row keeps the colored shell picture.
pub fn paint(ui: &mut egui::Ui, cache: &mut IconCache, key: &IconKey, points: f32) -> bool {
    let pixels = pixel_size(ui.ctx(), points);
    let Some(texture) = cache.texture(ui.ctx(), key, pixels) else {
        return false;
    };
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(points), Sense::empty());
    ui.painter().image(
        texture,
        rect,
        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );
    true
}

/// Shell picture for one file-list row.
pub fn row_icon_key(row: &Row) -> IconKey {
    match row.kind {
        RowKind::Parent | RowKind::Directory => IconKey::Folder,
        RowKind::File | RowKind::Archive | RowKind::Symlink => {
            if let Target::Path(path) = &row.target
                && path_specific(path)
            {
                return IconKey::Path(path.clone());
            }
            IconKey::Extension(extension_of(&row.name))
        }
    }
}

fn path_specific(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("lnk"))
}

fn extension_of(name: &str) -> String {
    let file = name.rsplit(['/', '\\']).next().unwrap_or(name);
    match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explorer::{Row, RowKind, Target};
    use std::path::PathBuf;

    fn row(name: &str, kind: RowKind, target: Target) -> Row {
        Row {
            name: name.to_string(),
            kind,
            size: None,
            packed: None,
            modified: None,
            crc32: None,
            type_label: String::new(),
            key: name.to_string(),
            target,
        }
    }

    #[test]
    fn icon_keys_follow_extension_path_and_folder() {
        let cases = [
            (
                row(
                    "notes.pdf",
                    RowKind::File,
                    Target::ArchiveMember("docs/notes.pdf".to_string()),
                ),
                IconKey::Extension("pdf".to_string()),
            ),
            (
                row(
                    "tool.EXE",
                    RowKind::File,
                    Target::Path(PathBuf::from("tool.EXE")),
                ),
                IconKey::Path(PathBuf::from("tool.EXE")),
            ),
            (
                row(
                    "setup.exe",
                    RowKind::Archive,
                    Target::ArchiveMember("setup.exe".to_string()),
                ),
                IconKey::Extension("exe".to_string()),
            ),
            (
                row(
                    "src",
                    RowKind::Directory,
                    Target::ArchiveDir("src".to_string()),
                ),
                IconKey::Folder,
            ),
            (row("..", RowKind::Parent, Target::Parent), IconKey::Folder),
            (
                row(
                    "README",
                    RowKind::File,
                    Target::ArchiveMember("README".to_string()),
                ),
                IconKey::Extension(String::new()),
            ),
            (
                row(
                    "link.lnk",
                    RowKind::Symlink,
                    Target::Path(PathBuf::from("link.lnk")),
                ),
                IconKey::Path(PathBuf::from("link.lnk")),
            ),
        ];
        for (row, want) in cases {
            assert_eq!(row_icon_key(&row), want, "name {}", row.name);
        }
    }

    #[test]
    fn caches_a_raster_from_the_provider() {
        let mut cache = IconCache::spawn(|key, pixels| {
            assert_eq!(pixels, 16);
            match key {
                IconKey::Extension(ext) if ext == "pdf" => Some(RgbaIcon {
                    width: 1,
                    height: 1,
                    pixels: vec![1, 2, 3, 255],
                }),
                _ => None,
            }
        });
        let ctx = egui::Context::default();
        let key = IconKey::Extension("pdf".to_string());
        assert!(cache.texture(&ctx, &key, 16).is_none());
        let start = std::time::Instant::now();
        let mut ready = false;
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if cache.texture(&ctx, &key, 16).is_some() {
                ready = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(ready);
        assert!(!cache.is_pending());
    }

    #[test]
    fn remembers_a_failed_lookup() {
        let mut cache = IconCache::spawn(|_, _| None);
        let ctx = egui::Context::default();
        let key = IconKey::Folder;
        assert!(cache.texture(&ctx, &key, 16).is_none());
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) && cache.is_pending() {
            let _ = cache.texture(&ctx, &key, 16);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(cache.texture(&ctx, &key, 16).is_none());
        assert!(!cache.is_pending());
        assert!(cache.failed.contains(&(key, 16)));
    }
}
