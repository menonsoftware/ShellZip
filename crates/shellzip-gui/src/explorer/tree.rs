//! Address-bar folder tree. Ancestors are known immediately; siblings load later.

use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use super::JumpTarget;
use super::listing::read_subdirs;
use crate::shell_icons::IconKey;

/// How the root of the dropdown is built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Namespace {
    /// Desktop, then This PC, then drive letters.
    Windows,
    /// Home, then mounts under Computer.
    Unix,
}

/// One line in the address dropdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    /// Stable identity for expand and highlight.
    pub id: NodeId,
    /// Text next to the icon.
    pub label: String,
    /// Indent step. Zero is a root.
    pub depth: u8,
    /// `None` for This PC, which only expands.
    pub target: Option<JumpTarget>,
    /// Shell picture for this node.
    pub icon: IconKey,
    /// Whether a caret is drawn.
    pub expandable: bool,
    /// Caret points down.
    pub expanded: bool,
    /// The place the file list is showing.
    pub current: bool,
}

/// Identity of a tree node.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeId {
    /// The user's desktop folder.
    Desktop,
    /// The user's home directory, on Unix.
    Home,
    /// Virtual computer: This PC, or mounts.
    Computer,
    /// A real directory.
    Dir(PathBuf),
    /// The open archive file.
    Archive(PathBuf),
    /// A folder inside the open archive. No trailing slash.
    Prefix {
        /// Archive file.
        archive: PathBuf,
        /// Archive-relative folder.
        prefix: String,
    },
}

/// Where the file list is, for highlighting and the ancestor chain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeFocus {
    /// Directory being listed, or the archive's parent.
    pub directory: PathBuf,
    /// Open archive, when the list is inside one.
    pub archive: Option<PathBuf>,
    /// Folder inside the archive. Empty is the archive root.
    pub prefix: String,
}

impl TreeFocus {
    /// A real directory.
    pub fn folder(path: &Path) -> Self {
        Self {
            directory: path.to_path_buf(),
            archive: None,
            prefix: String::new(),
        }
    }

    /// An open archive. `prefix` uses `/`.
    pub fn archive(path: &Path, prefix: &str) -> Self {
        let directory = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf());
        Self {
            directory,
            archive: Some(path.to_path_buf()),
            prefix: prefix.trim_matches('/').to_string(),
        }
    }
}

/// Desktop and home, when those folders exist.
#[derive(Clone, Debug, Default)]
pub struct Places {
    /// Desktop directory.
    pub desktop: Option<PathBuf>,
    /// Home directory.
    pub home: Option<PathBuf>,
}

impl Places {
    /// Folders reported by the OS.
    pub fn detect() -> Self {
        let dirs = directories::UserDirs::new();
        Self {
            desktop: dirs.as_ref().and_then(|dirs| {
                dirs.desktop_dir()
                    .filter(|path| path.is_dir())
                    .map(Path::to_path_buf)
            }),
            home: dirs
                .as_ref()
                .map(|dirs| dirs.home_dir().to_path_buf())
                .filter(|path| path.is_dir()),
        }
    }
}

/// A mounted volume.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Volume {
    /// `Local Disk (C:)` or the mount path.
    pub label: String,
    /// Filesystem root of the volume.
    pub path: PathBuf,
}

enum Job {
    Dirs(PathBuf),
    Volumes,
}

enum Msg {
    Dirs { path: PathBuf, dirs: Vec<PathBuf> },
    Volumes(Vec<Volume>),
}

/// Dropdown state: expansion, cached children, and the loader thread.
pub struct FolderTree {
    namespace: Namespace,
    places: Places,
    expanded: HashSet<NodeId>,
    children: HashMap<PathBuf, Vec<PathBuf>>,
    pending: HashSet<PathBuf>,
    volumes: Vec<Volume>,
    volumes_started: bool,
    volumes_ready: bool,
    loader_dead: bool,
    load_error: Option<String>,
    loader_reported: bool,
    tx: Sender<Job>,
    rx: Receiver<Msg>,
}

impl FolderTree {
    /// Detect places and start the directory loader.
    pub fn new() -> Self {
        Self::with_places(host_namespace(), Places::detect())
    }

    fn with_places(namespace: Namespace, places: Places) -> Self {
        let (tx, rx, failure) = spawn_loader();
        Self::with_channels(namespace, places, tx, rx, failure)
    }

    fn with_channels(
        namespace: Namespace,
        places: Places,
        tx: Sender<Job>,
        rx: Receiver<Msg>,
        failure: Option<String>,
    ) -> Self {
        let dead = failure.is_some();
        Self {
            namespace,
            places,
            expanded: HashSet::new(),
            children: HashMap::new(),
            pending: HashSet::new(),
            volumes: Vec::new(),
            volumes_started: dead,
            volumes_ready: dead,
            loader_dead: dead,
            load_error: failure,
            loader_reported: false,
            tx,
            rx,
        }
    }

    /// Apply finished directory and volume reads.
    pub fn poll(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Msg::Dirs { path, dirs } => {
                    self.pending.remove(&path);
                    self.children.insert(path, dirs);
                }
                Msg::Volumes(volumes) => {
                    self.volumes = volumes;
                    self.volumes_ready = true;
                }
            }
        }
        if !self.volumes_started && !self.loader_dead {
            self.volumes_started = true;
            if self.tx.send(Job::Volumes).is_err() {
                self.mark_loader_dead(LOADER_FAILED);
            }
        }
    }

    /// Why folder listing stopped, when the loader thread is gone.
    pub fn loader_error(&self) -> Option<&str> {
        self.load_error.as_deref()
    }

    /// The loader failure, once, for the status line.
    pub fn take_loader_error(&mut self) -> Option<String> {
        if self.loader_reported {
            return None;
        }
        let message = self.load_error.clone()?;
        self.loader_reported = true;
        Some(message)
    }

    /// True while a directory or the volume list is still outstanding.
    pub fn is_loading(&self) -> bool {
        !self.pending.is_empty() || !self.volumes_ready
    }

    /// Expand every ancestor of the current place.
    pub fn reveal(&mut self, focus: &TreeFocus) {
        let spine = spine(self.namespace, &self.places, focus, &self.volumes);
        if spine.len() > 1 {
            for id in &spine[..spine.len() - 1] {
                self.expanded.insert(id.clone());
            }
        }
    }

    /// Open or close one node.
    pub fn toggle(&mut self, id: &NodeId) {
        if !self.expanded.remove(id) {
            self.expanded.insert(id.clone());
        }
    }

    /// Visible rows. Expanded directories missing from the cache are requested.
    pub fn rows(&mut self, focus: &TreeFocus) -> Vec<TreeRow> {
        let layout = layout_rows(&LayoutInput {
            namespace: self.namespace,
            places: &self.places,
            focus,
            expanded: &self.expanded,
            children: &self.children,
            volumes: &self.volumes,
        });
        if !self.loader_dead {
            for path in layout.load {
                if self.children.contains_key(&path) || !self.pending.insert(path.clone()) {
                    continue;
                }
                if self.tx.send(Job::Dirs(path.clone())).is_err() {
                    self.mark_loader_dead(LOADER_FAILED);
                    break;
                }
            }
        }
        layout.rows
    }

    fn mark_loader_dead(&mut self, message: &str) {
        self.loader_dead = true;
        self.volumes_ready = true;
        self.pending.clear();
        if self.load_error.is_none() {
            self.load_error = Some(message.to_string());
        }
    }
}

const LOADER_FAILED: &str = "Could not list folders.";

fn spawn_loader() -> (Sender<Job>, Receiver<Msg>, Option<String>) {
    let (tx, request_rx) = mpsc::channel();
    let (result_tx, rx) = mpsc::channel();
    let started = std::thread::Builder::new()
        .name("shellzip-tree".to_string())
        .spawn(move || {
            while let Ok(job) = request_rx.recv() {
                let msg = match job {
                    Job::Dirs(path) => Msg::Dirs {
                        dirs: read_subdirs(&path),
                        path,
                    },
                    Job::Volumes => Msg::Volumes(list_volumes()),
                };
                if result_tx.send(msg).is_err() {
                    break;
                }
            }
        });
    let failure = match started {
        // Dropping the handle detaches the thread. It exits when `tx` is dropped.
        Ok(_) => None,
        // The closure was dropped with the error, so both channels are disconnected.
        Err(err) => Some(format!("Could not list folders: {err}")),
    };
    (tx, rx, failure)
}

/// Fixed disks and real mounts. Removable disks and pseudo filesystems are left out.
pub fn list_volumes() -> Vec<Volume> {
    let Ok(disks) = sysinfo::Disks::new_with_refreshed_list() else {
        return Vec::new();
    };
    let mut volumes = Vec::new();
    for disk in &disks {
        if disk.is_removable() || is_pseudo(disk.file_system()) {
            continue;
        }
        let path = disk.mount_point().to_path_buf();
        if path.as_os_str().is_empty()
            || volumes
                .iter()
                .any(|volume: &Volume| same_path(&volume.path, &path))
        {
            continue;
        }
        volumes.push(Volume {
            label: volume_label(disk.name(), &path),
            path,
        });
    }
    volumes.sort_by(|left, right| cmp_name(&left.path, &right.path));
    volumes
}

fn is_pseudo(fs: &std::ffi::OsStr) -> bool {
    let text = fs.to_string_lossy();
    matches!(
        text.as_ref(),
        "proc"
            | "sysfs"
            | "devtmpfs"
            | "tmpfs"
            | "cgroup"
            | "cgroup2"
            | "devpts"
            | "securityfs"
            | "pstore"
            | "bpf"
            | "autofs"
            | "configfs"
            | "debugfs"
            | "tracefs"
            | "fusectl"
            | "overlay"
            | "squashfs"
            | "ramfs"
            | "binfmt_misc"
            | "mqueue"
            | "hugetlbfs"
            | "nsfs"
            | "efivarfs"
            | "rpc_pipefs"
    )
}

fn volume_label(name: &std::ffi::OsStr, mount: &Path) -> String {
    let name = name.to_string_lossy();
    let name = name.trim();
    let point = mount_text(mount);
    if name.is_empty() || name.eq_ignore_ascii_case(&point) {
        point
    } else {
        format!("{name} ({point})")
    }
}

fn mount_text(path: &Path) -> String {
    let text = path.display().to_string();
    let trimmed = text.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        text
    } else {
        trimmed.to_string()
    }
}

/// Namespace for this operating system.
pub fn host_namespace() -> Namespace {
    if cfg!(windows) {
        Namespace::Windows
    } else {
        Namespace::Unix
    }
}

struct LayoutInput<'a> {
    namespace: Namespace,
    places: &'a Places,
    focus: &'a TreeFocus,
    expanded: &'a HashSet<NodeId>,
    children: &'a HashMap<PathBuf, Vec<PathBuf>>,
    volumes: &'a [Volume],
}

struct Layout {
    rows: Vec<TreeRow>,
    load: Vec<PathBuf>,
}

struct Builder<'a> {
    input: &'a LayoutInput<'a>,
    spine: Vec<NodeId>,
    volumes: Vec<Volume>,
    rows: Vec<TreeRow>,
    load: Vec<PathBuf>,
}

fn layout_rows(input: &LayoutInput<'_>) -> Layout {
    let volumes = volumes_for(input.volumes, &input.focus.directory);
    let spine = spine(input.namespace, input.places, input.focus, &volumes);
    let mut builder = Builder {
        input,
        spine,
        volumes,
        rows: Vec::new(),
        load: Vec::new(),
    };
    builder.emit_roots();
    Layout {
        rows: builder.rows,
        load: builder.load,
    }
}

fn volumes_for(stored: &[Volume], directory: &Path) -> Vec<Volume> {
    let mut volumes = stored.to_vec();
    if let Some(root) = drive_root(directory)
        && !volumes.iter().any(|volume| same_path(&volume.path, &root))
    {
        volumes.push(Volume {
            label: mount_text(&root),
            path: root,
        });
    }
    volumes.sort_by(|left, right| cmp_name(&left.path, &right.path));
    volumes
}

fn spine(
    namespace: Namespace,
    places: &Places,
    focus: &TreeFocus,
    volumes: &[Volume],
) -> Vec<NodeId> {
    let mut spine = match namespace {
        Namespace::Windows => windows_spine(places, &focus.directory),
        Namespace::Unix => unix_spine(places, &focus.directory, volumes),
    };
    push_archive(&mut spine, focus);
    spine
}

fn windows_spine(places: &Places, directory: &Path) -> Vec<NodeId> {
    let mut spine = Vec::new();
    if let Some(desktop) = &places.desktop {
        spine.push(NodeId::Desktop);
        if path_starts_with(directory, desktop) {
            push_relative(&mut spine, desktop, directory);
            return spine;
        }
    }
    spine.push(NodeId::Computer);
    if let Some(root) = drive_root(directory) {
        spine.push(NodeId::Dir(root.clone()));
        push_relative(&mut spine, &root, directory);
    }
    spine
}

fn unix_spine(places: &Places, directory: &Path, volumes: &[Volume]) -> Vec<NodeId> {
    let mut spine = Vec::new();
    if let Some(home) = &places.home
        && path_starts_with(directory, home)
    {
        spine.push(NodeId::Home);
        push_relative(&mut spine, home, directory);
        return spine;
    }
    if let Some(desktop) = &places.desktop
        && path_starts_with(directory, desktop)
    {
        spine.push(NodeId::Desktop);
        push_relative(&mut spine, desktop, directory);
        return spine;
    }
    spine.push(NodeId::Computer);
    let root = best_volume(volumes, directory).or_else(|| drive_root(directory));
    if let Some(root) = root {
        spine.push(NodeId::Dir(root.clone()));
        push_relative(&mut spine, &root, directory);
    }
    spine
}

fn push_relative(spine: &mut Vec<NodeId>, base: &Path, full: &Path) {
    for path in extend_from(base, full) {
        spine.push(NodeId::Dir(path));
    }
}

fn push_archive(spine: &mut Vec<NodeId>, focus: &TreeFocus) {
    let Some(archive) = &focus.archive else {
        return;
    };
    spine.push(NodeId::Archive(archive.clone()));
    let mut acc = String::new();
    for part in focus.prefix.split('/').filter(|part| !part.is_empty()) {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        spine.push(NodeId::Prefix {
            archive: archive.clone(),
            prefix: acc.clone(),
        });
    }
}

fn best_volume(volumes: &[Volume], path: &Path) -> Option<PathBuf> {
    volumes
        .iter()
        .filter(|volume| path_starts_with(path, &volume.path))
        .max_by_key(|volume| component_len(&volume.path))
        .map(|volume| volume.path.clone())
}

fn component_len(path: &Path) -> usize {
    path.components().count()
}

impl Builder<'_> {
    fn emit_roots(&mut self) {
        match self.input.namespace {
            Namespace::Windows => {
                if self.input.places.desktop.is_some() {
                    self.emit(NodeId::Desktop, 0);
                } else {
                    self.emit(NodeId::Computer, 0);
                }
            }
            Namespace::Unix => {
                if self.input.places.home.is_some() {
                    self.emit(NodeId::Home, 0);
                }
                if let Some(desktop) = &self.input.places.desktop {
                    let nested = self
                        .input
                        .places
                        .home
                        .as_ref()
                        .is_some_and(|home| path_starts_with(desktop, home));
                    if !nested {
                        self.emit(NodeId::Desktop, 0);
                    }
                }
                self.emit(NodeId::Computer, 0);
            }
        }
    }

    fn emit(&mut self, id: NodeId, depth: u8) {
        let expanded = self.is_expanded(&id);
        let current = self.spine.last().is_some_and(|item| same_id(item, &id));
        let (label, target, icon) = self.describe(&id);
        self.rows.push(TreeRow {
            id: id.clone(),
            label,
            depth,
            target,
            icon,
            expandable: true,
            expanded,
            current,
        });
        if expanded {
            self.emit_children(&id, depth);
        }
    }

    fn emit_children(&mut self, id: &NodeId, depth: u8) {
        let child_depth = depth.saturating_add(1);
        match id {
            NodeId::Desktop => {
                self.emit(NodeId::Computer, child_depth);
                if let Some(path) = self.input.places.desktop.clone() {
                    self.emit_dir_children(&path, depth);
                }
            }
            NodeId::Home => {
                if let Some(path) = self.input.places.home.clone() {
                    self.emit_dir_children(&path, depth);
                }
            }
            NodeId::Computer => {
                let volumes = self.volumes.clone();
                for volume in volumes {
                    self.emit(NodeId::Dir(volume.path), child_depth);
                }
            }
            NodeId::Dir(path) => self.emit_dir_children(path, depth),
            NodeId::Archive(_) | NodeId::Prefix { .. } => self.emit_next_prefix(id, child_depth),
        }
    }

    fn emit_dir_children(&mut self, path: &Path, depth: u8) {
        self.note_load(path);
        let mut nodes: Vec<NodeId> = self
            .dir_children(path)
            .into_iter()
            .map(NodeId::Dir)
            .collect();
        if let Some(archive) = &self.input.focus.archive
            && archive
                .parent()
                .is_some_and(|parent| same_path(parent, path))
        {
            nodes.push(NodeId::Archive(archive.clone()));
        }
        nodes.sort_by(|left, right| {
            self.describe(left)
                .0
                .to_lowercase()
                .cmp(&self.describe(right).0.to_lowercase())
        });
        let child_depth = depth.saturating_add(1);
        for node in nodes {
            self.emit(node, child_depth);
        }
    }

    fn dir_children(&self, path: &Path) -> Vec<PathBuf> {
        let mut kids = self
            .input
            .children
            .iter()
            .find(|(key, _)| same_path(key, path))
            .map(|(_, kids)| kids.clone())
            .unwrap_or_default();
        if let Some(next) = self.spine_child_dir(path)
            && !kids.iter().any(|kid| same_path(kid, &next))
        {
            kids.push(next);
        }
        kids.sort_by(|left, right| cmp_name(left, right));
        kids
    }

    fn spine_child_dir(&self, parent: &Path) -> Option<PathBuf> {
        let index = self.spine.iter().position(|id| match id {
            NodeId::Dir(path) => same_path(path, parent),
            NodeId::Desktop => self
                .input
                .places
                .desktop
                .as_ref()
                .is_some_and(|desktop| same_path(desktop, parent)),
            NodeId::Home => self
                .input
                .places
                .home
                .as_ref()
                .is_some_and(|home| same_path(home, parent)),
            _ => false,
        })?;
        let NodeId::Dir(path) = self.spine.get(index + 1)? else {
            return None;
        };
        path.parent()
            .is_some_and(|above| same_path(above, parent))
            .then(|| path.clone())
    }

    fn emit_next_prefix(&mut self, id: &NodeId, depth: u8) {
        let index = self.spine.iter().position(|item| same_id(item, id));
        let Some(index) = index else {
            return;
        };
        if let Some(NodeId::Prefix { archive, prefix }) = self.spine.get(index + 1) {
            self.emit(
                NodeId::Prefix {
                    archive: archive.clone(),
                    prefix: prefix.clone(),
                },
                depth,
            );
        }
    }

    fn note_load(&mut self, path: &Path) {
        let known = self.input.children.keys().any(|key| same_path(key, path));
        if known || self.load.iter().any(|item| same_path(item, path)) {
            return;
        }
        self.load.push(path.to_path_buf());
    }

    fn is_expanded(&self, id: &NodeId) -> bool {
        self.input.expanded.iter().any(|item| same_id(item, id))
    }

    fn describe(&self, id: &NodeId) -> (String, Option<JumpTarget>, IconKey) {
        match id {
            NodeId::Desktop => {
                let path = self
                    .input
                    .places
                    .desktop
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("Desktop"));
                (
                    "Desktop".to_string(),
                    Some(JumpTarget::Folder(path.clone())),
                    IconKey::Path(path),
                )
            }
            NodeId::Home => {
                let path = self
                    .input
                    .places
                    .home
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("Home"));
                (
                    "Home".to_string(),
                    Some(JumpTarget::Folder(path.clone())),
                    IconKey::Path(path),
                )
            }
            NodeId::Computer => (
                computer_label(self.input.namespace).to_string(),
                None,
                IconKey::Computer,
            ),
            NodeId::Dir(path) => {
                let label = self
                    .volumes
                    .iter()
                    .find(|volume| same_path(&volume.path, path))
                    .map(|volume| volume.label.clone())
                    .unwrap_or_else(|| file_label(path));
                (
                    label,
                    Some(JumpTarget::Folder(path.clone())),
                    IconKey::Path(path.clone()),
                )
            }
            NodeId::Archive(path) => (
                file_label(path),
                Some(JumpTarget::Prefix(String::new())),
                IconKey::Folder,
            ),
            NodeId::Prefix { prefix, .. } => (
                prefix_label(prefix),
                Some(JumpTarget::Prefix(prefix.clone())),
                IconKey::Folder,
            ),
        }
    }
}

fn computer_label(namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Windows => "This PC",
        Namespace::Unix => "Computer",
    }
}

/// Last folder in an archive prefix.
///
/// `rsplit` always yields at least one segment, including `""` for an empty
/// prefix, so a blank segment falls back to the whole prefix.
fn prefix_label(prefix: &str) -> String {
    prefix
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .unwrap_or(prefix)
        .to_string()
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| mount_text(path))
}

fn drive_root(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    match components.next()? {
        Component::Prefix(prefix) => {
            let mut root = PathBuf::from(prefix.as_os_str());
            root.push(std::path::MAIN_SEPARATOR.to_string());
            Some(root)
        }
        Component::RootDir => Some(PathBuf::from(std::path::MAIN_SEPARATOR.to_string())),
        _ => None,
    }
}

fn extend_from(base: &Path, full: &Path) -> Vec<PathBuf> {
    let full_parts = parts(full);
    let base_parts = parts(base);
    if base_parts.len() > full_parts.len() {
        return Vec::new();
    }
    let matches = full_parts
        .iter()
        .zip(&base_parts)
        .all(|(left, right)| left.eq_ignore_ascii_case(right));
    if !matches {
        return Vec::new();
    }
    let mut acc = base.to_path_buf();
    let mut out = Vec::new();
    for part in &full_parts[base_parts.len()..] {
        if part.is_empty() {
            continue;
        }
        acc.push(part);
        out.push(acc.clone());
    }
    out
}

fn parts(path: &Path) -> Vec<OsString> {
    path.components()
        .map(|component| component.as_os_str().to_os_string())
        .collect()
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    let path = parts(path);
    let prefix = parts(prefix);
    if prefix.len() > path.len() {
        return false;
    }
    path.iter()
        .zip(&prefix)
        .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

fn same_path(left: &Path, right: &Path) -> bool {
    let left = parts(left);
    let right = parts(right);
    left.len() == right.len()
        && left
            .iter()
            .zip(&right)
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn same_id(left: &NodeId, right: &NodeId) -> bool {
    match (left, right) {
        (NodeId::Desktop, NodeId::Desktop)
        | (NodeId::Home, NodeId::Home)
        | (NodeId::Computer, NodeId::Computer) => true,
        (NodeId::Dir(left), NodeId::Dir(right))
        | (NodeId::Archive(left), NodeId::Archive(right)) => same_path(left, right),
        (
            NodeId::Prefix {
                archive: left,
                prefix: left_prefix,
            },
            NodeId::Prefix {
                archive: right,
                prefix: right_prefix,
            },
        ) => same_path(left, right) && left_prefix == right_prefix,
        _ => false,
    }
}

fn cmp_name(left: &Path, right: &Path) -> std::cmp::Ordering {
    file_label(left)
        .to_lowercase()
        .cmp(&file_label(right).to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_label_keeps_the_full_prefix_when_the_segment_is_blank() {
        let cases = [
            ("docs/api", "api"),
            ("docs", "docs"),
            ("docs/", "docs/"),
            ("", ""),
            ("/", "/"),
        ];
        for (prefix, want) in cases {
            assert_eq!(prefix_label(prefix), want, "{prefix:?}");
        }
    }

    #[test]
    fn lists_at_least_one_volume() {
        let volumes = list_volumes();
        assert!(!volumes.is_empty(), "expected a fixed disk or a real mount");
    }

    #[test]
    fn drive_root_is_the_filesystem_root() {
        let (root, nested) = if cfg!(windows) {
            (
                PathBuf::from(r"C:\"),
                PathBuf::from(r"C:\Users\Administrator"),
            )
        } else {
            (PathBuf::from("/"), PathBuf::from("/home/user"))
        };
        let got = drive_root(&nested).expect("nested path has a root");
        assert!(same_path(&got, &root), "{got:?} vs {root:?}");
    }

    #[cfg(windows)]
    #[test]
    fn expands_to_the_current_folder_and_lists_siblings() {
        let desktop = PathBuf::from(r"C:\Users\Administrator\Desktop");
        let drive = PathBuf::from(r"C:\");
        let admin = PathBuf::from(r"C:\Users\Administrator");
        let downloads = admin.join("Downloads");
        let programs = admin.join("Programs");
        let places = Places {
            desktop: Some(desktop),
            home: Some(admin.clone()),
        };
        let focus = TreeFocus::folder(&downloads);
        let volumes = vec![Volume {
            label: "Local Disk (C:)".to_string(),
            path: drive,
        }];
        let spine = spine(Namespace::Windows, &places, &focus, &volumes);
        let expanded = spine[..spine.len() - 1].iter().cloned().collect();
        let mut children = HashMap::new();
        children.insert(admin, vec![programs, downloads.clone()]);
        let layout = layout_rows(&LayoutInput {
            namespace: Namespace::Windows,
            places: &places,
            focus: &focus,
            expanded: &expanded,
            children: &children,
            volumes: &volumes,
        });
        let labels: Vec<_> = layout.rows.iter().map(|row| row.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Desktop",
                "This PC",
                "Local Disk (C:)",
                "Users",
                "Administrator",
                "Downloads",
                "Programs",
            ]
        );
        assert!(
            layout
                .rows
                .iter()
                .any(|row| row.label == "Downloads" && row.current)
        );
        assert!(
            layout
                .rows
                .iter()
                .any(|row| row.label == "Programs" && !row.current)
        );
    }

    #[cfg(unix)]
    #[test]
    fn expands_home_and_lists_siblings() {
        let home = PathBuf::from("/home/user");
        let src = home.join("src");
        let docs = home.join("docs");
        let places = Places {
            desktop: Some(home.join("Desktop")),
            home: Some(home.clone()),
        };
        let focus = TreeFocus::folder(&src);
        let spine = spine(Namespace::Unix, &places, &focus, &[]);
        let expanded = spine[..spine.len() - 1].iter().cloned().collect();
        let mut children = HashMap::new();
        children.insert(home, vec![docs, src.clone()]);
        let layout = layout_rows(&LayoutInput {
            namespace: Namespace::Unix,
            places: &places,
            focus: &focus,
            expanded: &expanded,
            children: &children,
            volumes: &[],
        });
        let labels: Vec<_> = layout.rows.iter().map(|row| row.label.as_str()).collect();
        assert_eq!(labels, ["Home", "docs", "src", "Computer"]);
        assert!(
            layout
                .rows
                .iter()
                .any(|row| row.label == "src" && row.current)
        );
    }

    #[test]
    fn archive_prefixes_hang_under_the_archive() {
        let (parent, archive) = if cfg!(windows) {
            (PathBuf::from(r"C:\work"), PathBuf::from(r"C:\work\app.zip"))
        } else {
            (PathBuf::from("/work"), PathBuf::from("/work/app.zip"))
        };
        let places = Places {
            desktop: None,
            home: None,
        };
        let focus = TreeFocus::archive(&archive, "docs/api");
        let namespace = host_namespace();
        let volumes = vec![Volume {
            label: "disk".to_string(),
            path: drive_root(&parent).expect("parent has a root"),
        }];
        let spine = spine(namespace, &places, &focus, &volumes);
        let expanded = spine[..spine.len() - 1].iter().cloned().collect();
        let layout = layout_rows(&LayoutInput {
            namespace,
            places: &places,
            focus: &focus,
            expanded: &expanded,
            children: &HashMap::new(),
            volumes: &volumes,
        });
        let labels: Vec<_> = layout.rows.iter().map(|row| row.label.as_str()).collect();
        assert!(labels.contains(&"app.zip"), "{labels:?}");
        assert!(labels.contains(&"docs"), "{labels:?}");
        assert!(labels.contains(&"api"), "{labels:?}");
        assert!(
            layout
                .rows
                .iter()
                .any(|row| row.label == "api" && row.current)
        );
        assert_eq!(focus.directory, parent);
    }

    #[test]
    fn failed_spawn_does_not_queue_folder_reads() {
        let (tx, request_rx) = mpsc::channel();
        let (_result_tx, rx) = mpsc::channel();
        let home = PathBuf::from(if cfg!(windows) {
            r"C:\Users\Administrator"
        } else {
            "/home/user"
        });
        let mut tree = FolderTree::with_channels(
            Namespace::Unix,
            Places {
                desktop: None,
                home: Some(home.clone()),
            },
            tx,
            rx,
            Some(LOADER_FAILED.to_string()),
        );
        let focus = TreeFocus::folder(&home.join("src"));
        tree.poll();
        tree.reveal(&focus);
        let _rows = tree.rows(&focus);
        tree.poll();
        assert!(!tree.is_loading());
        assert_eq!(tree.loader_error(), Some(LOADER_FAILED));
        assert_eq!(tree.take_loader_error().as_deref(), Some(LOADER_FAILED));
        assert_eq!(tree.take_loader_error(), None);
        assert!(
            matches!(request_rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "a failed loader must not send directory or volume jobs"
        );
    }

    #[test]
    fn loader_thread_lists_a_sibling_directory() {
        let root = std::env::temp_dir().join(format!(
            "shellzip-tree-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock before unix epoch")
                .as_nanos()
        ));
        let _cleanup = RemoveOnDrop(root.clone());
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::create_dir_all(root.join("notes")).unwrap();
        let mut tree = FolderTree::with_places(
            Namespace::Unix,
            Places {
                desktop: None,
                home: Some(root.clone()),
            },
        );
        let focus = TreeFocus::folder(&root.join("docs"));
        tree.reveal(&focus);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut saw_notes = false;
        while std::time::Instant::now() < deadline {
            tree.poll();
            if tree.rows(&focus).iter().any(|row| row.label == "notes") {
                saw_notes = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            saw_notes,
            "loader thread did not list the sibling directory: {:?}",
            tree.loader_error()
        );
    }

    struct RemoveOnDrop(PathBuf);

    impl Drop for RemoveOnDrop {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
