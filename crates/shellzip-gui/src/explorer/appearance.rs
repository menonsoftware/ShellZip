//! Icon and tint for one explorer row.

use super::model::RowKind;
use crate::icons::Icon;
use crate::theme::Palette;

/// Folder rows use the filled yellow directory glyph. Selection inverts to white.
pub fn row_glyph(kind: RowKind, selected: bool, colors: Palette) -> (Icon, egui::Color32) {
    let (icon, tint) = match kind {
        RowKind::Parent | RowKind::Directory => (Icon::Folder, colors.folder),
        RowKind::Archive => (Icon::FileArchive, colors.open),
        RowKind::Symlink => (Icon::Link, colors.muted),
        RowKind::File => (Icon::File, colors.muted),
    };
    if selected {
        (icon, colors.select_text)
    } else {
        (icon, tint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directories_use_the_yellow_folder() {
        let (icon, tint) = row_glyph(RowKind::Directory, false, crate::theme::LIGHT);
        assert!(matches!(icon, Icon::Folder));
        assert_eq!(tint, crate::theme::LIGHT.folder);
        let (parent, _) = row_glyph(RowKind::Parent, false, crate::theme::LIGHT);
        assert!(matches!(parent, Icon::Folder));
    }

    #[test]
    fn selected_glyphs_use_the_selection_ink() {
        let (_, tint) = row_glyph(RowKind::Directory, true, crate::theme::LIGHT);
        assert_eq!(tint, crate::theme::LIGHT.select_text);
    }
}
