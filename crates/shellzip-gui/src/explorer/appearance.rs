//! Icon and tint for one explorer row.

use super::model::RowKind;
use crate::icons::Icon;
use crate::theme::{FOLDER, MUTED, OPEN, SELECT_TEXT};

/// Folder rows use the filled yellow directory glyph. Selection inverts to white.
pub fn row_glyph(kind: RowKind, selected: bool) -> (Icon, egui::Color32) {
    let (icon, tint) = match kind {
        RowKind::Parent | RowKind::Directory => (Icon::Folder, FOLDER),
        RowKind::Archive => (Icon::FileArchive, OPEN),
        RowKind::Symlink => (Icon::Link, MUTED),
        RowKind::File => (Icon::File, MUTED),
    };
    if selected {
        (icon, SELECT_TEXT)
    } else {
        (icon, tint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_use_the_yellow_folder() {
        let (icon, tint) = row_glyph(RowKind::Directory, false);
        assert!(matches!(icon, Icon::Folder));
        assert_eq!(tint, FOLDER);
        let (parent, _) = row_glyph(RowKind::Parent, false);
        assert!(matches!(parent, Icon::Folder));
    }

    #[test]
    fn selected_glyphs_use_the_selection_ink() {
        let (_, tint) = row_glyph(RowKind::Directory, true);
        assert_eq!(tint, SELECT_TEXT);
    }
}
