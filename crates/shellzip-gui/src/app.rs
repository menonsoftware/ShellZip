//! ShellZip window. Buttons call the same core operations as `szip`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};

use shellzip_core::{
    CompressOpts, ConflictChoice, Entry, Error, ExtractOpts, ListOpts, Password, SevenZOpts,
    SfxDirection, SfxPayload, UpdateMode, add, add_prompted, add_sfx, convert, delete, extract,
    list_with, test,
};
use shellzip_integrate::{selection_dir, suggested_archive_path};

use crate::about;
use crate::add_dialog::{self, AddAction, AddRequest, AskPrompt, AskReply};
use crate::chrome::{
    self, AddressEvent, DeleteChoice, MenuCommand, PackSettings, ShellIntegrateChoice,
    ShellIntegrateKind, ToolCommand, ToolbarState,
};
use crate::explorer::{
    ArchiveIndex, Explorer, FolderTree, JumpTarget, Row, RowKind, Target, TreeFocus,
    archive_prefix, folder_parent, format_size, is_supported_archive, jumps, location_text,
    path_under, read_directory, sort_rows, totals, totals_label,
};
use crate::extract_dialog::{self, ExtractAction, ExtractRequest};
use crate::icons::Icon;
use crate::info_dialog::{self, InfoAction};
use crate::launch::{self, LaunchAction};
use crate::progress_dialog::{self, ProgressCommand};
use crate::rename_dialog::{self, RenameAction, RenameTarget};
use crate::report_dialog::{self, ReportAction, ReportArchive, ReportRow};
use crate::settings::{
    self, DeleteArchive, ExtractSettings, ReportSettings, SfxSettings, ToolbarSize,
};
use crate::sfx_dialog::{self, SfxAction, SfxRequest};
use crate::shell_icons::IconCache;
use crate::test_dialog::{self, TestCommand};
use crate::theme::{self, Preference};
use crate::view_dialog::{self, PreviewData};
use crate::worker::ExtractJob;
use crate::worker::{self, JobToken, Outcome, PasswordJob, PasswordRequest, SharedProgress};

/// An unpacked archive waiting for a delete confirmation.
struct PendingArchiveDelete {
    path: PathBuf,
    recycle: bool,
    open: bool,
}

/// Password dialog waiting on the window.
struct PasswordAsk {
    request: PasswordRequest,
    text: String,
}

/// Archive work that owns the progress counters.
#[derive(Clone, Copy, PartialEq, Eq)]
enum OpKind {
    /// Unpack an archive.
    Extract,
    /// Create or update an archive.
    Add,
    /// Test checksums.
    Test,
    /// Remove members.
    Delete,
    /// Rename one member.
    Rename,
    /// Wrap a ZIP or 7z as a self-extractor.
    Sfx,
}

struct Operation {
    id: u64,
    kind: OpKind,
    /// Progress window is open and the file list is locked.
    foreground: bool,
    path: PathBuf,
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
    /// Extract, add, test, or delete currently running.
    operation: Option<Operation>,
    /// An archive open that should lock the file list. Background jobs leave this clear.
    listing: bool,
    /// Archive open whose result is still outstanding.
    browse_job: Option<u64>,
    next_id: u64,
    loaded: bool,
    about: bool,
    info_dialog: Option<info_dialog::InfoDialog>,
    info_job: Option<u64>,
    view_dialog: Option<view_dialog::ViewDialog>,
    view_job: Option<u64>,
    rename_dialog: Option<rename_dialog::RenameDialog>,
    test_dialog: Option<test_dialog::TestDialog>,
    report_dialog: Option<report_dialog::ReportDialog>,
    report_job: Option<u64>,
    pending_delete: Option<Vec<String>>,
    pending_shell: Option<ShellIntegrateKind>,
    /// Outstanding Velopack update check, if any.
    update_job: Option<u64>,
    add_dialog: Option<add_dialog::AddDialog>,
    extract_dialog: Option<extract_dialog::ExtractDialog>,
    sfx_dialog: Option<sfx_dialog::SfxDialog>,
    pending_archive_delete: Option<PendingArchiveDelete>,
    ask: Option<AskPrompt>,
    password_ask: Option<PasswordAsk>,
    /// Password that opened the archive currently shown, if it needed one.
    opened_secret: Option<(PathBuf, Password)>,
    options: PackSettings,
    saved: PackSettings,
    extract: ExtractSettings,
    sfx: SfxSettings,
    recent: Vec<String>,
    /// Recently opened archives, newest first.
    opened: Vec<String>,
    /// Light, dark, or follow the operating system.
    theme: Preference,
    /// Large or extra-large toolbar glyphs.
    toolbar: ToolbarSize,
    /// Generate-report choices.
    report: ReportSettings,
    /// Archive folder to restore after a refresh reloads the same file.
    restore_prefix: Option<(u64, String)>,
    generation: u64,
    icons: IconCache,
    tree: FolderTree,
    progress: Arc<SharedProgress>,
    tx: Sender<Outcome>,
    rx: Receiver<Outcome>,
    /// Shell-menu extract jobs waiting until the previous one finishes.
    shell_queue: Vec<(PathBuf, PathBuf, bool)>,
    /// Remaining archives for a multi-file **Extract to…** after the dialog.
    shell_extract_to: Vec<PathBuf>,
    /// True when this window was opened only for a shell context-menu verb.
    shell_verb: bool,
    /// Close the window on the next frame after a shell verb finishes.
    close_after_shell: bool,
}

impl ShellZipApp {
    /// Install image loaders, apply the file-manager theme, and honour `launch`.
    pub fn new(cc: &eframe::CreationContext<'_>, launch: LaunchAction) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let stored = settings::load();
        theme::apply(&cc.egui_ctx, stored.theme);
        let (tx, rx) = mpsc::channel();
        let open_path = match &launch {
            LaunchAction::Open(path) if path.exists() => Some(path.clone()),
            LaunchAction::Add(paths)
            | LaunchAction::AddZip(paths)
            | LaunchAction::Add7z(paths)
            | LaunchAction::ExtractTo(paths)
            | LaunchAction::ExtractHere(paths)
            | LaunchAction::ExtractHereNamed(paths) => paths.first().cloned(),
            _ => None,
        };
        let open_file = open_path.as_ref().is_some_and(|path| path.is_file());
        let path = match &open_path {
            Some(path) if path.is_dir() => path.clone(),
            Some(path) => path
                .parent()
                .filter(|parent| parent.is_dir())
                .map(Path::to_path_buf)
                .unwrap_or_else(starting_folder),
            None => starting_folder(),
        };
        let generation = 1;
        let options = stored.pack;
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
            operation: None,
            listing: false,
            browse_job: None,
            next_id: 0,
            loaded: false,
            about: false,
            info_dialog: None,
            info_job: None,
            view_dialog: None,
            view_job: None,
            rename_dialog: None,
            test_dialog: None,
            report_dialog: None,
            report_job: None,
            pending_delete: None,
            pending_shell: None,
            update_job: None,
            add_dialog: None,
            extract_dialog: None,
            sfx_dialog: None,
            pending_archive_delete: None,
            ask: None,
            password_ask: None,
            opened_secret: None,
            options,
            saved: options,
            extract: stored.extract,
            sfx: stored.sfx,
            recent: stored.recent,
            opened: stored.opened,
            theme: stored.theme,
            toolbar: stored.toolbar,
            report: stored.report,
            restore_prefix: None,
            generation,
            icons: IconCache::new(),
            tree: FolderTree::new(),
            progress: Arc::new(SharedProgress::new()),
            tx,
            rx,
            shell_queue: Vec::new(),
            shell_extract_to: Vec::new(),
            shell_verb: false,
            close_after_shell: false,
        };
        app.rebuild_view();
        app.apply_launch(launch);
        app
    }

    fn apply_launch(&mut self, launch: LaunchAction) {
        match launch {
            LaunchAction::Browse => {}
            LaunchAction::Open(path) if path.is_file() => self.start_list(path),
            LaunchAction::Open(path) if path.is_dir() => self.open_folder(path),
            LaunchAction::Open(_) => {}
            LaunchAction::Add(paths) => {
                self.shell_verb = true;
                self.begin_shell_add_dialog(paths);
            }
            LaunchAction::AddZip(paths) => {
                self.shell_verb = true;
                self.begin_shell_quick_add(paths, "zip");
            }
            LaunchAction::Add7z(paths) => {
                self.shell_verb = true;
                self.begin_shell_quick_add(paths, "7z");
            }
            LaunchAction::ExtractTo(paths) => {
                self.shell_verb = true;
                self.begin_shell_extract_to(paths);
            }
            LaunchAction::ExtractHere(paths) => {
                self.shell_verb = true;
                let jobs = paths
                    .into_iter()
                    .map(|archive| {
                        let output = launch::extract_here_dir(&archive);
                        (archive, output, false)
                    })
                    .collect();
                self.enqueue_shell_extracts(jobs);
            }
            LaunchAction::ExtractHereNamed(paths) => {
                self.shell_verb = true;
                let jobs = paths
                    .into_iter()
                    .map(|archive| {
                        let output = launch::extract_here_named_dir(&archive);
                        (archive, output, false)
                    })
                    .collect();
                self.enqueue_shell_extracts(jobs);
            }
        }
    }

    /// Close after a shell-menu add/extract once nothing else is pending.
    ///
    /// Errors keep the window open so the message stays visible.
    fn maybe_close_shell_verb(&mut self) {
        if !self.shell_verb || self.close_after_shell {
            return;
        }
        if self.error.is_some()
            || self.operation.is_some()
            || !self.shell_queue.is_empty()
            || self.add_dialog.is_some()
            || self.extract_dialog.is_some()
            || self.password_ask.is_some()
            || self.ask.is_some()
            || self.pending_archive_delete.is_some()
        {
            return;
        }
        self.close_after_shell = true;
    }

    fn begin_shell_add_dialog(&mut self, files: Vec<PathBuf>) {
        if self.refuse_operation() || files.is_empty() || self.add_dialog.is_some() {
            return;
        }
        let dir = selection_dir(&files);
        self.add_dialog = Some(add_dialog::AddDialog::prepare(&dir, files, &self.options));
    }

    fn begin_shell_quick_add(&mut self, files: Vec<PathBuf>, extension: &str) {
        if self.refuse_operation() || files.is_empty() {
            return;
        }
        let dest = suggested_archive_path(&files, extension);
        if quick_add_needs_dialog(&files, &dest, extension) {
            self.begin_shell_add_dialog(files);
            return;
        }
        let request = AddRequest {
            dest,
            inputs: files,
            level: self.options.level.min(5),
            method: self.options.method,
            update: UpdateMode::Add,
            sfx: false,
            password: String::new(),
            sevenz: SevenZOpts {
                method: self.options.sevenz_method,
                solid: self.options.solid,
                encrypt_headers: false,
            },
            sfx_payload: if extension.eq_ignore_ascii_case("7z") {
                SfxPayload::SevenZ
            } else {
                SfxPayload::Zip
            },
            comment: None,
            background: self.options.background,
        };
        self.start_add(request);
    }

    fn begin_shell_extract_to(&mut self, mut paths: Vec<PathBuf>) {
        let Some(first) = paths.first().cloned() else {
            return;
        };
        // Check before queueing rest paths — otherwise a refused call leaves
        // stale entries that the next Submit drains into `shell_queue`.
        if self.refuse_operation() || self.extract_dialog.is_some() {
            return;
        }
        self.shell_extract_to = paths.split_off(1);
        self.extract_dialog = Some(extract_dialog::ExtractDialog::prepare(
            &first,
            None,
            "",
            self.extract,
        ));
    }

    fn enqueue_shell_extracts(&mut self, mut jobs: Vec<(PathBuf, PathBuf, bool)>) {
        if jobs.is_empty() {
            return;
        }
        if self.operation.is_none() {
            let (archive, output, show) = jobs.remove(0);
            self.shell_queue = jobs;
            self.start_shell_extract(archive, output, show);
        } else {
            self.shell_queue.extend(jobs);
        }
    }

    fn start_shell_extract(&mut self, archive: PathBuf, output: PathBuf, show_in_explorer: bool) {
        let settings = self.extract;
        let flat = settings.paths == settings::PathMode::None;
        let request = ExtractRequest {
            archive,
            output,
            update: settings.update,
            overwrite: settings.overwrite,
            flat,
            strip_prefix: String::new(),
            keep_broken: settings.keep_broken,
            restore_mtime: settings.restore_mtime,
            show_in_explorer,
            delete: settings.delete,
            recycle: settings.recycle,
            background: settings.background,
            names: None,
        };
        self.start_extract(request);
    }

    fn pump_shell_queue(&mut self) {
        if self.operation.is_some() {
            return;
        }
        if let Some((archive, output, show)) = self.shell_queue.first().cloned() {
            self.shell_queue.remove(0);
            self.start_shell_extract(archive, output, show);
        }
    }

    fn open_archive(&mut self) {
        if self.shell_locked() {
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
        if self.shell_locked() {
            return;
        }
        self.remember_opened(&path);
        let generation = self.next_generation();
        let id = self.alloc_id();
        let background = self.operation.as_ref().is_some_and(|op| !op.foreground);
        if !background {
            self.listing = true;
            self.error = None;
            self.status = format!("Opening {}", path.display());
        }
        self.browse_job = Some(id);
        let token = JobToken { id, generation };
        let tx = self.tx.clone();
        let again = password.is_some();
        worker::spawn(tx, token, move || {
            match list_archive(&path, password.as_ref()) {
                Ok((opened, entries)) => Ok(Outcome::Listed {
                    token,
                    format: opened.detected().label(),
                    path,
                    entries,
                }),
                Err(err) if needs_password(&err) => Ok(Outcome::NeedPassword(PasswordRequest {
                    id,
                    generation,
                    path,
                    job: PasswordJob::List,
                    again,
                    foreground: true,
                })),
                Err(err) => Err(err),
            }
        });
    }

    fn extract_archive(&mut self) {
        let Some(archive) = self.command_archive() else {
            self.error = Some("Open an archive, or select one.".to_string());
            return;
        };
        if self.refuse_operation() || self.extract_dialog.is_some() {
            return;
        }
        let names = self.member_filter(&archive);
        let prefix = match &self.place {
            Place::Archive { path, prefix, .. } if path == &archive => prefix.clone(),
            _ => String::new(),
        };
        self.extract_dialog = Some(extract_dialog::ExtractDialog::prepare(
            &archive,
            names,
            &prefix,
            self.extract,
        ));
    }

    fn start_extract(&mut self, request: ExtractRequest) {
        let password = self.saved_password(&request.archive);
        let archive = request.archive;
        let job = ExtractJob {
            output: request.output,
            update: request.update,
            overwrite: request.overwrite,
            flat: request.flat,
            strip_prefix: request.strip_prefix,
            keep_broken: request.keep_broken,
            restore_mtime: request.restore_mtime,
            show_in_explorer: request.show_in_explorer,
            delete: request.delete,
            recycle: request.recycle,
            names: request.names,
        };
        self.spawn_extract(archive, job, password, !request.background);
        if self.extract.background != request.background {
            self.extract.background = request.background;
            self.persist_settings();
        }
    }

    fn spawn_extract(
        &mut self,
        archive: PathBuf,
        job: ExtractJob,
        password: Option<Password>,
        foreground: bool,
    ) {
        if self.refuse_operation() {
            return;
        }
        let asking = job.overwrite == shellzip_core::Overwrite::Ask;
        let progress = Arc::clone(&self.progress);
        let token = self.begin_operation(OpKind::Extract, archive.clone(), foreground);
        let tx = self.tx.clone();
        let again = password.is_some();
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
                detail: "This file is already in the destination.",
            });
            (names_tx, replies_rx)
        });
        let id = token.id;
        let generation = token.generation;
        worker::spawn(tx, token, move || {
            let mut opts = ExtractOpts::new(&job.output);
            opts.update = job.update;
            opts.overwrite = job.overwrite;
            opts.flat = job.flat;
            opts.strip_prefix = job.strip_prefix.clone();
            opts.keep_broken = job.keep_broken;
            opts.restore_mtime = job.restore_mtime;
            opts.password = password;
            if job.names.is_some() {
                opts.exact_paths = job.names.clone();
            }
            let report = if let Some((names_tx, replies_rx)) = prompt {
                let policy = std::sync::Mutex::new(None);
                let decide = move |path: &Path| {
                    decide_member(&path.display().to_string(), &policy, &names_tx, &replies_rx)
                };
                extract(&archive, &opts, Some(&decide), progress.as_ref())
            } else {
                extract(&archive, &opts, None, progress.as_ref())
            };
            match report {
                Ok(report) => Ok(Outcome::Extracted {
                    token,
                    summary: format!("extracted {}, skipped {}", report.extracted, report.skipped),
                    output: job.output,
                    archive,
                    show_in_explorer: job.show_in_explorer,
                    delete: job.delete,
                    recycle: job.recycle,
                    clean: report.warnings.is_empty(),
                    full: job.names.is_none(),
                    wrote: report.extracted > 0,
                }),
                Err(err) if needs_password(&err) => Ok(Outcome::NeedPassword(PasswordRequest {
                    id,
                    generation,
                    path: archive,
                    job: PasswordJob::Extract(job),
                    again,
                    foreground,
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
        if self.refuse_operation() {
            return;
        }
        let password = self.saved_password(&archive);
        self.spawn_test(archive, password);
    }

    fn spawn_test(&mut self, archive: PathBuf, password: Option<Password>) {
        if self.refuse_operation() {
            return;
        }
        let progress = Arc::clone(&self.progress);
        let token = self.begin_operation(OpKind::Test, archive.clone(), true);
        let tx = self.tx.clone();
        let again = password.is_some();
        let id = token.id;
        let generation = token.generation;
        self.test_dialog = Some(test_dialog::TestDialog::running(
            archive.display().to_string(),
        ));
        worker::spawn(tx, token, move || {
            let report = match test(&archive, password.as_ref(), progress.as_ref()) {
                Err(err) if needs_password(&err) => {
                    return Ok(Outcome::NeedPassword(PasswordRequest {
                        id,
                        generation,
                        path: archive,
                        job: PasswordJob::Test,
                        again,
                        foreground: true,
                    }));
                }
                other => other?,
            };
            if report.checksums.is_empty() && !report.encrypted.is_empty() && !again {
                return Ok(Outcome::NeedPassword(PasswordRequest {
                    id,
                    generation,
                    path: archive,
                    job: PasswordJob::Test,
                    again: false,
                    foreground: true,
                }));
            }
            Ok(Outcome::Tested { token, report })
        });
    }

    fn create_archive(&mut self) {
        if self.refuse_operation() {
            return;
        }
        let target = self.updatable_archive();
        let mut files = self.selected_inputs();
        if files.is_empty() {
            let Some(picked) = rfd::FileDialog::new().pick_files() else {
                return;
            };
            files = picked;
        }
        self.open_add_dialog(files, target);
    }

    fn open_add_dialog(&mut self, files: Vec<PathBuf>, target: Option<PathBuf>) {
        if self.refuse_operation() || files.is_empty() || self.add_dialog.is_some() {
            return;
        }
        let dialog = if let Some(archive) = target {
            let secret = archive
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("7z"))
                .then(|| self.saved_password(&archive))
                .flatten()
                .map(|password| password.expose().to_string());
            add_dialog::AddDialog::prepare_into(&archive, secret.as_deref(), files, &self.options)
        } else {
            add_dialog::AddDialog::prepare(&self.current_dir(), files, &self.options)
        };
        self.add_dialog = Some(dialog);
    }

    fn create_sfx(&mut self) {
        if self.refuse_operation() || self.sfx_dialog.is_some() {
            return;
        }
        let Some(source) = self.sfx_source() else {
            self.error = Some(
                "Open a ZIP, 7z, or ShellZip self-extractor to create or unwrap one.".to_string(),
            );
            return;
        };
        let stripping = shellzip_core::locate(&source).ok().flatten().is_some();
        if !stripping && let Err(message) = find_stub() {
            self.error = Some(message);
            return;
        }
        let suggested = suggested_sfx_path(&source);
        self.sfx_dialog = Some(sfx_dialog::SfxDialog::prepare(
            &source, &suggested, self.sfx, stripping,
        ));
    }

    fn start_sfx(&mut self, request: SfxRequest) {
        // Wrapping builds a new package beside the open archive — keep the
        // progress window like other archive-mutating jobs. Stripping may run
        // in the background when the dialog asks for it.
        let stripping = shellzip_core::locate(&request.source)
            .ok()
            .flatten()
            .is_some();
        let stub = if stripping {
            None
        } else {
            match find_stub() {
                Ok(path) => Some(path),
                Err(message) => {
                    self.error = Some(message);
                    return;
                }
            }
        };
        let background = stripping && request.background;
        if stripping && self.sfx.background != request.background {
            self.sfx.background = request.background;
            self.persist_settings();
        }
        let source = request.source;
        let dest = request.dest;
        let foreground = !background;
        let token = self.begin_operation(OpKind::Sfx, dest.clone(), foreground);
        let progress = Arc::clone(&self.progress);
        let tx = self.tx.clone();
        worker::spawn(tx, token, move || {
            let direction = convert(&source, &dest, stub.as_deref(), progress.as_ref())?;
            let summary = match direction {
                SfxDirection::Wrapped => format!("created {}", dest.display()),
                SfxDirection::Stripped => format!("wrote {}", dest.display()),
            };
            Ok(Outcome::Finished { token, summary })
        });
    }

    /// ZIP / 7z to wrap, or an open ShellZip SFX to unwrap.
    fn sfx_source(&self) -> Option<PathBuf> {
        match &self.place {
            Place::Archive { format, path, .. }
                if format == "ZIP" || format == "7Z" || format.starts_with("SFX") =>
            {
                Some(path.clone())
            }
            _ => None,
        }
    }

    /// Open ZIP or 7z the file pane is inside. Other formats stay create-new.
    fn updatable_archive(&self) -> Option<PathBuf> {
        match &self.place {
            Place::Archive { format, path, .. } if format == "ZIP" || format == "7Z" => {
                Some(path.clone())
            }
            _ => None,
        }
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
        if self.refuse_operation() {
            return;
        }
        self.options.level = request.level.min(5);
        self.options.method = request.method;
        self.options.sevenz_method = request.sevenz.method;
        self.options.solid = request.sevenz.solid;
        self.options.background = request.background;
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
        let foreground = !request.background;
        self.next_generation();
        let progress = Arc::clone(&self.progress);
        let token = self.begin_operation(OpKind::Add, request.dest.clone(), foreground);
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
                detail: "This name is already in the archive.",
            });
            (names_tx, replies_rx)
        });
        worker::spawn(tx, token, move || {
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
                    token,
                    path: dest,
                    format: opened.detected().label(),
                    entries,
                }),
                Err(err) => Ok(Outcome::Finished {
                    token,
                    summary: format!(
                        "created {} ({}); listing failed: {err}",
                        report.path.display(),
                        format_size(report.packed_bytes)
                    ),
                }),
            }
        });
    }

    fn request_delete(&mut self) {
        if self.refuse_operation() {
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
        if self.refuse_operation() {
            return;
        }
        self.next_generation();
        let progress = Arc::clone(&self.progress);
        let token = self.begin_operation(OpKind::Delete, archive.clone(), true);
        let tx = self.tx.clone();
        let again = password.is_some();
        let listed = password.clone();
        let id = token.id;
        let generation = token.generation;
        worker::spawn(tx, token, move || {
            let report = match delete(&archive, &names, password.as_ref(), progress.as_ref()) {
                Err(err) if needs_password(&err) => {
                    return Ok(Outcome::NeedPassword(PasswordRequest {
                        id,
                        generation,
                        path: archive,
                        job: PasswordJob::Delete { names },
                        again,
                        foreground: true,
                    }));
                }
                other => other?,
            };
            match list_archive(&archive, listed.as_ref()) {
                Ok((opened, entries)) => Ok(Outcome::Listed {
                    token,
                    path: archive,
                    format: opened.detected().label(),
                    entries,
                }),
                Err(err) => Ok(Outcome::Finished {
                    token,
                    summary: format!("deleted {} ({err})", report.removed),
                }),
            }
        });
    }

    fn show_info(&mut self) {
        if self.refuse_operation() {
            return;
        }
        let Some(path) = self.command_archive() else {
            self.error = Some("Select an archive, or open one.".to_string());
            return;
        };
        self.spawn_summary(path);
    }

    fn spawn_summary(&mut self, path: PathBuf) {
        let id = self.alloc_id();
        let generation = self.generation;
        let token = JobToken { id, generation };
        self.info_job = Some(id);
        self.info_dialog = None;
        self.error = None;
        self.status = format!("Reading {}", path.display());
        let tx = self.tx.clone();
        worker::spawn(tx, token, move || {
            let summary = shellzip_core::archive_summary(&path)?;
            Ok(Outcome::Summarized {
                token,
                path,
                summary,
            })
        });
    }

    fn view_file(&mut self) {
        if self.refuse_operation() {
            return;
        }
        let selected = self.selected_rows();
        let [row] = selected.as_slice() else {
            self.error = Some("Select one file to view.".to_string());
            return;
        };
        if matches!(row.kind, RowKind::Directory | RowKind::Parent) {
            self.error = Some("Select a file, not a folder.".to_string());
            return;
        }
        match &row.target {
            Target::Path(path) => {
                self.spawn_view_disk(path.clone(), row.name.clone(), row.size);
            }
            Target::ArchiveMember(member) => {
                let Some(archive) = self.command_archive() else {
                    self.error = Some("Open an archive to view this file.".to_string());
                    return;
                };
                let password = self.saved_password(&archive);
                self.spawn_view_member(
                    archive,
                    member.clone(),
                    row.name.clone(),
                    row.size,
                    password,
                );
            }
            Target::Parent | Target::ArchiveDir(_) => {
                self.error = Some("Select a file, not a folder.".to_string());
            }
        }
    }

    fn spawn_view_disk(&mut self, path: PathBuf, label: String, size: Option<u64>) {
        let id = self.alloc_id();
        let generation = self.generation;
        let token = JobToken { id, generation };
        self.view_job = Some(id);
        self.view_dialog = None;
        self.error = None;
        self.status = format!("Reading {label}");
        let tx = self.tx.clone();
        worker::spawn(tx, token, move || {
            let data = view_dialog::read_disk(&path, size)?;
            Ok(preview_outcome(token, label, data))
        });
    }

    fn spawn_view_member(
        &mut self,
        archive: PathBuf,
        member: String,
        label: String,
        size: Option<u64>,
        password: Option<Password>,
    ) {
        let id = self.alloc_id();
        let generation = self.generation;
        let token = JobToken { id, generation };
        self.view_job = Some(id);
        self.view_dialog = None;
        self.error = None;
        self.status = format!("Reading {label}");
        let tx = self.tx.clone();
        let again = password.is_some();
        worker::spawn(tx, token, move || {
            match view_dialog::read_member(&archive, &member, password.as_ref(), size) {
                Ok(data) => Ok(preview_outcome(token, label, data)),
                Err(err) if needs_password(&err) => Ok(Outcome::NeedPassword(PasswordRequest {
                    id,
                    generation,
                    path: archive,
                    job: PasswordJob::View {
                        member,
                        label,
                        size,
                    },
                    again,
                    foreground: false,
                })),
                Err(err) => Err(err),
            }
        });
    }

    fn rename_selected(&mut self) {
        if self.refuse_operation() {
            return;
        }
        let selected = self.selected_rows();
        let [row] = selected.as_slice() else {
            self.error = Some("Select one file or folder to rename.".to_string());
            return;
        };
        if matches!(row.kind, RowKind::Parent) {
            self.error = Some("Select a file or folder to rename.".to_string());
            return;
        }
        let target = match &row.target {
            Target::Path(path) => RenameTarget::Disk(path.clone()),
            Target::ArchiveMember(member) | Target::ArchiveDir(member) => {
                if let Place::Archive { format, .. } = &self.place
                    && format != "ZIP"
                    && format != "7Z"
                {
                    self.error = Some("Rename only rewrites ZIP and 7z archives.".to_string());
                    return;
                }
                RenameTarget::Member(member.trim_end_matches('/').to_string())
            }
            Target::Parent => {
                self.error = Some("Select a file or folder to rename.".to_string());
                return;
            }
        };
        self.rename_dialog = Some(rename_dialog::RenameDialog::new(row.name.clone(), target));
    }

    fn apply_rename(&mut self, target: RenameTarget, leaf: String) {
        if let Some(message) = rename_dialog::reject_new_name(&leaf) {
            self.error = Some(message.to_string());
            return;
        }
        match target {
            RenameTarget::Disk(path) => self.rename_on_disk(&path, &leaf),
            RenameTarget::Member(from) => self.rename_in_archive(&from, &leaf),
        }
    }

    fn rename_on_disk(&mut self, path: &Path, leaf: &str) {
        if path.file_name().is_some_and(|name| name == leaf) {
            return;
        }
        let Some(parent) = path.parent() else {
            self.error = Some(format!("rename {}: the file has no folder", path.display()));
            return;
        };
        let dest = parent.join(leaf);
        if dest.exists() {
            self.error = Some(format!("{leaf} already exists."));
            return;
        }
        if let Err(err) = std::fs::rename(path, &dest) {
            self.error = Some(format!("rename {}: {err}", path.display()));
            return;
        }
        self.pending_select = Some(dest.display().to_string());
        self.refresh();
    }

    fn rename_in_archive(&mut self, from: &str, leaf: &str) {
        let (format, path) = match &self.place {
            Place::Archive { format, path, .. } => (format.clone(), path.clone()),
            Place::Folder { .. } => {
                self.error = Some("Open a ZIP or 7z archive to rename members.".to_string());
                return;
            }
        };
        if format != "ZIP" && format != "7Z" {
            self.error = Some("Rename only rewrites ZIP and 7z archives.".to_string());
            return;
        }
        let from = from.trim_end_matches('/').to_string();
        let to = rename_dialog::renamed_member(&from, leaf);
        if to == from {
            return;
        }
        let password = self.saved_password(&path);
        self.spawn_rename(path, from, to, password);
    }

    fn spawn_rename(
        &mut self,
        archive: PathBuf,
        from: String,
        to: String,
        password: Option<Password>,
    ) {
        if self.refuse_operation() {
            return;
        }
        self.next_generation();
        let progress = Arc::clone(&self.progress);
        let token = self.begin_operation(OpKind::Rename, archive.clone(), true);
        let tx = self.tx.clone();
        let again = password.is_some();
        let listed = password.clone();
        let id = token.id;
        let generation = token.generation;
        worker::spawn(tx, token, move || {
            let report = match shellzip_core::rename(
                &archive,
                &from,
                &to,
                password.as_ref(),
                progress.as_ref(),
            ) {
                Err(err) if needs_password(&err) => {
                    return Ok(Outcome::NeedPassword(PasswordRequest {
                        id,
                        generation,
                        path: archive,
                        job: PasswordJob::Rename { from, to },
                        again,
                        foreground: true,
                    }));
                }
                other => other?,
            };
            match list_archive(&archive, listed.as_ref()) {
                Ok((opened, entries)) => Ok(Outcome::Listed {
                    token,
                    path: archive,
                    format: opened.detected().label(),
                    entries,
                }),
                Err(err) => Ok(Outcome::Finished {
                    token,
                    summary: format!("renamed {} ({err})", report.kept),
                }),
            }
        });
    }

    fn open_report(&mut self) {
        if self.refuse_operation() || self.report_dialog.is_some() {
            return;
        }
        match &self.place {
            Place::Archive { path, .. } => {
                let path = path.clone();
                let dest = report_dialog::default_report_path(&path, self.report.format);
                self.report_dialog = Some(report_dialog::ReportDialog::open(
                    self.report,
                    dest,
                    Vec::new(),
                    true,
                ));
                self.spawn_report_lists(vec![path]);
            }
            Place::Folder { path, .. } => {
                let selected = self.selected_rows();
                if selected.is_empty() {
                    self.error = Some("Select files or archives, or open an archive.".to_string());
                    return;
                }
                let mut loose = Vec::new();
                let mut archives = Vec::new();
                for row in &selected {
                    match &row.target {
                        Target::Path(file) if row.kind == RowKind::Archive => {
                            archives.push(file.clone());
                        }
                        Target::Path(_) => loose.push(report_row(row)),
                        Target::Parent | Target::ArchiveMember(_) | Target::ArchiveDir(_) => {}
                    }
                }
                if loose.is_empty() && archives.is_empty() {
                    self.error = Some("Select files or archives, or open an archive.".to_string());
                    return;
                }
                let anchor = archives
                    .first()
                    .cloned()
                    .unwrap_or_else(|| path.join("report"));
                let dest = report_dialog::default_report_path(&anchor, self.report.format);
                let pending = !archives.is_empty();
                self.report_dialog = Some(report_dialog::ReportDialog::open(
                    self.report,
                    dest,
                    loose,
                    pending,
                ));
                if pending {
                    self.spawn_report_lists(archives);
                }
            }
        }
    }

    fn spawn_report_lists(&mut self, archives: Vec<PathBuf>) {
        let id = self.alloc_id();
        let generation = self.generation;
        let token = JobToken { id, generation };
        self.report_job = Some(id);
        let secret = self.opened_secret.clone();
        let tx = self.tx.clone();
        worker::spawn(tx, token, move || {
            let mut listed = Vec::new();
            for path in archives {
                let password = secret
                    .as_ref()
                    .filter(|(saved, _)| saved == &path)
                    .map(|(_, password)| password);
                let (_opened, entries) = list_archive(&path, password)?;
                listed.push(ReportArchive {
                    name: path.display().to_string(),
                    rows: entries.into_iter().map(report_entry).collect(),
                });
            }
            Ok(Outcome::ReportListed {
                token,
                archives: listed,
            })
        });
    }

    fn selected_rows(&self) -> Vec<Row> {
        self.view
            .iter()
            .filter(|row| row.kind != RowKind::Parent && self.explorer.is_selected(&row.key))
            .cloned()
            .collect()
    }

    fn begin_operation(&mut self, kind: OpKind, path: PathBuf, foreground: bool) -> JobToken {
        let foreground = match kind {
            OpKind::Test | OpKind::Delete | OpKind::Rename => true,
            OpKind::Extract | OpKind::Add | OpKind::Sfx => foreground,
        };
        let id = self.alloc_id();
        let generation = self.generation;
        self.operation = Some(Operation {
            id,
            kind,
            foreground,
            path,
        });
        self.error = None;
        self.status.clear();
        self.progress.reset();
        JobToken { id, generation }
    }

    fn clear_operation(&mut self) {
        self.operation = None;
        self.progress.set_paused(false);
    }

    fn alloc_id(&mut self) -> u64 {
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 {
            self.next_id = 1;
        }
        self.next_id
    }

    /// Archive commands are busy. A background extract or add still counts.
    fn refuse_operation(&mut self) -> bool {
        if self.operation.is_some() {
            self.error = Some("An archive operation is already running.".to_string());
            true
        } else {
            self.listing
        }
    }

    /// The file list ignores navigation. A background job leaves this false.
    fn shell_locked(&self) -> bool {
        self.listing || self.operation.as_ref().is_some_and(|op| op.foreground)
    }

    /// A modal or foreground progress window owns the shell.
    fn dialog_blocks_shell(&self) -> bool {
        self.about
            || self.add_dialog.is_some()
            || self.extract_dialog.is_some()
            || self.sfx_dialog.is_some()
            || self.info_dialog.is_some()
            || self.view_dialog.is_some()
            || self.rename_dialog.is_some()
            || self.test_dialog.is_some()
            || self.report_dialog.is_some()
            || self.pending_delete.is_some()
            || self.pending_shell.is_some()
            || self.pending_archive_delete.is_some()
            || self.password_ask.is_some()
            || self.ask.as_ref().is_some_and(|ask| ask.pending.is_some())
            || self.operation.as_ref().is_some_and(|op| op.foreground)
    }

    fn running_status(&self) -> Option<chrome::RunningStatus> {
        if let Some(op) = &self.operation {
            let snapshot = self.progress.snapshot();
            let (fraction, indeterminate) =
                crate::progress_dialog::job_fraction(snapshot.done, snapshot.total);
            let interactive = matches!(op.kind, OpKind::Extract | OpKind::Add | OpKind::Sfx);
            let background = interactive && !op.foreground;
            return Some(chrome::RunningStatus {
                text: crate::progress_dialog::status_text(&snapshot),
                fraction,
                indeterminate,
                paused: snapshot.paused,
                pause: background,
                details: background,
            });
        }
        if self.listing {
            let text = if self.status.is_empty() {
                "Opening".to_string()
            } else {
                self.status.clone()
            };
            return Some(chrome::RunningStatus {
                text,
                fraction: 0.0,
                indeterminate: true,
                paused: false,
                pause: false,
                details: false,
            });
        }
        None
    }

    fn show_progress(&mut self, ctx: &egui::Context) {
        let Some(op) = &self.operation else {
            return;
        };
        if !op.foreground {
            return;
        }
        let title = match op.kind {
            OpKind::Extract => "Extracting",
            OpKind::Add => "Adding",
            OpKind::Sfx => "Creating self-extractor",
            OpKind::Test | OpKind::Delete | OpKind::Rename => return,
        };
        let path = op.path.display().to_string();
        let snapshot = self.progress.snapshot();
        let Some(command) = progress_dialog::show(ctx, title, &path, &snapshot) else {
            return;
        };
        match command {
            ProgressCommand::Pause => self.progress.set_paused(true),
            ProgressCommand::Resume => self.progress.set_paused(false),
            ProgressCommand::Background => {
                if let Some(op) = &mut self.operation {
                    op.foreground = false;
                }
            }
            ProgressCommand::Cancel => {
                self.progress.request_cancel();
                self.status = "Cancelling...".to_string();
                if self.ask.as_ref().is_some_and(|ask| ask.pending.is_some()) {
                    self.answer_ask(AskReply::Cancel);
                }
            }
        }
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
                    token,
                    path,
                    format,
                    entries,
                } => self.finish_listed(token, path, format, entries),
                Outcome::Finished { token, summary } => {
                    if self.operation.as_ref().is_some_and(|op| op.id == token.id) {
                        let kind = self.operation.as_ref().map(|op| op.kind);
                        self.clear_operation();
                        self.ask = None;
                        if matches!(kind, Some(OpKind::Test)) {
                            self.test_dialog = None;
                        }
                        self.status = summary;
                        if matches!(kind, Some(OpKind::Add | OpKind::Extract)) {
                            self.maybe_close_shell_verb();
                        }
                    } else if self.browse_job == Some(token.id) {
                        self.browse_job = None;
                        self.listing = false;
                        if token.generation == self.generation {
                            self.status = summary;
                        }
                    } else {
                        self.settle_side_status(token);
                    }
                }
                Outcome::Extracted {
                    token,
                    summary,
                    output,
                    archive,
                    show_in_explorer,
                    delete,
                    recycle,
                    clean,
                    full,
                    wrote,
                } => {
                    if !self.operation.as_ref().is_some_and(|op| op.id == token.id) {
                        continue;
                    }
                    self.clear_operation();
                    self.ask = None;
                    self.status = summary;
                    self.remember_destination(&output);
                    if show_in_explorer {
                        self.open_destination(&output);
                    }
                    if clean && full && wrote && delete != DeleteArchive::Never {
                        if delete == DeleteArchive::Always {
                            self.remove_archive(&archive, recycle);
                        } else {
                            self.pending_archive_delete = Some(PendingArchiveDelete {
                                path: archive,
                                recycle,
                                open: true,
                            });
                        }
                    }
                    self.maybe_close_shell_verb();
                }
                Outcome::Tested { token, report } => {
                    if self.operation.as_ref().is_some_and(|op| op.id == token.id) {
                        self.clear_operation();
                        self.ask = None;
                        self.status = test_dialog::summary_line(&report);
                        if let Some(dialog) = &mut self.test_dialog {
                            dialog.finish(&report);
                        }
                    }
                }
                Outcome::Previewed {
                    token,
                    name,
                    body,
                    truncated,
                    size,
                } => {
                    if self.view_job == Some(token.id) {
                        self.view_job = None;
                        if token.generation == self.generation {
                            self.status.clear();
                            self.view_dialog = Some(view_dialog::ViewDialog::from_parts(
                                name, body, truncated, size,
                            ));
                        }
                    }
                }
                Outcome::Summarized {
                    token,
                    path,
                    summary,
                } => {
                    if self.info_job == Some(token.id) {
                        self.info_job = None;
                        if token.generation == self.generation {
                            self.status.clear();
                            self.info_dialog = Some(info_dialog::InfoDialog::new(&path, summary));
                        }
                    }
                }
                Outcome::ReportListed { token, archives } => {
                    if self.report_job == Some(token.id) {
                        self.report_job = None;
                        if let Some(dialog) = &mut self.report_dialog {
                            if token.generation == self.generation {
                                dialog.set_archives(archives);
                            } else {
                                dialog.fail(
                                    "The folder changed before the report was ready.".to_string(),
                                );
                            }
                        }
                    }
                }
                Outcome::Failed { token, message } => {
                    if self.operation.as_ref().is_some_and(|op| op.id == token.id) {
                        let kind = self.operation.as_ref().map(|op| op.kind);
                        self.clear_operation();
                        self.ask = None;
                        self.status.clear();
                        if matches!(kind, Some(OpKind::Test)) {
                            self.test_dialog = None;
                        }
                        self.error = Some(message);
                    } else if self.browse_job == Some(token.id) {
                        self.browse_job = None;
                        self.listing = false;
                        if token.generation == self.generation {
                            self.status.clear();
                            self.error = Some(message);
                        }
                    } else if self.view_job == Some(token.id) {
                        self.view_job = None;
                        if token.generation == self.generation {
                            self.status.clear();
                            self.error = Some(message);
                        }
                    } else if self.info_job == Some(token.id) {
                        self.info_job = None;
                        if token.generation == self.generation {
                            self.status.clear();
                            self.error = Some(message);
                        }
                    } else if self.report_job == Some(token.id) {
                        self.report_job = None;
                        if token.generation == self.generation {
                            if let Some(dialog) = &mut self.report_dialog {
                                dialog.fail(message);
                            } else {
                                self.error = Some(message);
                            }
                        }
                    } else if self.update_job == Some(token.id) {
                        self.update_job = None;
                        self.status.clear();
                        self.error = Some(message);
                    }
                }
                Outcome::UpdateChecked {
                    token,
                    message,
                    is_error,
                } => {
                    if self.update_job == Some(token.id) {
                        self.update_job = None;
                        if is_error {
                            self.status.clear();
                            self.error = Some(message);
                        } else {
                            self.error = None;
                            self.status = message;
                        }
                    }
                }
                Outcome::NeedPassword(request) => self.finish_password(request),
            }
        }
        self.pump_shell_queue();
    }

    fn settle_side_status(&mut self, token: JobToken) {
        if self.view_job == Some(token.id) {
            self.view_job = None;
            if token.generation == self.generation {
                self.status.clear();
            }
        }
        if self.info_job == Some(token.id) {
            self.info_job = None;
            if token.generation == self.generation {
                self.status.clear();
            }
        }
    }

    fn finish_listed(
        &mut self,
        token: JobToken,
        path: PathBuf,
        format: String,
        entries: Vec<Entry>,
    ) {
        let kind = self
            .operation
            .as_ref()
            .filter(|op| op.id == token.id)
            .map(|op| op.kind);
        match listed_effect(kind, token.generation, self.generation) {
            ListedEffect::Finish { kind, open } => {
                self.clear_operation();
                self.ask = None;
                if self.shell_verb && kind == OpKind::Add {
                    self.status = format!("created {}", path.display());
                    self.maybe_close_shell_verb();
                } else if open {
                    self.apply_archive(path, format, entries);
                } else if kind == OpKind::Add {
                    self.status = format!("created {}", path.display());
                }
            }
            ListedEffect::Browse { open } => {
                if self.browse_job == Some(token.id) {
                    self.browse_job = None;
                    self.listing = false;
                }
                if open {
                    self.apply_archive(path, format, entries);
                }
            }
        }
    }

    fn finish_password(&mut self, mut request: PasswordRequest) {
        let matches_operation = self
            .operation
            .as_ref()
            .is_some_and(|op| op.id == request.id);
        let matches_browse = self.browse_job == Some(request.id);
        let matches_view = self.view_job == Some(request.id);
        if matches_operation && let Some(op) = &self.operation {
            request.foreground = op.foreground;
        }
        if matches_operation {
            self.clear_operation();
        }
        if matches_browse {
            self.browse_job = None;
            self.listing = false;
        }
        if matches_view {
            self.view_job = None;
            self.status.clear();
        }
        self.ask = None;
        let show = match request.job {
            PasswordJob::List | PasswordJob::View { .. } => request.generation == self.generation,
            PasswordJob::Extract(_)
            | PasswordJob::Test
            | PasswordJob::Delete { .. }
            | PasswordJob::Rename { .. } => true,
        };
        if (matches_operation || matches_browse || matches_view) && show {
            self.password_ask = Some(PasswordAsk {
                request,
                text: String::new(),
            });
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
        let mut submit = false;
        let mut cancel = false;
        let prompt = if ask.request.again {
            "That password was incorrect. Enter it again."
        } else {
            "Enter the password to open this archive."
        };
        let result = chrome::modal(ctx, "shellzip-password", "Password", |ui| {
            ui.set_min_width(360.0);
            ui.label(prompt);
            let field = ui.add(
                egui::TextEdit::singleline(&mut ask.text)
                    .password(true)
                    .desired_width(f32::INFINITY)
                    .id_salt("shellzip-open-password"),
            );
            field.request_focus();
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let colors = theme::palette(ui.ctx());
                    if chrome::labeled_button(ui, Icon::X, colors.ink, "Cancel").clicked() {
                        cancel = true;
                    }
                    if chrome::labeled_button(ui, Icon::Check, colors.ok, "OK").clicked()
                        || ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        submit = true;
                    }
                });
            });
        });
        if cancel || result.escape {
            let job = self.password_ask.take().map(|ask| ask.request.job);
            if matches!(job, Some(PasswordJob::Test)) {
                self.test_dialog = None;
            }
            if matches!(job, Some(PasswordJob::View { .. })) {
                self.view_job = None;
                self.status.clear();
            }
            if matches!(job, Some(PasswordJob::Extract(_))) {
                self.shell_queue.clear();
                self.maybe_close_shell_verb();
            }
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
            PasswordJob::Extract(job) => {
                let foreground = request.foreground;
                self.spawn_extract(request.path, job, Some(password), foreground);
            }
            PasswordJob::Delete { names } => self.spawn_delete(request.path, names, Some(password)),
            PasswordJob::Rename { from, to } => {
                self.spawn_rename(request.path, from, to, Some(password));
            }
            PasswordJob::View {
                member,
                label,
                size,
            } => {
                self.spawn_view_member(request.path, member, label, size, Some(password));
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
        if !folder_still_shown(generation, self.generation) {
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
        let prefix = match self.restore_prefix.take() {
            Some((generation, prefix))
                if generation == self.generation
                    && (prefix.is_empty() || index.contains_folder(&prefix)) =>
            {
                prefix
            }
            _ => String::new(),
        };
        self.address = location_text(&path, Some(&prefix));
        self.place = Place::Archive {
            path,
            format,
            index,
            members,
            prefix,
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
        if self.shell_locked() {
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

    fn refresh(&mut self) {
        if self.shell_locked() {
            return;
        }
        match &self.place {
            Place::Folder { path, .. } => {
                let path = path.clone();
                self.open_folder_inner(path, false);
            }
            Place::Archive { path, prefix, .. } => {
                let path = path.clone();
                let prefix = prefix.clone();
                self.start_list(path);
                self.restore_prefix = Some((self.generation, prefix));
            }
        }
    }

    fn go_up(&mut self) {
        if self.shell_locked() {
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
        if self.shell_locked() {
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
        if self.shell_locked() {
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

    fn can_view(&self) -> bool {
        if self.shell_locked() || self.operation.is_some() {
            return false;
        }
        let mut found = None;
        for row in &self.view {
            if row.kind == RowKind::Parent || !self.explorer.is_selected(&row.key) {
                continue;
            }
            if found.is_some() {
                return false;
            }
            found = Some(row);
        }
        found.is_some_and(|row| {
            !matches!(row.kind, RowKind::Directory)
                && !matches!(row.target, Target::Parent | Target::ArchiveDir(_))
        })
    }

    fn can_delete(&self) -> bool {
        if self.shell_locked() || self.operation.is_some() {
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

    fn tree_focus(&self) -> TreeFocus {
        match &self.place {
            Place::Folder { path, .. } => TreeFocus::folder(path),
            Place::Archive { path, prefix, .. } => TreeFocus::archive(path, prefix),
        }
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
            MenuCommand::View => self.view_file(),
            MenuCommand::SelectAll => self.explorer.select_all(&self.view),
            MenuCommand::SelectNone => self.explorer.clear_selection(),
            MenuCommand::InvertSelection => self.explorer.invert_selection(&self.view),
            MenuCommand::About => self.about = true,
            MenuCommand::Delete => self.request_delete(),
            MenuCommand::Rename => self.rename_selected(),
            MenuCommand::Info => self.show_info(),
            MenuCommand::Report => self.open_report(),
            MenuCommand::Exit => {}
            MenuCommand::OpenRecent(index) => {
                if let Some(path) = self.opened.get(index) {
                    self.start_list(PathBuf::from(path.clone()));
                }
            }
            MenuCommand::ClearHistory => self.clear_history(),
            MenuCommand::Theme(choice) => {
                self.theme = choice;
                self.persist_settings();
            }
            MenuCommand::ToolbarSize(size) => {
                self.toolbar = size;
                self.persist_settings();
            }
            MenuCommand::Integrate => {
                if !self.dialog_blocks_shell() {
                    self.pending_shell = Some(ShellIntegrateKind::Install);
                }
            }
            MenuCommand::Unintegrate => {
                if !self.dialog_blocks_shell() {
                    self.pending_shell = Some(ShellIntegrateKind::Remove);
                }
            }
            MenuCommand::CheckForUpdates => self.check_for_updates(),
        }
    }

    fn check_for_updates(&mut self) {
        if self.update_job.is_some() {
            self.status = "An update check is already running.".to_string();
            return;
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let token = JobToken {
            id,
            generation: self.generation,
        };
        self.update_job = Some(id);
        self.error = None;
        self.status = "Checking for updates…".to_string();
        let tx = self.tx.clone();
        worker::spawn(tx, token, move || {
            let result = crate::update::check_and_apply();
            Ok(Outcome::UpdateChecked {
                token,
                message: result.status_line(),
                is_error: result.is_error(),
            })
        });
    }

    fn finish_shell_integrate(&mut self, kind: ShellIntegrateKind) {
        match kind {
            ShellIntegrateKind::Install => match launch::run_integrate() {
                Ok(()) => self.status = "File manager integration installed.".to_string(),
                Err(err) => self.error = Some(err),
            },
            ShellIntegrateKind::Remove => match launch::run_unintegrate() {
                Ok(()) => self.status = "File manager integration removed.".to_string(),
                Err(err) => self.error = Some(err),
            },
        }
    }

    fn on_tool(&mut self, command: ToolCommand) {
        match command {
            ToolCommand::Open => self.open_archive(),
            ToolCommand::Add => self.create_archive(),
            ToolCommand::Extract => self.extract_archive(),
            ToolCommand::Test => self.test_archive(),
            ToolCommand::Delete => self.request_delete(),
            ToolCommand::Info => self.show_info(),
            ToolCommand::About => self.about = true,
            ToolCommand::Sfx => self.create_sfx(),
            ToolCommand::View => self.view_file(),
        }
    }

    fn on_address(&mut self, event: AddressEvent) {
        match event {
            AddressEvent::Up => self.go_up(),
            AddressEvent::Submit => self.open_address(),
            AddressEvent::Refresh => self.refresh(),
            AddressEvent::Jump(JumpTarget::Folder(path)) => self.open_folder(path),
            AddressEvent::Jump(JumpTarget::Prefix(prefix)) => self.set_prefix(prefix),
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F5)) {
            self.refresh();
        }
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
        if self.shell_locked() || self.dialog_blocks_shell() {
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
        let target = self.updatable_archive();
        self.open_add_dialog(dropped, target);
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

    fn remember_destination(&mut self, path: &Path) {
        let text = path.display().to_string();
        self.recent.retain(|item| item != &text);
        self.recent.insert(0, text);
        self.recent.truncate(12);
        self.persist_settings();
    }

    fn remember_opened(&mut self, path: &Path) {
        let text = path.display().to_string();
        self.opened.retain(|item| item != &text);
        self.opened.insert(0, text);
        self.opened.truncate(10);
        self.persist_settings();
    }

    fn clear_history(&mut self) {
        forget_history(&mut self.opened, &mut self.recent);
        self.persist_settings();
    }

    fn persist_settings(&mut self) {
        let stored = settings::StoredSettings {
            pack: self.options,
            extract: self.extract,
            sfx: self.sfx,
            recent: self.recent.clone(),
            opened: self.opened.clone(),
            theme: self.theme,
            toolbar: self.toolbar,
            report: self.report,
        };
        if let Err(err) = settings::save(&stored)
            && self.error.is_none()
        {
            self.error = Some(err);
        }
    }

    fn open_destination(&mut self, path: &Path) {
        #[cfg(windows)]
        {
            if let Err(err) = std::process::Command::new("explorer").arg(path).spawn()
                && self.error.is_none()
            {
                self.error = Some(format!("open {}: {err}", path.display()));
            }
        }
        #[cfg(not(windows))]
        {
            let _ = path;
        }
    }

    fn remove_archive(&mut self, path: &Path, recycle: bool) {
        let result = if recycle {
            trash::delete(path).map_err(|err| err.to_string())
        } else {
            std::fs::remove_file(path).map_err(|err| format!("delete {}: {err}", path.display()))
        };
        if let Err(err) = result
            && self.error.is_none()
        {
            self.error = Some(err);
        }
    }
}

fn needs_password(err: &Error) -> bool {
    matches!(err, Error::Encrypted(_) | Error::BadPassword)
}

fn preview_outcome(token: JobToken, name: String, data: PreviewData) -> Outcome {
    let truncated = data.truncated;
    let size = data.size;
    let body = view_dialog::classify(&data.bytes, truncated);
    Outcome::Previewed {
        token,
        name,
        body,
        truncated,
        size,
    }
}

fn report_row(row: &Row) -> ReportRow {
    ReportRow {
        name: row.name.clone(),
        size: row.size.unwrap_or(0),
        packed: row.packed.unwrap_or(0),
        modified: row.modified,
        crc: row.crc32,
    }
}

fn report_entry(entry: Entry) -> ReportRow {
    ReportRow {
        name: entry.path,
        size: entry.size.unwrap_or(0),
        packed: entry.packed_size.unwrap_or(0),
        modified: entry.modified,
        crc: entry.crc32,
    }
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

fn suggested_sfx_path(source: &Path) -> PathBuf {
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("archive");
    let ext = if cfg!(windows) { "exe" } else { "sfx" };
    let dest = parent.join(format!("{stem}.{ext}"));
    if dest == source {
        parent.join(format!("{stem}-sfx.{ext}"))
    } else {
        dest
    }
}

fn forget_history(opened: &mut Vec<String>, recent: &mut Vec<String>) {
    opened.clear();
    recent.clear();
}

impl eframe::App for ShellZipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();
        self.tree.poll();
        if self.error.is_none()
            && let Some(message) = self.tree.take_loader_error()
        {
            self.error = Some(message);
        }
        self.poll_ask();
        self.password_window(ctx);
        crate::icons::warm(ctx);
        if self.operation.is_some()
            || self.listing
            || !self.loaded
            || self.tree.is_loading()
            || self.icons.is_pending()
            || self.view_job.is_some()
            || self.info_job.is_some()
            || self.report_job.is_some()
        {
            ctx.request_repaint();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.title()));
        self.take_drop(ctx);

        let into_archive = self.updatable_archive().is_some();
        let mut menu_command = None;
        egui::TopBottomPanel::top("menu")
            .frame(chrome::bar_frame(ctx, 4, 0))
            .show(ctx, |ui| {
                menu_command =
                    chrome::menu_bar(ui, self.theme, self.toolbar, into_archive, &self.opened);
            });
        let blocked = self.dialog_blocks_shell();
        if !blocked {
            if matches!(menu_command, Some(MenuCommand::Exit)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if let Some(command) = menu_command {
                self.on_menu(command);
            }
        }

        let tool_state = ToolbarState {
            operation: self.operation.is_some(),
            locked: self.shell_locked() || blocked,
            archive_ready: self.archive_ready(),
            can_delete: self.can_delete(),
            can_view: self.can_view(),
            into_archive,
        };
        let mut tool_command = None;
        egui::TopBottomPanel::top("toolbar")
            .frame(chrome::bar_frame(ctx, 4, 1))
            .show(ctx, |ui| {
                tool_command = chrome::toolbar(ui, &tool_state, self.toolbar);
            });
        if !blocked && let Some(command) = tool_command {
            self.on_tool(command);
        }

        let can_up = self.can_go_up() && !self.shell_locked() && !blocked;
        let can_refresh = !self.shell_locked() && !blocked;
        let focus = self.tree_focus();
        let mut address_event = None;
        egui::TopBottomPanel::top("address")
            .frame(chrome::bar_frame(ctx, 6, 4))
            .show(ctx, |ui| {
                address_event = chrome::address_bar(
                    ui,
                    &mut self.address,
                    can_up,
                    can_refresh,
                    &mut self.tree,
                    &focus,
                    &mut self.icons,
                );
            });
        let submitted = matches!(address_event, Some(AddressEvent::Submit));
        if !blocked && let Some(event) = address_event {
            self.on_address(event);
        }

        let jump_list = self.address_jumps();
        let mut crumb = None;
        egui::TopBottomPanel::top("crumbs")
            .frame(chrome::bar_frame(ctx, 8, 2))
            .show_animated(ctx, self.inside_archive(), |ui| {
                crumb = chrome::breadcrumbs(ui, &jump_list);
            });
        if !blocked && let Some(event) = crumb {
            self.on_address(event);
        }

        let summary = self.summary();
        let note = self.note().to_string();
        let running = self.running_status();
        let mut status_event = chrome::StatusEvent {
            dismiss_error: false,
            cancel: false,
            toggle_pause: false,
            details: false,
        };
        egui::TopBottomPanel::bottom("status")
            .frame(chrome::bar_frame(ctx, 10, 4))
            .show(ctx, |ui| {
                status_event = chrome::status_bar(
                    ui,
                    &summary,
                    running.as_ref(),
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
            if self
                .operation
                .as_ref()
                .is_some_and(|op| op.kind == OpKind::Test)
            {
                self.test_dialog = None;
            }
            if self.ask.as_ref().is_some_and(|ask| ask.pending.is_some()) {
                self.answer_ask(AskReply::Cancel);
            }
        }
        if status_event.toggle_pause {
            let paused = self.progress.snapshot().paused;
            self.progress.set_paused(!paused);
        }
        if status_event.details
            && let Some(op) = &mut self.operation
        {
            op.foreground = true;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::palette(ctx).canvas))
            .show(ctx, |ui| {
                if let Some(index) = crate::explorer::show(
                    ui,
                    &mut self.explorer,
                    &mut self.view,
                    &mut self.scroll_to,
                    &mut self.icons,
                ) {
                    self.activate(index);
                }
            });

        about::show(ctx, &mut self.about);
        if let Some(InfoAction::Close) = self
            .info_dialog
            .as_mut()
            .and_then(|dialog| dialog.show(ctx))
        {
            self.info_dialog = None;
        }
        let close_view = self
            .view_dialog
            .as_mut()
            .is_some_and(|dialog| dialog.show(ctx));
        if close_view {
            self.view_dialog = None;
        }
        let rename_action = self
            .rename_dialog
            .as_mut()
            .and_then(|dialog| dialog.show(ctx));
        match rename_action {
            Some(RenameAction::Submit { target, leaf }) => {
                self.rename_dialog = None;
                self.apply_rename(target, leaf);
            }
            Some(RenameAction::Cancel) => self.rename_dialog = None,
            None => {}
        }
        let test_snapshot = self
            .operation
            .as_ref()
            .filter(|op| op.kind == OpKind::Test)
            .map(|_| self.progress.snapshot());
        let test_command = self
            .test_dialog
            .as_mut()
            .and_then(|dialog| dialog.show(ctx, test_snapshot.as_ref()));
        match test_command {
            Some(TestCommand::Cancel) => {
                self.progress.request_cancel();
                self.test_dialog = None;
                self.status = "Cancelling...".to_string();
                if self
                    .password_ask
                    .as_ref()
                    .is_some_and(|ask| matches!(ask.request.job, PasswordJob::Test))
                {
                    self.password_ask = None;
                }
            }
            Some(TestCommand::Close) => self.test_dialog = None,
            None => {}
        }
        let report_action = self
            .report_dialog
            .as_mut()
            .and_then(|dialog| dialog.show(ctx));
        match report_action {
            Some(ReportAction::Save(settings)) => {
                self.report = settings;
                self.persist_settings();
            }
            Some(ReportAction::Wrote(path)) => {
                self.status = format!("Wrote {}", path.display());
                self.report_dialog = None;
                self.report_job = None;
            }
            Some(ReportAction::Close) => {
                self.report_dialog = None;
                self.report_job = None;
            }
            None => {}
        }
        self.show_progress(ctx);
        let extract_action = if let Some(dialog) = &mut self.extract_dialog {
            extract_dialog::show(ctx, dialog, &mut self.icons)
        } else {
            None
        };
        match extract_action {
            Some(ExtractAction::Submit(request)) => {
                self.extract_dialog = None;
                let rest = std::mem::take(&mut self.shell_extract_to);
                if !rest.is_empty() {
                    let output = request.output.clone();
                    let show = request.show_in_explorer;
                    let jobs = rest
                        .into_iter()
                        .map(|archive| (archive, output.clone(), show))
                        .collect::<Vec<_>>();
                    self.shell_queue.extend(jobs);
                }
                self.start_extract(request);
            }
            Some(ExtractAction::Save(settings)) => {
                self.extract = settings;
                self.persist_settings();
            }
            Some(ExtractAction::Close) => {
                self.extract_dialog = None;
                self.shell_extract_to.clear();
                self.maybe_close_shell_verb();
            }
            None => {}
        }
        let sfx_action = self
            .sfx_dialog
            .as_mut()
            .and_then(|dialog| sfx_dialog::show(ctx, dialog));
        match sfx_action {
            Some(SfxAction::Submit(request)) => {
                self.sfx_dialog = None;
                self.start_sfx(request);
            }
            Some(SfxAction::Save(settings)) => {
                self.sfx = settings;
                self.persist_settings();
            }
            Some(SfxAction::Close) => self.sfx_dialog = None,
            None => {}
        }
        self.pump_shell_queue();
        let delete_choice = self.pending_archive_delete.as_mut().map(|pending| {
            let path = pending.path.clone();
            let recycle = pending.recycle;
            let confirmed = extract_dialog::confirm_delete_archive(ctx, &path, &mut pending.open);
            (confirmed, pending.open, path, recycle)
        });
        if let Some((confirmed, still_open, path, recycle)) = delete_choice {
            if confirmed {
                self.pending_archive_delete = None;
                self.remove_archive(&path, recycle);
                self.maybe_close_shell_verb();
            } else if !still_open {
                self.pending_archive_delete = None;
                self.maybe_close_shell_verb();
            }
        }
        let add_action = self
            .add_dialog
            .as_mut()
            .and_then(|dialog| add_dialog::show(ctx, dialog));
        match add_action {
            Some(AddAction::Submit(request)) => {
                self.add_dialog = None;
                self.start_add(request);
            }
            Some(AddAction::Close) => {
                self.add_dialog = None;
                self.maybe_close_shell_verb();
            }
            None => {}
        }
        let pending_ask = self
            .ask
            .as_ref()
            .and_then(|ask| ask.pending.clone().map(|name| (name, ask.detail)));
        if let Some((name, detail)) = pending_ask
            && let Some(reply) = add_dialog::ask_overwrite(ctx, &name, detail)
        {
            self.answer_ask(reply);
        }
        let delete_choice = self
            .pending_delete
            .as_deref()
            .map(|names| chrome::confirm_delete(ctx, names));
        match delete_choice {
            Some(DeleteChoice::Delete) => self.run_delete(),
            Some(DeleteChoice::Cancel) => self.pending_delete = None,
            Some(DeleteChoice::Wait) | None => {}
        }
        let shell_choice = self
            .pending_shell
            .map(|kind| (kind, chrome::confirm_shell_integrate(ctx, kind)));
        if let Some((kind, choice)) = shell_choice {
            match choice {
                ShellIntegrateChoice::Confirm => {
                    self.pending_shell = None;
                    self.finish_shell_integrate(kind);
                }
                ShellIntegrateChoice::Cancel => self.pending_shell = None,
                ShellIntegrateChoice::Wait => {}
            }
        }
        if self.options != self.saved {
            self.persist_settings();
            self.saved = self.options;
        }
        if !submitted && !self.dialog_blocks_shell() {
            self.shortcuts(ctx);
        }
        if self.close_after_shell {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

/// What to do with a listing that arrived while an archive job may still be running.
enum ListedEffect {
    /// The add or delete finished. Open the archive only when the user has not moved.
    Finish { kind: OpKind, open: bool },
    /// A browse listing. Show it when it is still the latest navigation.
    Browse { open: bool },
}

fn listed_effect(kind: Option<OpKind>, listed_generation: u64, current: u64) -> ListedEffect {
    match kind {
        Some(kind @ (OpKind::Add | OpKind::Delete | OpKind::Rename | OpKind::Sfx)) => {
            ListedEffect::Finish {
                kind,
                open: listed_generation == current,
            }
        }
        Some(OpKind::Extract | OpKind::Test) | None => ListedEffect::Browse {
            open: listed_generation == current,
        },
    }
}

/// A folder read applies only when the user is still looking at that generation.
fn folder_still_shown(listing_generation: u64, current: u64) -> bool {
    listing_generation == current
}

fn spawn_directory(tx: Sender<Outcome>, path: PathBuf, generation: u64) {
    let token = JobToken { id: 0, generation };
    worker::spawn(tx, token, move || {
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

/// True when a one-click shell add should open the dialog instead of packing.
///
/// That covers packing a file into itself, and overwriting a non-archive path.
fn quick_add_needs_dialog(inputs: &[PathBuf], dest: &Path, extension: &str) -> bool {
    if inputs.iter().any(|path| path == dest) {
        return true;
    }
    if !dest.exists() {
        return false;
    }
    !dest
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_operation_keeps_a_later_folder_listing() {
        assert!(folder_still_shown(4, 4));
        assert!(
            !folder_still_shown(2, 4),
            "an older folder read must not replace the place the user opened"
        );
        assert!(matches!(
            listed_effect(Some(OpKind::Add), 2, 4),
            ListedEffect::Finish { open: false, .. }
        ));
        assert!(matches!(
            listed_effect(Some(OpKind::Extract), 4, 4),
            ListedEffect::Browse { open: true }
        ));
        assert!(matches!(
            listed_effect(None, 4, 4),
            ListedEffect::Browse { open: true }
        ));
    }

    #[test]
    fn add_opens_the_archive_when_the_user_stayed() {
        assert!(matches!(
            listed_effect(Some(OpKind::Add), 2, 2),
            ListedEffect::Finish {
                kind: OpKind::Add,
                open: true
            }
        ));
    }

    #[test]
    fn quick_add_opens_dialog_for_self_or_foreign_file() {
        let notes = PathBuf::from("notes.zip");
        assert!(quick_add_needs_dialog(
            std::slice::from_ref(&notes),
            &notes,
            "zip"
        ));
        assert!(quick_add_needs_dialog(
            &[PathBuf::from("a.txt")],
            Path::new("a.txt"),
            "zip"
        ));
        assert!(!quick_add_needs_dialog(
            &[PathBuf::from("a.txt")],
            Path::new("missing.zip"),
            "zip"
        ));
    }

    #[test]
    fn sfx_suggestion_uses_exe_on_windows_and_avoids_the_source_path() {
        let source = PathBuf::from(if cfg!(windows) {
            r"C:\work\docs.exe"
        } else {
            "/work/docs.sfx"
        });
        let dest = suggested_sfx_path(&source);
        assert_ne!(dest, source);
        let name = dest.file_name().and_then(|name| name.to_str()).unwrap();
        assert!(name.contains("docs"));
        assert!(name.ends_with(if cfg!(windows) { ".exe" } else { ".sfx" }));
    }

    #[test]
    fn clear_history_forgets_opened_archives_and_destinations() {
        let mut opened = vec!["docs.zip".to_string()];
        let mut recent = vec![r"C:\work\out".to_string()];
        forget_history(&mut opened, &mut recent);
        assert!(opened.is_empty());
        assert!(recent.is_empty());
    }
}
