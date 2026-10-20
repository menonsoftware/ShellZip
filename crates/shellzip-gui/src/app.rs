//! ShellZip window. Buttons call the same core operations as `szip`.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use shellzip_core::{
    CompressOpts, Entry, ExtractOpts, Overwrite, ZipMethod, add, extract, format_bytes, list, test,
};

use crate::panels;
use crate::theme::{self, INK};
use crate::worker::{self, Outcome, SharedProgress};

/// Desktop state for Milestone 1: open, list, extract, test, and create.
pub struct ShellZipApp {
    archive: Option<PathBuf>,
    format: String,
    entries: Vec<Entry>,
    selected: HashSet<String>,
    status: String,
    error: Option<String>,
    busy: bool,
    overwrite: Overwrite,
    level: u8,
    method: ZipMethod,
    flat: bool,
    progress: Arc<SharedProgress>,
    tx: Sender<Outcome>,
    rx: Receiver<Outcome>,
}

impl ShellZipApp {
    /// Install image loaders and the zinc theme.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        Self {
            archive: None,
            format: String::new(),
            entries: Vec::new(),
            selected: HashSet::new(),
            status: String::new(),
            error: None,
            busy: false,
            overwrite: Overwrite::Never,
            level: 3,
            method: ZipMethod::Deflate,
            flat: false,
            progress: Arc::new(SharedProgress::new()),
            tx,
            rx,
        }
    }

    fn open_archive(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Archives", &["zip", "tar", "gz", "tgz", "zst", "tzst"])
            .pick_file()
        else {
            return;
        };
        self.start_list(path);
    }

    fn start_list(&mut self, path: PathBuf) {
        if self.busy {
            return;
        }
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let (opened, entries) = list(&path)?;
            Ok(Outcome::Listed {
                format: opened.detected().label(),
                path,
                entries,
            })
        });
    }

    fn extract_archive(&mut self) {
        let Some(archive) = self.archive.clone() else {
            self.error = Some("Open an archive first.".to_string());
            return;
        };
        if self.busy {
            return;
        }
        let Some(output) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let names: Vec<String> = self.selected.iter().cloned().collect();
        let flat = self.flat;
        let overwrite = self.overwrite;
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let mut opts = ExtractOpts::new(output);
            opts.flat = flat;
            opts.overwrite = overwrite;
            if !names.is_empty() {
                opts.exact_paths = Some(names);
            }
            let report = extract(&archive, &opts, None, progress.as_ref())?;
            Ok(Outcome::Finished(format!(
                "extracted {}, skipped {}",
                report.extracted, report.skipped
            )))
        });
    }

    fn test_archive(&mut self) {
        let Some(archive) = self.archive.clone() else {
            self.error = Some("Open an archive first.".to_string());
            return;
        };
        if self.busy {
            return;
        }
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let report = test(&archive, progress.as_ref())?;
            let code = report.exit_code();
            if code == 0 {
                Ok(Outcome::Finished(format!("OK, {} entries", report.entries)))
            } else if !report.checksums.is_empty() {
                Err(shellzip_core::Error::BadChecksum {
                    entry: report.checksums.join(", "),
                })
            } else {
                Err(shellzip_core::Error::Encrypted(report.encrypted.join(", ")))
            }
        });
    }

    fn create_archive(&mut self) {
        if self.busy {
            return;
        }
        let Some(files) = rfd::FileDialog::new().pick_files() else {
            return;
        };
        if files.is_empty() {
            return;
        }
        let Some(dest) = rfd::FileDialog::new()
            .add_filter("ZIP", &["zip"])
            .add_filter("tar", &["tar"])
            .add_filter("tar.gz", &["tar.gz", "tgz"])
            .add_filter("tar.zst", &["tar.zst", "tzst"])
            .add_filter("gzip", &["gz"])
            .add_filter("zstd", &["zst"])
            .save_file()
        else {
            return;
        };
        let level = self.level;
        let method = self.method;
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let opts = CompressOpts {
                level,
                zip_method: method,
                force: true,
                ..CompressOpts::default()
            };
            let report = add(&dest, &files, &opts, progress.as_ref())?;
            match list(&dest) {
                Ok((opened, entries)) => Ok(Outcome::Listed {
                    path: dest,
                    format: opened.detected().label(),
                    entries,
                }),
                Err(err) => Ok(Outcome::Finished(format!(
                    "created {} ({}); listing failed: {err}",
                    report.path.display(),
                    format_bytes(report.packed_bytes)
                ))),
            }
        });
    }

    fn begin_job(&mut self) {
        self.busy = true;
        self.error = None;
        self.status.clear();
        self.progress.reset();
    }

    fn poll(&mut self) {
        while let Ok(outcome) = self.rx.try_recv() {
            self.busy = false;
            match outcome {
                Outcome::Listed {
                    path,
                    format,
                    entries,
                } => {
                    self.status = format!("opened {}", path.display());
                    self.archive = Some(path);
                    self.format = format;
                    self.entries = entries;
                    self.selected.clear();
                }
                Outcome::Finished(summary) => self.status = summary,
                Outcome::Failed(message) => self.error = Some(message),
            }
        }
    }

    fn take_drop(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        if let Some(path) = dropped.into_iter().next() {
            self.start_list(path);
        }
    }
}

impl eframe::App for ShellZipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        if self.busy {
            ctx.request_repaint();
        }
        self.take_drop(ctx);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading(egui::RichText::new("ShellZip").color(INK));
                ui.separator();
                ui.add_enabled_ui(!self.busy, |ui| {
                    if panels::tool(ui, crate::icons::Icon::FolderOpen, "Open") {
                        self.open_archive();
                    }
                    if panels::tool(ui, crate::icons::Icon::FolderDown, "Extract") {
                        self.extract_archive();
                    }
                    if panels::tool(ui, crate::icons::Icon::CircleCheck, "Test") {
                        self.test_archive();
                    }
                    if panels::tool(ui, crate::icons::Icon::Plus, "New") {
                        self.create_archive();
                    }
                });
                ui.separator();
                ui.add(egui::Slider::new(&mut self.level, 0..=5).text("level"));
                egui::ComboBox::from_id_salt("method")
                    .selected_text(match self.method {
                        ZipMethod::Deflate => "deflate",
                        ZipMethod::Zstd => "zstd",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.method, ZipMethod::Deflate, "deflate");
                        ui.selectable_value(&mut self.method, ZipMethod::Zstd, "zstd");
                    });
                egui::ComboBox::from_id_salt("overwrite")
                    .selected_text(overwrite_label(self.overwrite))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.overwrite, Overwrite::Never, "skip existing");
                        ui.selectable_value(
                            &mut self.overwrite,
                            Overwrite::Always,
                            "replace existing",
                        );
                        ui.selectable_value(&mut self.overwrite, Overwrite::Rename, "rename");
                    });
                ui.checkbox(&mut self.flat, "Flat extract");
            });
            if let Some(path) = &self.archive {
                ui.label(
                    egui::RichText::new(path.display().to_string()).color(crate::theme::MUTED),
                );
            }
            ui.add_space(6.0);
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(4.0);
            panels::status(
                ui,
                &self.format,
                self.entries.len(),
                self.selected.len(),
                self.busy,
                &self.status,
                self.progress.snapshot(),
            );
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(message) = &self.error {
                if panels::error_banner(ui, message) {
                    self.error = None;
                }
                ui.separator();
            }
            if self.entries.is_empty() {
                panels::empty_state(ui);
            } else {
                panels::file_list(ui, &self.entries, &mut self.selected);
            }
        });
    }
}

fn overwrite_label(mode: Overwrite) -> &'static str {
    match mode {
        Overwrite::Never => "skip existing",
        Overwrite::Always => "replace existing",
        Overwrite::Rename => "rename",
        Overwrite::Ask => "skip existing",
    }
}
