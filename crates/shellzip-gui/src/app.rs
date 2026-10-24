//! ShellZip window. Buttons call the same core operations as `szip`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use shellzip_core::{
    CompressOpts, ConflictChoice, Entry, Error, ExtractOpts, ListOpts, Password, UpdateMode, add,
    add_prompted, add_sfx, delete, extract, format_bytes, format_crc, list_with, test,
};

use crate::add_dialog::{self, AddAction, AddRequest, AskPrompt, AskReply};
use crate::chrome::{self, AddressEvent, MenuCommand, PackSettings, ToolCommand};
use crate::explorer::{
    ArchiveIndex, Explorer, JumpTarget, Row, RowKind, Target, archive_prefix, folder_parent,
    is_supported_archive, jumps, location_text, path_under, read_directory, sort_rows, totals,
    totals_label,
};
use crate::theme::{self, CANVAS};
use crate::worker::{self, Outcome, PasswordJob, PasswordRequest, SharedProgress};

/// Password dialog waiting on the window.
struct PasswordAsk {
    request: PasswordRequest,
    text: String,
}

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
    info: Option<String>,
    pending_delete: Option<Vec<String>>,
    add_dialog: Option<add_dialog::AddDialog>,
    ask: Option<AskPrompt>,
    password_ask: Option<PasswordAsk>,
    /// Password that opened the archive currently shown, if it needed one.
    opened_secret: Option<(PathBuf, Password)>,
    options: PackSettings,
    saved: PackSettings,
    generation: u64,
    progress: Arc<SharedProgress>,
    tx: Sender<Outcome>,
    rx: Receiver<Outcome>,
}

impl ShellZipApp {
    /// Install image loaders, apply the file-manager theme, and list `launch` or the desktop.
    pub fn new(cc: &eframe::CreationContext<'_>, launch: Option<PathBuf>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let (tx, rx) = mpsc::channel();
        let launch = launch.filter(|path| path.exists());
        let open_file = launch.as_ref().is_some_and(|path| path.is_file());
        let path = match &launch {
            Some(path) if path.is_dir() => path.clone(),
            Some(path) => path
                .parent()
                .filter(|parent| parent.is_dir())
                .map(Path::to_path_buf)
                .unwrap_or_else(starting_folder),
            None => starting_folder(),
        };
        let generation = 1;
        let options = crate::settings::load();
        if !open_file {
            spawn_directory(tx.clone(), path.clone(), generation);
        }
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
            info: None,
            pending_delete: None,
            add_dialog: None,
            ask: None,
            password_ask: None,
            opened_secret: None,
            options,
            saved: options,
            generation,
            progress: Arc::new(SharedProgress::new()),
            tx,
            rx,
        };
        app.rebuild_view();
        if let Some(path) = launch.filter(|path| path.is_file()) {
            app.start_list(path);
        }
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
                &[
                    "zip", "tar", "gz", "tgz", "zst", "tzst", "rar", "7z", "exe", "sfx",
                ],
            )
            .pick_file()
        else {
            return;
        };
        self.start_list(path);
    }

    fn start_list(&mut self, path: PathBuf) {
        let password = self.saved_password(&path);
        self.start_list_with(path, password);
    }

    fn start_list_with(&mut self, path: PathBuf, password: Option<Password>) {
        if self.busy {
            return;
        }
        let generation = self.next_generation();
        self.begin_job();
        self.status = format!("Opening {}", path.display());
        let tx = self.tx.clone();
        let again = password.is_some();
        worker::spawn(tx, move || match list_archive(&path, password.as_ref()) {
            Ok((opened, entries)) => Ok(Outcome::Listed {
                generation,
                format: opened.detected().label(),
                path,
                entries,
            }),
            Err(err) if needs_password(&err) => Ok(Outcome::NeedPassword(PasswordRequest {
                generation,
                path,
                job: PasswordJob::List,
                again,
            })),
            Err(err) => Err(err),
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
        let password = self.saved_password(&archive);
        self.spawn_extract(archive, output, flat, overwrite, names, password);
    }

    fn spawn_extract(
        &mut self,
        archive: PathBuf,
        output: PathBuf,
        flat: bool,
        overwrite: shellzip_core::Overwrite,
        names: Option<Vec<String>>,
        password: Option<Password>,
    ) {
        if self.busy {
            return;
        }
        let progress = Arc::clone(&self.progress);
        let generation = self.generation;
        self.begin_job();
        let tx = self.tx.clone();
        let again = password.is_some();
        worker::spawn(tx, move || {
            let mut opts = ExtractOpts::new(&output);
            opts.flat = flat;
            opts.overwrite = overwrite;
            opts.password = password;
            if names.is_some() {
                opts.exact_paths = names.clone();
            }
            match extract(&archive, &opts, None, progress.as_ref()) {
                Ok(report) => Ok(Outcome::Finished(format!(
                    "extracted {}, skipped {}",
                    report.extracted, report.skipped
                ))),
                Err(err) if needs_password(&err) => Ok(Outcome::NeedPassword(PasswordRequest {
                    generation,
                    path: archive,
                    job: PasswordJob::Extract {
                        output,
                        flat,
                        overwrite,
                        names,
                    },
                    again,
                })),
                Err(err) => Err(err),
            }
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
        let password = self.saved_password(&archive);
        self.spawn_test(archive, password);
    }

    fn spawn_test(&mut self, archive: PathBuf, password: Option<Password>) {
        if self.busy {
            return;
        }
        let progress = Arc::clone(&self.progress);
        let generation = self.generation;
        self.begin_job();
        let tx = self.tx.clone();
        let again = password.is_some();
        worker::spawn(tx, move || {
            let report = match test(&archive, password.as_ref(), progress.as_ref()) {
                Err(err) if needs_password(&err) => {
                    return Ok(Outcome::NeedPassword(PasswordRequest {
                        generation,
                        path: archive,
                        job: PasswordJob::Test,
                        again,
                    }));
                }
                other => other?,
            };
            let code = report.exit_code();
            if code == 0 {
                Ok(Outcome::Finished(format!("OK, {} entries", report.entries)))
            } else if !report.checksums.is_empty() {
                Err(shellzip_core::Error::BadChecksum {
                    entry: report.checksums.join(", "),
                })
            } else if !again && !report.encrypted.is_empty() {
                Ok(Outcome::NeedPassword(PasswordRequest {
                    generation,
                    path: archive,
                    job: PasswordJob::Test,
                    again: false,
                }))
            } else {
                Err(Error::Encrypted(report.encrypted.join(", ")))
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
        self.open_add_dialog(files);
    }

    fn open_add_dialog(&mut self, files: Vec<PathBuf>) {
        if self.busy || files.is_empty() || self.add_dialog.is_some() {
            return;
        }
        self.add_dialog = Some(add_dialog::AddDialog::prepare(
            &self.current_dir(),
            files,
            &self.options,
        ));
    }

    fn current_dir(&self) -> PathBuf {
        match &self.place {
            Place::Folder { path, .. } => path.clone(),
            Place::Archive { path, .. } => path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from(".")),
        }
    }

    fn start_add(&mut self, request: AddRequest) {
        if self.busy {
            return;
        }
        self.options.level = request.level.min(5);
        self.options.method = request.method;
        self.options.sevenz_method = request.sevenz.method;
        self.options.solid = request.sevenz.solid;
        let password = if request.password.is_empty() {
            None
        } else {
            match Password::new(request.password) {
                Ok(password) => Some(password),
                Err(err) => {
                    self.error = Some(err.to_string());
                    return;
                }
            }
        };
        let stub = if request.sfx {
            match find_stub() {
                Ok(path) => Some(path),
                Err(message) => {
                    self.error = Some(message);
                    return;
                }
            }
        } else {
            None
        };
        let merging = stub.is_none()
            && request.dest.is_file()
            && request.dest.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("7z")
            });
        let force = request.dest.exists() && !merging;
        let generation = self.next_generation();
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        let asking = request.update == UpdateMode::Ask && stub.is_none();
        if !asking {
            self.ask = None;
        }
        let prompt = asking.then(|| {
            let (names_tx, names_rx) = mpsc::channel();
            let (replies_tx, replies_rx) = mpsc::channel();
            self.ask = Some(AskPrompt {
                names: names_rx,
                replies: replies_tx,
                pending: None,
            });
            (names_tx, replies_rx)
        });
        worker::spawn(tx, move || {
            let listed = password.clone();
            let opts = CompressOpts {
                level: request.level.min(5),
                zip_method: request.method,
                force,
                update: request.update,
                comment: request.comment,
                password,
                sevenz: request.sevenz,
                ..CompressOpts::default()
            };
            let dest = request.dest;
            let files = request.inputs;
            let report = if let Some(stub) = stub {
                add_sfx(
                    &dest,
                    &stub,
                    &files,
                    &opts,
                    request.sfx_payload,
                    progress.as_ref(),
                )?
            } else if let Some((names_tx, replies_rx)) = prompt {
                let policy = std::sync::Mutex::new(None);
                let decide = move |name: &str| decide_member(name, &policy, &names_tx, &replies_rx);
                add_prompted(&dest, &files, &opts, progress.as_ref(), &decide)?
            } else {
                add(&dest, &files, &opts, progress.as_ref())?
            };
            match list_archive(&dest, listed.as_ref()) {
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

    fn request_delete(&mut self) {
        if self.busy {
            return;
        }
        let (format, path) = match &self.place {
            Place::Archive { format, path, .. } => (format.clone(), path.clone()),
            Place::Folder { .. } => {
                self.error = Some("Open a ZIP or 7z archive to delete members.".to_string());
                return;
            }
        };
        if format != "ZIP" && format != "7Z" {
            self.error = Some("Delete only rewrites ZIP and 7z archives.".to_string());
            return;
        }
        let Some(names) = self.member_filter(&path) else {
            self.error = Some("Select the members to delete.".to_string());
            return;
        };
        if names.is_empty() {
            self.error = Some("Select the members to delete.".to_string());
            return;
        }
        self.pending_delete = Some(names);
    }

    fn run_delete(&mut self) {
        let Some(names) = self.pending_delete.take() else {
            return;
        };
        let Place::Archive { path, .. } = &self.place else {
            return;
        };
        let archive = path.clone();
        let password = self.saved_password(&archive);
        self.spawn_delete(archive, names, password);
    }

    fn spawn_delete(&mut self, archive: PathBuf, names: Vec<String>, password: Option<Password>) {
        if self.busy {
            return;
        }
        let generation = self.next_generation();
        let progress = Arc::clone(&self.progress);
        self.begin_job();
        let tx = self.tx.clone();
        let again = password.is_some();
        let listed = password.clone();
        worker::spawn(tx, move || {
            let report = match delete(&archive, &names, password.as_ref(), progress.as_ref()) {
                Err(err) if needs_password(&err) => {
                    return Ok(Outcome::NeedPassword(PasswordRequest {
                        generation,
                        path: archive,
                        job: PasswordJob::Delete { names },
                        again,
                    }));
                }
                other => other?,
            };
            match list_archive(&archive, listed.as_ref()) {
                Ok((opened, entries)) => Ok(Outcome::Listed {
                    generation,
                    path: archive,
                    format: opened.detected().label(),
                    entries,
                }),
                Err(err) => Ok(Outcome::Finished(format!(
                    "deleted {} ({err})",
                    report.removed
                ))),
            }
        });
    }

    fn show_info(&mut self) {
        self.info = Some(self.info_text());
    }

    fn info_text(&self) -> String {
        let mut lines = Vec::new();
        match &self.place {
            Place::Folder { path, .. } => {
                lines.push(format!("Folder: {}", path.display()));
            }
            Place::Archive {
                path,
                format,
                members,
                ..
            } => {
                lines.push(format!("{format}: {}", path.display()));
                lines.push(format!("{} members", members.len()));
            }
        }
        for row in &self.view {
            if row.kind == RowKind::Parent || !self.explorer.is_selected(&row.key) {
                continue;
            }
            lines.push(String::new());
            lines.push(row.name.clone());
            if let Some(size) = row.size {
                lines.push(format!("Size: {}", format_bytes(size)));
            }
            if let Some(packed) = row.packed {
                lines.push(format!("Packed: {}", format_bytes(packed)));
            }
            if row.crc32.is_some() {
                lines.push(format!("CRC32: {}", format_crc(row.crc32)));
            }
        }
        lines.join("\n")
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
                    self.ask = None;
                    if generation == self.generation {
                        self.apply_archive(path, format, entries);
                    }
                }
                Outcome::Finished(summary) => {
                    self.busy = false;
                    self.ask = None;
                    self.status = summary;
                }
                Outcome::Failed(message) => {
                    self.busy = false;
                    self.ask = None;
                    self.status.clear();
                    self.error = Some(message);
                }
                Outcome::NeedPassword(request) => {
                    self.busy = false;
                    self.ask = None;
                    if request.generation == self.generation {
                        self.password_ask = Some(PasswordAsk {
                            request,
                            text: String::new(),
                        });
                    }
                }
            }
        }
    }

    fn saved_password(&self, path: &Path) -> Option<Password> {
        self.opened_secret
            .as_ref()
            .filter(|(saved, _)| saved == path)
            .map(|(_, password)| password.clone())
    }

    fn password_window(&mut self, ctx: &egui::Context) {
        let Some(ask) = self.password_ask.as_mut() else {
            return;
        };
        let mut open = true;
        let mut submit = false;
        let mut cancel = false;
        let prompt = if ask.request.again {
            "That password was incorrect. Enter it again."
        } else {
            "Enter the password to open this archive."
        };
        egui::Window::new("Password")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                ui.label(prompt);
                let field = ui.add(
                    egui::TextEdit::singleline(&mut ask.text)
                        .password(true)
                        .desired_width(f32::INFINITY)
                        .id_salt("shellzip-open-password"),
                );
                field.request_focus();
                if ui.button("OK").clicked()
                    || ui.input(|input| input.key_pressed(egui::Key::Enter))
                {
                    submit = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        if !open || cancel {
            self.password_ask = None;
            return;
        }
        if !submit {
            return;
        }
        let Some(ask) = self.password_ask.take() else {
            return;
        };
        if ask.text.is_empty() {
            self.error = Some("Enter a password.".to_string());
            self.password_ask = Some(PasswordAsk {
                request: ask.request,
                text: String::new(),
            });
            return;
        }
        if ask.text.as_bytes().contains(&0) {
            self.error = Some("The password cannot contain a NUL byte.".to_string());
            return;
        }
        let Ok(password) = Password::new(ask.text) else {
            return;
        };
        self.opened_secret = Some((ask.request.path.clone(), password.clone()));
        self.retry_with_password(ask.request, password);
    }

    fn retry_with_password(&mut self, request: PasswordRequest, password: Password) {
        match request.job {
            PasswordJob::List => self.start_list_with(request.path, Some(password)),
            PasswordJob::Test => self.spawn_test(request.path, Some(password)),
            PasswordJob::Extract {
                output,
                flat,
                overwrite,
                names,
            } => self.spawn_extract(request.path, output, flat, overwrite, names, Some(password)),
            PasswordJob::Delete { names } => self.spawn_delete(request.path, names, Some(password)),
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

    fn can_delete(&self) -> bool {
        if self.busy {
            return false;
        }
        let Place::Archive { format, .. } = &self.place else {
            return false;
        };
        format == "ZIP"
            && self
                .view
                .iter()
                .any(|row| row.kind != RowKind::Parent && self.explorer.is_selected(&row.key))
    }

    fn inside_archive(&self) -> bool {
        matches!(self.place, Place::Archive { .. })
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
            MenuCommand::Delete => self.request_delete(),
            MenuCommand::Info => self.show_info(),
            MenuCommand::Exit => {}
        }
    }

    fn on_tool(&mut self, command: ToolCommand) {
        match command {
            ToolCommand::Add => self.create_archive(),
            ToolCommand::Extract => self.extract_archive(),
            ToolCommand::Test => self.test_archive(),
            ToolCommand::Delete => self.request_delete(),
            ToolCommand::Info => self.show_info(),
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
        if dropped.is_empty() {
            return;
        }
        if dropped.len() == 1 {
            let path = &dropped[0];
            if path.is_dir() {
                self.open_folder(path.clone());
                return;
            }
            if is_supported_archive(path) || is_sfx_name(path) {
                self.start_list(path.clone());
                return;
            }
        }
        self.open_add_dialog(dropped);
    }

    fn poll_ask(&mut self) {
        let Some(ask) = &mut self.ask else {
            return;
        };
        if ask.pending.is_none()
            && let Ok(name) = ask.names.try_recv()
        {
            ask.pending = Some(name);
        }
    }

    fn answer_ask(&mut self, reply: AskReply) {
        if let Some(ask) = &self.ask {
            let _ = ask.replies.send(reply);
        }
        if let Some(ask) = &mut self.ask {
            ask.pending = None;
        }
        if matches!(reply, AskReply::Cancel) {
            self.progress.request_cancel();
            self.status = "Cancelling...".to_string();
        }
    }
}

fn needs_password(err: &Error) -> bool {
    matches!(err, Error::Encrypted(_) | Error::BadPassword)
}

fn list_archive(
    path: &Path,
    password: Option<&Password>,
) -> shellzip_core::Result<(shellzip_core::Opened, Vec<Entry>)> {
    list_with(
        path,
        &ListOpts {
            password: password.cloned(),
            ..ListOpts::default()
        },
    )
}

fn is_sfx_name(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("sfx"))
}

impl eframe::App for ShellZipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        self.poll_ask();
        self.password_window(ctx);
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
        let can_delete = self.can_delete();
        let busy = self.busy;
        let mut tool_command = None;
        egui::TopBottomPanel::top("toolbar")
            .frame(chrome::bar_frame(6, 2))
            .show(ctx, |ui| {
                tool_command = chrome::toolbar(ui, busy, archive_ready, can_delete);
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

        let jump_list = self.address_jumps();
        let mut crumb = None;
        egui::TopBottomPanel::top("crumbs")
            .frame(chrome::bar_frame(8, 2))
            .show_animated(ctx, self.inside_archive(), |ui| {
                crumb = chrome::breadcrumbs(ui, &jump_list);
            });
        if let Some(event) = crumb {
            self.on_address(event);
        }

        let summary = self.summary();
        let note = self.note().to_string();
        let progress = self.progress.snapshot();
        let mut status_event = chrome::StatusEvent {
            dismiss_error: false,
            cancel: false,
        };
        egui::TopBottomPanel::bottom("status")
            .frame(chrome::bar_frame(10, 4))
            .show(ctx, |ui| {
                status_event = chrome::status_bar(
                    ui,
                    &summary,
                    self.busy,
                    progress,
                    self.error.as_deref(),
                    &note,
                );
            });
        if status_event.dismiss_error {
            self.error = None;
        }
        if status_event.cancel {
            self.progress.request_cancel();
            self.status = "Cancelling...".to_string();
            if self.ask.as_ref().is_some_and(|ask| ask.pending.is_some()) {
                self.answer_ask(AskReply::Cancel);
            }
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
        chrome::info_window(ctx, &mut self.info);
        let add_action = self
            .add_dialog
            .as_mut()
            .and_then(|dialog| add_dialog::show(ctx, dialog));
        match add_action {
            Some(AddAction::Submit(request)) => {
                self.add_dialog = None;
                self.start_add(request);
            }
            Some(AddAction::Close) => self.add_dialog = None,
            None => {}
        }
        let pending_name = self.ask.as_ref().and_then(|ask| ask.pending.clone());
        if let Some(name) = pending_name
            && let Some(reply) = add_dialog::ask_overwrite(ctx, &name)
        {
            self.answer_ask(reply);
        }
        let mut delete_count = self.pending_delete.as_ref().map(Vec::len);
        if chrome::confirm_delete(ctx, &mut delete_count) {
            self.run_delete();
        } else if delete_count.is_none() {
            self.pending_delete = None;
        }
        if self.options != self.saved {
            if let Err(err) = crate::settings::save(&self.options)
                && self.error.is_none()
            {
                self.error = Some(err);
            }
            self.saved = self.options;
        }
        if !submitted && self.add_dialog.is_none() && self.ask.is_none() {
            self.shortcuts(ctx);
        }
    }
}

fn decide_member(
    name: &str,
    policy: &std::sync::Mutex<Option<ConflictChoice>>,
    names: &std::sync::mpsc::Sender<String>,
    replies: &std::sync::mpsc::Receiver<AskReply>,
) -> Result<ConflictChoice, shellzip_core::Error> {
    if let Some(choice) = *policy
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
    {
        return Ok(choice);
    }
    names
        .send(name.to_string())
        .map_err(|_| shellzip_core::Error::Interrupted)?;
    match replies.recv() {
        Ok(AskReply::Replace) => Ok(ConflictChoice::Overwrite),
        Ok(AskReply::Skip) => Ok(ConflictChoice::Skip),
        Ok(AskReply::Rename) => Ok(ConflictChoice::Rename),
        Ok(AskReply::ReplaceAll) => {
            *policy
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(ConflictChoice::Overwrite);
            Ok(ConflictChoice::Overwrite)
        }
        Ok(AskReply::SkipAll) => {
            *policy
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(ConflictChoice::Skip);
            Ok(ConflictChoice::Skip)
        }
        Ok(AskReply::Cancel) | Err(_) => Err(shellzip_core::Error::Interrupted),
    }
}

fn find_stub() -> Result<PathBuf, String> {
    if let Some(value) = std::env::var_os("SZIP_SFX_STUB") {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
        return Err(format!("SFX stub {} was not found", path.display()));
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        for name in shellzip_core::stub_file_names() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err("SFX stub not found. Build shellzip-sfx and place it next to ShellZip.".to_string())
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
