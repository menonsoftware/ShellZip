//! Context-menu labels for one or many selected archives.

use std::path::Path;

use crate::stem::folder_stem;

/// One verb the shell extension or desktop action exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Open the archive in the ShellZip window.
    Open,
    /// Show the extract dialog.
    ExtractTo,
    /// Extract into the archive's parent directory.
    ExtractHere,
    /// Extract into a new folder named after the archive stem.
    ExtractHereNamed,
}

/// Labels and the extract-here-named caption for the current selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuModel {
    /// Caption for the named extract item.
    pub extract_here_named_label: String,
    /// When true, `--extract-here-named` creates one folder per archive.
    pub separate_folders: bool,
}

/// Build the menu model for the selected archive paths.
pub fn menu_for_selection(paths: &[impl AsRef<Path>]) -> MenuModel {
    match paths {
        [] => MenuModel {
            extract_here_named_label: "Extract Here (folder)".to_string(),
            separate_folders: false,
        },
        [one] => {
            let stem = folder_stem(one.as_ref());
            MenuModel {
                extract_here_named_label: format!("Extract Here ({stem})"),
                separate_folders: false,
            }
        }
        _ => MenuModel {
            extract_here_named_label: "Extract Here (separate folders)".to_string(),
            separate_folders: true,
        },
    }
}

/// Static caption for data-only Linux menus that cannot put the stem in the label.
pub fn linux_named_label() -> &'static str {
    "Extract Here (folder)"
}

/// Human label for a fixed menu action.
pub fn action_label(action: MenuAction) -> &'static str {
    match action {
        MenuAction::Open => "Open",
        MenuAction::ExtractTo => "Extract to…",
        MenuAction::ExtractHere => "Extract Here",
        MenuAction::ExtractHereNamed => "Extract Here (folder)",
    }
}

/// CLI flag for a menu action.
pub fn action_flag(action: MenuAction) -> &'static str {
    match action {
        MenuAction::Open => "",
        MenuAction::ExtractTo => "--extract-to",
        MenuAction::ExtractHere => "--extract-here",
        MenuAction::ExtractHereNamed => "--extract-here-named",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn single_and_multi_labels() {
        let one = menu_for_selection(&[PathBuf::from("report.zip")]);
        assert_eq!(one.extract_here_named_label, "Extract Here (report)");
        assert!(!one.separate_folders);

        let many = menu_for_selection(&[PathBuf::from("a.zip"), PathBuf::from("b.tar.gz")]);
        assert_eq!(
            many.extract_here_named_label,
            "Extract Here (separate folders)"
        );
        assert!(many.separate_folders);
    }
}
