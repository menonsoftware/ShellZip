//! Folder pane for disk directories and open archives.
//!
//! [`egui_extras::TableBuilder`] paints the list: rows are virtualized, columns
//! resize, and only the visible range is built. Archive members are indexed
//! once in [`model::ArchiveIndex`], so changing folders does not rescan the
//! listing.

mod listing;
mod model;
mod view;

pub use listing::read_directory;
pub use model::{
    ArchiveIndex, Jump, JumpTarget, Row, RowKind, Target, archive_prefix, folder_parent,
    is_supported_archive, jumps, location_text, path_under, sort_rows, totals, totals_label,
};
pub use view::{Explorer, show};
