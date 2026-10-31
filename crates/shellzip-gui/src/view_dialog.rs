//! Read-only preview of one disk file or one archive member.
//!
//! The worker reads at most 1 MiB. The window only paints the result.

use std::io::{Read, Write};
use std::path::Path;

use shellzip_core::{Error, Password};

use crate::chrome;
use crate::icons::Icon;
use crate::theme;

/// Preview stops after this many bytes.
pub(crate) const PREVIEW_CAP: usize = 1024 * 1024;

/// Bytes the viewer can show.
pub(crate) struct PreviewData {
    /// Leading bytes, never longer than the cap.
    pub bytes: Vec<u8>,
    /// The cap stopped the read before the end.
    pub truncated: bool,
    /// Full size when it is known, otherwise the bytes kept.
    pub size: u64,
}

/// What the viewer paints. Invalid UTF-8 is not shown as text.
pub(crate) enum PreviewBody {
    /// The bytes are valid UTF-8.
    Text(String),
    /// The bytes are not text.
    Binary,
}

/// Centered viewer. `true` from [`ViewDialog::show`] means the user closed it.
pub(crate) struct ViewDialog {
    name: String,
    body: PreviewBody,
    truncated: bool,
    size: u64,
}

impl ViewDialog {
    /// Paint `body` for `name`. Classification already happened on the worker.
    pub(crate) fn from_parts(
        name: impl Into<String>,
        body: PreviewBody,
        truncated: bool,
        size: u64,
    ) -> Self {
        Self {
            name: name.into(),
            body,
            truncated,
            size,
        }
    }

    /// Paint the viewer. `true` means Close.
    pub(crate) fn show(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        let mut close = false;
        let title = format!("View {}", self.name);
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(480.0);
                ui.set_min_height(220.0);
                match &self.body {
                    PreviewBody::Text(text) => {
                        egui::ScrollArea::vertical()
                            .id_salt("shellzip-view-text")
                            .max_height(360.0)
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(text).monospace());
                            });
                    }
                    PreviewBody::Binary => {
                        ui.label(format!("Size: {}", chrome::format_grouped(self.size)));
                        ui.label("This file is not text.");
                    }
                }
                if self.truncated {
                    ui.add_space(6.0);
                    ui.label("The preview was truncated.");
                }
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if chrome::labeled_button(
                            ui,
                            Icon::X,
                            theme::palette(ui.ctx()).ink,
                            "Close",
                        )
                        .clicked()
                        {
                            close = true;
                        }
                    });
                });
            });
        close || !open
    }
}

/// Read a disk file, stopping after `cap` bytes.
pub(crate) fn read_disk(
    path: &Path,
    known_size: Option<u64>,
) -> shellzip_core::Result<PreviewData> {
    read_limited(path, PREVIEW_CAP, known_size)
}

/// Write one archive member into memory, stopping after 1 MiB.
///
/// `name` is the member path. Nothing is written to disk.
pub(crate) fn read_member(
    path: &Path,
    name: &str,
    password: Option<&Password>,
    known_size: Option<u64>,
) -> shellzip_core::Result<PreviewData> {
    let mut writer = CapWriter::new(PREVIEW_CAP);
    let result = shellzip_core::write_member(path, Some(name), &mut writer, password);
    if writer.truncated {
        return Ok(PreviewData {
            size: known_size.unwrap_or(bytes_len(&writer.buf)),
            bytes: writer.buf,
            truncated: true,
        });
    }
    let written = result?;
    Ok(PreviewData {
        bytes: writer.buf,
        truncated: false,
        size: known_size.unwrap_or(written),
    })
}

/// Valid UTF-8 becomes text. A cap that splits a final character still counts as text.
pub(crate) fn classify(bytes: &[u8], truncated: bool) -> PreviewBody {
    match std::str::from_utf8(bytes) {
        Ok(text) => PreviewBody::Text(text.to_string()),
        Err(err) if truncated && err.error_len().is_none() => {
            let valid = err.valid_up_to();
            let Some(prefix) = bytes.get(..valid) else {
                return PreviewBody::Binary;
            };
            match std::str::from_utf8(prefix) {
                Ok(text) => PreviewBody::Text(text.to_string()),
                Err(_) => PreviewBody::Binary,
            }
        }
        Err(_) => PreviewBody::Binary,
    }
}

fn read_limited(
    path: &Path,
    cap: usize,
    known_size: Option<u64>,
) -> shellzip_core::Result<PreviewData> {
    let file = std::fs::File::open(path)
        .map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let size = known_size
        .or_else(|| file.metadata().ok().map(|meta| meta.len()))
        .unwrap_or(0);
    let mut file = std::io::BufReader::new(file);
    let mut buf = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    while buf.len() < cap {
        let room = cap - buf.len();
        let want = room.min(chunk.len());
        let Some(dest) = chunk.get_mut(..want) else {
            break;
        };
        let read = file
            .read(dest)
            .map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        if read == 0 {
            break;
        }
        let Some(piece) = chunk.get(..read) else {
            break;
        };
        buf.extend_from_slice(piece);
    }
    let mut truncated = false;
    if buf.len() == cap {
        let mut extra = [0u8; 1];
        let read = file
            .read(&mut extra)
            .map_err(|err| Error::io(format!("read {}", path.display()), err))?;
        truncated = read > 0;
    }
    Ok(PreviewData {
        bytes: buf,
        truncated,
        size,
    })
}

fn bytes_len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

struct CapWriter {
    buf: Vec<u8>,
    limit: usize,
    truncated: bool,
}

impl CapWriter {
    fn new(limit: usize) -> Self {
        Self {
            buf: Vec::new(),
            limit,
            truncated: false,
        }
    }
}

impl Write for CapWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        if data.is_empty() {
            return Ok(0);
        }
        if self.buf.len() >= self.limit {
            self.truncated = true;
            return Err(std::io::Error::new(
                std::io::ErrorKind::QuotaExceeded,
                "preview cap",
            ));
        }
        let room = self.limit - self.buf.len();
        let take = data.len().min(room);
        if let Some(piece) = data.get(..take) {
            self.buf.extend_from_slice(piece);
        }
        if take < data.len() {
            self.truncated = true;
            return Err(std::io::Error::new(
                std::io::ErrorKind::QuotaExceeded,
                "preview cap",
            ));
        }
        Ok(take)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_accepts_utf8_and_rejects_binary() {
        assert!(matches!(classify(b"hello", false), PreviewBody::Text(_)));
        assert!(matches!(classify(b"", false), PreviewBody::Text(_)));
        assert!(matches!(classify(&[0xff, 0xfe], true), PreviewBody::Binary));
        assert!(matches!(
            classify(&[b'a', 0xC2], true),
            PreviewBody::Text(_)
        ));
        assert!(matches!(
            classify(&[b'a', 0xC2], false),
            PreviewBody::Binary
        ));
    }

    #[test]
    fn cap_writer_stops_at_the_limit() {
        let mut writer = CapWriter::new(4);
        assert_eq!(writer.write(b"ab").unwrap(), 2);
        let err = writer.write(b"cdef").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::QuotaExceeded);
        assert!(writer.truncated);
        assert_eq!(writer.buf, b"abcd");
    }

    #[test]
    fn read_disk_stops_at_the_cap() {
        let dir = std::env::temp_dir().join(format!(
            "shellzip-preview-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _guard = Scratch(dir.clone());
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("note.txt");
        std::fs::write(&path, b"abcdef").unwrap();
        let preview = read_limited(&path, 4, None).unwrap();
        assert_eq!(preview.bytes, b"abcd");
        assert!(preview.truncated);
        assert_eq!(preview.size, 6);
        let whole = read_limited(&path, 10, None).unwrap();
        assert_eq!(whole.bytes, b"abcdef");
        assert!(!whole.truncated);
        assert_eq!(whole.size, 6);
    }

    struct Scratch(std::path::PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
