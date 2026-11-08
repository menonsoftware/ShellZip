//! Cascaded context-menu model for Explorer and desktop actions.

use std::path::{Path, PathBuf};

use crate::formats::icon_for_path;
use crate::stem::folder_stem;

/// One verb the shell extension or desktop action exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Open the add dialog for the selection.
    Add,
    /// Pack immediately as ZIP with saved settings.
    AddZip,
    /// Pack immediately as 7z with saved settings.
    Add7z,
    /// Open the archive in the ShellZip window.
    Open,
    /// Show the extract dialog.
    ExtractTo,
    /// Extract into the archive's parent directory.
    ExtractHere,
    /// Extract into a new folder named after the archive stem.
    ExtractHereNamed,
}

/// One entry in the cascaded ShellZip submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuRow {
    /// A clickable command.
    Command {
        /// Verb to invoke.
        action: MenuAction,
        /// Caption shown in the submenu.
        label: String,
    },
    /// Visual break between add and extract groups.
    Separator,
}

/// Labels and rows for the current Explorer selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuModel {
    /// Ordered submenu rows, including a separator when extract items follow.
    pub rows: Vec<MenuRow>,
    /// Every selected path; used by add verbs.
    pub selection: Vec<PathBuf>,
    /// Selected paths that look like registered archives; used by extract/open.
    pub archives: Vec<PathBuf>,
    /// Suggested ZIP destination next to the selection.
    pub add_zip_path: PathBuf,
    /// Suggested 7z destination next to the selection.
    pub add_7z_path: PathBuf,
    /// Caption for the named extract item (Linux static menus still use
    /// [`linux_named_label`]).
    pub extract_here_named_label: String,
    /// When true, `--extract-here-named` creates one folder per archive.
    pub separate_folders: bool,
}

/// Build the cascaded menu model for the selected paths.
pub fn menu_for_selection(paths: &[impl AsRef<Path>]) -> MenuModel {
    let selection: Vec<PathBuf> = paths.iter().map(|p| p.as_ref().to_path_buf()).collect();
    let archives: Vec<PathBuf> = selection
        .iter()
        .filter(|path| is_registered_archive(path))
        .cloned()
        .collect();
    let add_zip_path = suggested_archive_path(&selection, "zip");
    let add_7z_path = suggested_archive_path(&selection, "7z");
    let (extract_here_named_label, separate_folders) = named_extract_caption(&archives);

    let mut rows = Vec::new();
    if !selection.is_empty() {
        rows.push(MenuRow::Command {
            action: MenuAction::Add,
            label: "Add to archive...".to_string(),
        });
        rows.push(MenuRow::Command {
            action: MenuAction::AddZip,
            label: format!("Add to {}", file_name_label(&add_zip_path)),
        });
        rows.push(MenuRow::Command {
            action: MenuAction::Add7z,
            label: format!("Add to {}", file_name_label(&add_7z_path)),
        });
    }
    if !archives.is_empty() {
        if !rows.is_empty() {
            rows.push(MenuRow::Separator);
        }
        rows.push(MenuRow::Command {
            action: MenuAction::Open,
            label: action_label(MenuAction::Open).to_string(),
        });
        rows.push(MenuRow::Command {
            action: MenuAction::ExtractTo,
            label: "Extract files...".to_string(),
        });
        rows.push(MenuRow::Command {
            action: MenuAction::ExtractHere,
            label: action_label(MenuAction::ExtractHere).to_string(),
        });
        rows.push(MenuRow::Command {
            action: MenuAction::ExtractHereNamed,
            label: extract_here_named_label.clone(),
        });
    }

    MenuModel {
        rows,
        selection,
        archives,
        add_zip_path,
        add_7z_path,
        extract_here_named_label,
        separate_folders,
    }
}

/// Paths a menu action should pass to `shellzip`.
pub fn paths_for_action(model: &MenuModel, action: MenuAction) -> &[PathBuf] {
    match action {
        MenuAction::Add | MenuAction::AddZip | MenuAction::Add7z => &model.selection,
        MenuAction::Open
        | MenuAction::ExtractTo
        | MenuAction::ExtractHere
        | MenuAction::ExtractHereNamed => &model.archives,
    }
}

/// True when `path` matches a registered archive extension, including compounds.
pub fn is_registered_archive(path: &Path) -> bool {
    icon_for_path(path).is_some()
}

/// Destination folder / archive name for a quick-add of `extension`.
pub fn suggested_archive_path(paths: &[impl AsRef<Path>], extension: &str) -> PathBuf {
    let dir = selection_dir(paths);
    let stem = if paths.len() == 1 {
        paths[0]
            .as_ref()
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty())
    } else {
        None
    };
    let stem = stem.unwrap_or_else(|| {
        dir.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Archive".to_string())
    });
    dir.join(format!("{stem}.{extension}"))
}

/// Parent directory of the first selected path, or `.`.
pub fn selection_dir(paths: &[impl AsRef<Path>]) -> PathBuf {
    paths
        .first()
        .and_then(|path| path.as_ref().parent())
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Static caption for data-only Linux menus that cannot put the stem in the label.
pub fn linux_named_label() -> &'static str {
    "Extract Here (folder)"
}

/// Human label for a fixed menu action (Linux / static menus).
pub fn action_label(action: MenuAction) -> &'static str {
    match action {
        MenuAction::Add => "Add to archive...",
        MenuAction::AddZip => "Add to ZIP",
        MenuAction::Add7z => "Add to 7z",
        MenuAction::Open => "Open",
        MenuAction::ExtractTo => "Extract to…",
        MenuAction::ExtractHere => "Extract Here",
        MenuAction::ExtractHereNamed => "Extract Here (folder)",
    }
}

/// CLI flag for a menu action.
pub fn action_flag(action: MenuAction) -> &'static str {
    match action {
        MenuAction::Add => "--add",
        MenuAction::AddZip => "--add-zip",
        MenuAction::Add7z => "--add-7z",
        MenuAction::Open => "",
        MenuAction::ExtractTo => "--extract-to",
        MenuAction::ExtractHere => "--extract-here",
        MenuAction::ExtractHereNamed => "--extract-here-named",
    }
}

/// Canonical verb string for `IContextMenu::GetCommandString`.
pub fn action_verb(action: MenuAction) -> &'static str {
    match action {
        MenuAction::Add => "shellzip.add",
        MenuAction::AddZip => "shellzip.addzip",
        MenuAction::Add7z => "shellzip.add7z",
        MenuAction::Open => "shellzip.open",
        MenuAction::ExtractTo => "shellzip.extract",
        MenuAction::ExtractHere => "shellzip.extracthere",
        MenuAction::ExtractHereNamed => "shellzip.extractto",
    }
}

fn named_extract_caption(archives: &[PathBuf]) -> (String, bool) {
    match archives {
        [] => ("Extract Here (folder)".to_string(), false),
        [one] => {
            let stem = folder_stem(one);
            (format!("Extract to \"{stem}\\\""), false)
        }
        _ => ("Extract each archive to separate folder".to_string(), true),
    }
}

fn file_name_label(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_file_shows_add_only() {
        let model = menu_for_selection(&[PathBuf::from(r"D:\docs\notes.txt")]);
        assert!(model.archives.is_empty());
        assert_eq!(model.rows.len(), 3);
        assert_eq!(
            model.rows[0],
            MenuRow::Command {
                action: MenuAction::Add,
                label: "Add to archive...".to_string(),
            }
        );
        assert_eq!(
            model.rows[1],
            MenuRow::Command {
                action: MenuAction::AddZip,
                label: "Add to notes.zip".to_string(),
            }
        );
        assert_eq!(
            model.rows[2],
            MenuRow::Command {
                action: MenuAction::Add7z,
                label: "Add to notes.7z".to_string(),
            }
        );
        assert_eq!(model.add_zip_path, PathBuf::from(r"D:\docs\notes.zip"));
    }

    #[test]
    fn single_archive_shows_add_and_extract() {
        let model = menu_for_selection(&[PathBuf::from("report.zip")]);
        assert_eq!(model.archives.len(), 1);
        assert!(model.rows.iter().any(|row| matches!(
            row,
            MenuRow::Command {
                action: MenuAction::ExtractHereNamed,
                ..
            }
        )));
        assert_eq!(model.extract_here_named_label, "Extract to \"report\\\"");
        assert!(!model.separate_folders);
        assert!(
            model
                .rows
                .iter()
                .any(|row| matches!(row, MenuRow::Separator))
        );
    }

    #[test]
    fn multi_archive_uses_separate_folders_label() {
        let model = menu_for_selection(&[PathBuf::from("a.zip"), PathBuf::from("b.tar.gz")]);
        assert_eq!(
            model.extract_here_named_label,
            "Extract each archive to separate folder"
        );
        assert!(model.separate_folders);
        assert_eq!(
            model.add_zip_path.file_name().and_then(|n| n.to_str()),
            Some("Archive.zip")
        );
    }

    #[test]
    fn multi_file_uses_parent_folder_name() {
        let model = menu_for_selection(&[
            PathBuf::from(r"D:\pack\one.txt"),
            PathBuf::from(r"D:\pack\two.txt"),
        ]);
        assert_eq!(model.add_zip_path, PathBuf::from(r"D:\pack\pack.zip"));
        assert_eq!(model.add_7z_path, PathBuf::from(r"D:\pack\pack.7z"));
    }

    #[test]
    fn mixed_selection_extracts_only_archives() {
        let model = menu_for_selection(&[
            PathBuf::from(r"D:\mix\photo.jpg"),
            PathBuf::from(r"D:\mix\backup.tar.gz"),
            PathBuf::from(r"D:\mix\notes.txt"),
        ]);
        assert_eq!(model.selection.len(), 3);
        assert_eq!(model.archives, vec![PathBuf::from(r"D:\mix\backup.tar.gz")]);
        assert_eq!(paths_for_action(&model, MenuAction::AddZip).len(), 3);
        assert_eq!(
            paths_for_action(&model, MenuAction::ExtractHere),
            &[PathBuf::from(r"D:\mix\backup.tar.gz")]
        );
        assert_eq!(model.extract_here_named_label, "Extract to \"backup\\\"");
    }

    #[test]
    fn flags_and_verbs() {
        assert_eq!(action_flag(MenuAction::Add), "--add");
        assert_eq!(action_flag(MenuAction::AddZip), "--add-zip");
        assert_eq!(action_flag(MenuAction::Add7z), "--add-7z");
        assert_eq!(action_verb(MenuAction::ExtractTo), "shellzip.extract");
        assert_eq!(
            action_verb(MenuAction::ExtractHereNamed),
            "shellzip.extractto"
        );
    }
}
