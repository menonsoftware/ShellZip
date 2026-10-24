//! ShellZip window. Buttons call the same core operations as `szip`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use shellzip_core::{
    CompressOpts, Entry, ExtractOpts, Overwrite, ZipMethod, add, extract, format_bytes, list, test,
};

use crate::chrome::{self, AddressEvent, MenuCommand, PackSettings, ToolCommand};
use crate::explorer::{
    ArchiveIndex, Explorer, JumpTarget, Row, RowKind, Target, archive_prefix, folder_parent,
    is_supported_archive, jumps, location_text, path_under, read_directory, sort_rows, totals,
    totals_label,
};
use crate::theme::{self, CANVAS};
use crate::worker::{self, Outcome, SharedProgress};

/// Where the file pane is looking.
enum Place {
    /// A real directory. `rows` does not include `..`.
    Folder { path: PathBuf, rows: Vec<Row> },
    /// An open archive, browsed one folder at a time.
    Archive {
        path: PathBuf,
        format: String,
        index: Box<ArchiveIndex>,
        members: Vec<String>,
        prefix: String,
    },
}

/// Desktop window: filesystem browser, and the same browser inside an archive.
pub struct ShellZipApp {
    place: Place,
    view: Vec<Row>,
    explorer: Explorer,
    address: String,
    scroll_to: Option<usize>,
    pending_select: Option<String>,
    status: String,
    error: Option<String>,
    busy: bool,
    loaded: bool,
    about: bool,
    options: PackSettings,
    generation: u64,
    progress: Arc<SharedProgress>,
    tx: Sender<Outcome>,
    rx: Receiver<Outcome>,
}

impl ShellZipApp {
    /// Install image loaders, apply the file-manager theme, and list the desktop.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        let path = starting_folder();
        let generation = 1;
        spawn_directory(tx.clone(), path.clone(), generation);
        let mut app = Self {
            place: Place::Folder {
                path: path.clone(),
                rows: Vec::new(),
            },
            view: Vec::new(),
            explorer: Explorer::new(),
            address: path.display().to_string(),
            scroll_to: None,
            pending_select: None,
            status: String::new(),
            error: None,
            busy: false,
            loaded: false,
            about: false,
            options: PackSettings {
                level: 3,
                method: ZipMethod::Deflate,
                overwrite: Overwrite::Never,
                flat: false,
            },
            generation,
            progress: Arc::new(SharedProgress::new()),
            tx,
            rx,
        };
        app.rebuild_view();
        app
    }

    fn open_archive(&mut self) {
        if self.busy {
            return;
        }
        let selected = self.selected_archives();
        if let [path] = selected.as_slice() {
            self.start_list(path.clone());
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Archives",
                &["zip", "tar", "gz", "tgz", "zst", "tzst", "rar", "7z"],
            )
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
        let generation = self.next_generation();
        self.begin_job();
        self.status = format!("Opening {}", path.display());
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let (opened, entries) = list(&path)?;
            Ok(Outcome::Listed {
                generation,
                format: opened.detected().label(),
                path,
                entries,
            })
        });
    }

    fn extract_archive(&mut self) {
        let Some(archive) = self.command_archive() else {
            self.error = Some("Open an archive, or select one.".to_string());
            return;
        };
        if self.busy {
            return;
        }
        let Some(output) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let names = self.member_filter(&archive);
        let flat = self.options.flat;
        let overwrite = self.options.overwrite;
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let mut opts = ExtractOpts::new(output);
            opts.flat = flat;
            opts.overwrite = overwrite;
            if names.is_some() {
                opts.exact_paths = names;
            }
            let report = extract(&archive, &opts, None, progress.as_ref())?;
            Ok(Outcome::Finished(format!(
                "extracted {}, skipped {}",
                report.extracted, report.skipped
            )))
        });
    }

    fn test_archive(&mut self) {
        let Some(archive) = self.command_archive() else {
            self.error = Some("Open an archive, or select one.".to_string());
            return;
        };
        if self.busy {
            return;
        }
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let report = test(&archive, None, progress.as_ref())?;
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
        let mut files = self.selected_inputs();
        if files.is_empty() {
            let Some(picked) = rfd::FileDialog::new().pick_files() else {
                return;
            };
            files = picked;
        }
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
        let level = self.options.level;
        let method = self.options.method;
        // Adding into an existing ZIP keeps its other files. `-y` still replaces from the CLI.
        let update_zip = existing_zip(&dest);
        let generation = self.next_generation();
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        worker::spawn(tx, move || {
            let opts = CompressOpts {
                level,
                zip_method: method,
                force: !update_zip,
                ..CompressOpts::default()
            };
            let report = add(&dest, &files, &opts, progress.as_ref())?;
            match list(&dest) {
                Ok((opened, entries)) => Ok(Outcome::Listed {
                    generation,
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

    fn next_generation(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.generation
    }

    fn poll(&mut self) {
        while let Ok(outcome) = self.rx.try_recv() {
            match outcome {
                Outcome::Directory {
                    generation,
                    path,
                    rows,
                    error,
                } => self.apply_directory(generation, path, rows, error),
                Outcome::Listed {
                    generation,
                    path,
                    format,
                    entries,
                } => {
                    self.busy = false;
                    if generation == self.generation {
                        self.apply_archive(path, format, entries);
                    }
                }
                Outcome::Finished(summary) => {
                    self.busy = false;
                    self.status = summary;
                }
                Outcome::Failed(message) => {
                    self.busy = false;
                    self.status.clear();
                    self.error = Some(message);
                }
            }
        }
    }

    fn apply_directory(
        &mut self,
        generation: u64,
        path: PathBuf,
        rows: Vec<Row>,
        error: Option<String>,
    ) {
        if generation != self.generation {
            return;
        }
        self.place = Place::Folder { path, rows };
        self.loaded = true;
        self.explorer.clear_selection();
        self.rebuild_view();
        if let Some(key) = self.pending_select.take()
            && self.view.iter().any(|row| row.key == key)
        {
            self.scroll_to = self.view.iter().position(|row| row.key == key);
            self.explorer.select_only(&key);
        }
        match error {
            Some(message) => {
                self.error = Some(message);
                self.status.clear();
            }
            None => self.status.clear(),
        }
    }

    fn apply_archive(&mut self, path: PathBuf, format: String, entries: Vec<Entry>) {
        let members = entries.iter().map(|entry| entry.path.clone()).collect();
        let index = Box::new(ArchiveIndex::build(&entries));
        self.address = location_text(&path, Some(""));
        self.place = Place::Archive {
            path,
            format,
            index,
            members,
            prefix: String::new(),
        };
        self.loaded = true;
        self.pending_select = None;
        self.explorer.clear_selection();
        self.rebuild_view();
        self.error = None;
        self.status = match &self.place {
            Place::Archive { path, .. } => format!("Opened {}", path.display()),
            Place::Folder { .. } => String::new(),
        };
    }

    fn open_folder(&mut self, path: PathBuf) {
        self.open_folder_inner(path, true);
    }

    fn open_folder_inner(&mut self, path: PathBuf, clear_pending: bool) {
        if self.busy {
            return;
        }
        if clear_pending {
            self.pending_select = None;
        }
        let generation = self.next_generation();
        self.address = path.display().to_string();
        self.place = Place::Folder {
            path: path.clone(),
            rows: Vec::new(),
        };
        self.loaded = false;
        self.error = None;
        self.status.clear();
        self.explorer.clear_selection();
        self.rebuild_view();
        spawn_directory(self.tx.clone(), path, generation);
    }

    fn set_prefix(&mut self, prefix: String) {
        let allowed = match &self.place {
            Place::Archive { index, .. } => prefix.is_empty() || index.contains_folder(&prefix),
            Place::Folder { .. } => false,
        };
        if !allowed {
            self.error = Some("That folder is not in the archive.".to_string());
            return;
        }
        let address = match &self.place {
            Place::Archive { path, .. } => location_text(path, Some(&prefix)),
            Place::Folder { .. } => return,
        };
        if let Place::Archive {
            prefix: slot, path, ..
        } = &mut self.place
        {
            *slot = prefix;
            if slot.is_empty() {
                self.status = format!("Opened {}", path.display());
            } else {
                self.status.clear();
            }
        }
        self.address = address;
        self.error = None;
        self.explorer.clear_selection();
        self.rebuild_view();
    }

    fn go_up(&mut self) {
        if self.busy {
            return;
        }
        enum Step {
            Folder(PathBuf),
            Prefix(String),
            Leave { parent: PathBuf, key: String },
        }
        let step = match &self.place {
            Place::Folder { path, .. } => {
                let Some(parent) = folder_parent(path).map(Path::to_path_buf) else {
                    return;
                };
                Step::Folder(parent)
            }
            Place::Archive { path, prefix, .. } => {
                if prefix.is_empty() {
                    let Some(parent) = folder_parent(path).map(Path::to_path_buf) else {
                        return;
                    };
                    Step::Leave {
                        parent,
                        key: path.display().to_string(),
                    }
                } else {
                    let parent = prefix
                        .rsplit_once('/')
                        .map(|(head, _)| head.to_string())
                        .unwrap_or_default();
                    Step::Prefix(parent)
                }
            }
        };
        match step {
            Step::Folder(path) => self.open_folder(path),
            Step::Prefix(prefix) => self.set_prefix(prefix),
            Step::Leave { parent, key } => {
                self.pending_select = Some(key);
                self.open_folder_inner(parent, false);
            }
        }
    }

    fn open_address(&mut self) {
        if self.busy {
            return;
        }
        let typed = self.address.trim().to_string();
        if typed.is_empty() {
            return;
        }
        let archive = match &self.place {
            Place::Archive { path, .. } => Some(path.clone()),
            Place::Folder { .. } => None,
        };
        if let Some(archive) = archive
            && let Some(prefix) = archive_prefix(&typed, &archive)
        {
            self.set_prefix(prefix);
            return;
        }
        let path = PathBuf::from(&typed);
        if path.is_dir() {
            self.open_folder(path);
        } else if path.is_file() && is_supported_archive(&path) {
            self.start_list(path);
        } else {
            self.error = Some(format!("Cannot open {typed}"));
        }
    }

    fn activate(&mut self, index: usize) {
        if self.busy {
            return;
        }
        let Some(row) = self.view.get(index).cloned() else {
            return;
        };
        match row.target {
            Target::Parent => self.go_up(),
            Target::Path(path) => match row.kind {
                RowKind::Directory => self.open_folder(path),
                RowKind::Archive => self.start_list(path),
                RowKind::File | RowKind::Symlink | RowKind::Parent => {}
            },
            Target::ArchiveDir(prefix) => self.set_prefix(prefix),
            Target::ArchiveMember(_) => {}
        }
    }

    fn activate_selected(&mut self) {
        let key = self.explorer.anchor().map(str::to_string);
        let Some(key) = key else {
            return;
        };
        let Some(index) = self.view.iter().position(|row| row.key == key) else {
            return;
        };
        self.activate(index);
    }

    fn rebuild_view(&mut self) {
        self.view = match &self.place {
            Place::Folder { rows, .. } => rows.clone(),
            Place::Archive { index, prefix, .. } => index.children(prefix).to_vec(),
        };
        if self.can_go_up() {
            self.view.insert(0, Row::parent());
        }
        let (column, order) = self.explorer.ordering();
        sort_rows(&mut self.view, column, order);
    }

    fn can_go_up(&self) -> bool {
        match &self.place {
            Place::Folder { path, .. } => folder_parent(path).is_some(),
            Place::Archive { .. } => true,
        }
    }

    fn archive_ready(&self) -> bool {
        match &self.place {
            Place::Archive { .. } => true,
            Place::Folder { .. } => self
                .view
                .iter()
                .any(|row| row.kind == RowKind::Archive && self.explorer.is_selected(&row.key)),
        }
    }

    fn command_archive(&self) -> Option<PathBuf> {
        match &self.place {
            Place::Archive { path, .. } => Some(path.clone()),
            Place::Folder { .. } => self.selected_archives().into_iter().next(),
        }
    }

    fn selected_archives(&self) -> Vec<PathBuf> {
        let Place::Folder { .. } = &self.place else {
            return Vec::new();
        };
        self.view
            .iter()
            .filter_map(|row| match &row.target {
                Target::Path(path)
                    if row.kind == RowKind::Archive && self.explorer.is_selected(&row.key) =>
                {
                    Some(path.clone())
                }
                _ => None,
            })
            .collect()
    }

    fn selected_inputs(&self) -> Vec<PathBuf> {
        let Place::Folder { .. } = &self.place else {
            return Vec::new();
        };
        self.view
            .iter()
            .filter_map(|row| {
                if row.kind == RowKind::Parent || !self.explorer.is_selected(&row.key) {
                    return None;
                }
                match &row.target {
                    Target::Path(path) => Some(path.clone()),
                    _ => None,
                }
            })
            .collect()
    }

    fn member_filter(&self, archive: &Path) -> Option<Vec<String>> {
        let Place::Archive { path, members, .. } = &self.place else {
            return None;
        };
        if path != archive {
            return None;
        }
        let chosen: Vec<&Row> = self
            .view
            .iter()
            .filter(|row| row.kind != RowKind::Parent && self.explorer.is_selected(&row.key))
            .collect();
        if chosen.is_empty() {
            return None;
        }
        let mut paths = Vec::new();
        for row in chosen {
            match &row.target {
                Target::ArchiveMember(member) => paths.push(member.clone()),
                Target::ArchiveDir(prefix) => {
                    for member in members {
                        if path_under(member, prefix) {
                            paths.push(member.clone());
                        }
                    }
                }
                Target::Parent | Target::Path(_) => {}
            }
        }
        Some(paths)
    }

    fn address_jumps(&self) -> Vec<crate::explorer::Jump> {
        match &self.place {
            Place::Folder { path, .. } => jumps(path, None),
            Place::Archive { path, prefix, .. } => jumps(path, Some(prefix)),
        }
    }

    fn summary(&self) -> String {
        if self.loaded {
            let (folders, files, bytes) = totals(&self.view);
            totals_label(folders, files, bytes)
        } else {
            "Reading...".to_string()
        }
    }

    fn note(&self) -> &str {
        if !self.status.is_empty() {
            self.status.as_str()
        } else {
            match &self.place {
                Place::Archive { format, .. } => format.as_str(),
                Place::Folder { .. } => "",
            }
        }
    }

    fn title(&self) -> String {
        match &self.place {
            Place::Folder { path, .. } => folder_title(path),
            Place::Archive { path, prefix, .. } => {
                if prefix.is_empty() {
                    folder_title(path)
                } else {
                    prefix.rsplit('/').next().unwrap_or("ShellZip").to_string()
                }
            }
        }
    }

    fn on_menu(&mut self, command: MenuCommand) {
        match command {
            MenuCommand::Open => self.open_archive(),
            MenuCommand::Add => self.create_archive(),
            MenuCommand::Extract => self.extract_archive(),
            MenuCommand::Test => self.test_archive(),
            MenuCommand::SelectAll => self.explorer.select_all(&self.view),
            MenuCommand::SelectNone => self.explorer.clear_selection(),
            MenuCommand::About => self.about = true,
            MenuCommand::Exit => {}
        }
    }

    fn on_tool(&mut self, command: ToolCommand) {
        match command {
            ToolCommand::Add => self.create_archive(),
            ToolCommand::Extract => self.extract_archive(),
            ToolCommand::Test => self.test_archive(),
            ToolCommand::Open => self.open_archive(),
        }
    }

    fn on_address(&mut self, event: AddressEvent) {
        match event {
            AddressEvent::Up => self.go_up(),
            AddressEvent::Submit => self.open_address(),
            AddressEvent::Jump(JumpTarget::Folder(path)) => self.open_folder(path),
            AddressEvent::Jump(JumpTarget::Prefix(prefix)) => self.set_prefix(prefix),
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let up = ctx.input_mut(|input| {
            input.consume_key(egui::Modifiers::NONE, egui::Key::Backspace)
                || input.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp)
        });
        if up {
            self.go_up();
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::A)) {
            self.explorer.select_all(&self.view);
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            self.activate_selected();
        }
    }

    fn take_drop(&mut self, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        let dropped: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });
        let Some(path) = dropped.into_iter().next() else {
            return;
        };
        if path.is_dir() {
            self.open_folder(path);
        } else if is_supported_archive(&path) {
            self.start_list(path);
        } else {
            self.error = Some(format!("{} is not a supported archive", path.display()));
        }
    }
}

impl eframe::App for ShellZipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        crate::icons::warm(ctx);
        if self.busy || !self.loaded {
            ctx.request_repaint();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));
        self.take_drop(ctx);

        let mut menu_command = None;
        egui::TopBottomPanel::top("menu")
            .frame(chrome::bar_frame(4, 0))
            .show(ctx, |ui| {
                menu_command = chrome::menu_bar(ui, &mut self.options);
            });
        if matches!(menu_command, Some(MenuCommand::Exit)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if let Some(command) = menu_command {
            self.on_menu(command);
        }

        let archive_ready = self.archive_ready();
        let busy = self.busy;
        let mut tool_command = None;
        egui::TopBottomPanel::top("toolbar")
            .frame(chrome::bar_frame(6, 2))
            .show(ctx, |ui| {
                tool_command = chrome::toolbar(ui, busy, archive_ready);
            });
        if let Some(command) = tool_command {
            self.on_tool(command);
        }

        let can_up = self.can_go_up() && !self.busy;
        let jump_list = self.address_jumps();
        let mut address_event = None;
        egui::TopBottomPanel::top("address")
            .frame(chrome::bar_frame(6, 4))
            .show(ctx, |ui| {
                address_event = chrome::address_bar(ui, &mut self.address, can_up, &jump_list);
            });
        let submitted = matches!(address_event, Some(AddressEvent::Submit));
        if let Some(event) = address_event {
            self.on_address(event);
        }

        let summary = self.summary();
        let note = self.note().to_string();
        let progress = self.progress.snapshot();
        let mut dismiss_error = false;
        egui::TopBottomPanel::bottom("status")
            .frame(chrome::bar_frame(10, 4))
            .show(ctx, |ui| {
                dismiss_error = chrome::status_bar(
                    ui,
                    &summary,
                    self.busy,
                    progress,
                    self.error.as_deref(),
                    &note,
                );
            });
        if dismiss_error {
            self.error = None;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(CANVAS))
            .show(ctx, |ui| {
                if let Some(index) = crate::explorer::show(
                    ui,
                    &mut self.explorer,
                    &mut self.view,
                    &mut self.scroll_to,
                ) {
                    self.activate(index);
                }
            });

        chrome::about(ctx, &mut self.about);
        if !submitted {
            self.shortcuts(ctx);
        }
    }
}

fn starting_folder() -> PathBuf {
    for key in ["USERPROFILE", "HOME"] {
        let Ok(home) = std::env::var(key) else {
            continue;
        };
        let desktop = PathBuf::from(&home).join("Desktop");
        if desktop.is_dir() {
            return desktop;
        }
        let home = PathBuf::from(home);
        if home.is_dir() {
            return home;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn existing_zip(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
}

fn folder_title(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

fn spawn_directory(tx: Sender<Outcome>, path: PathBuf, generation: u64) {
    worker::spawn(tx, move || {
        let (rows, error) = match read_directory(&path) {
            Ok(rows) => (rows, None),
            Err(message) => (Vec::new(), Some(message)),
        };
        Ok(Outcome::Directory {
            generation,
            path,
            rows,
            error,
        })
    });
}
