//! Virtualized name, size, packed, type, modified, and CRC columns.

use std::collections::HashSet;

use egui_extras::{Column, TableBuilder};
use shellzip_core::format_crc;

use super::appearance;
use super::model::{self, Row, RowKind, SortColumn, SortOrder};
use crate::icons::{self, Caret};
use crate::shell_icons::{self, IconCache};
use crate::theme;

/// Selection and sort for the file pane.
#[derive(Debug, Default)]
pub struct Explorer {
    column: SortColumn,
    order: SortOrder,
    selected: HashSet<String>,
    anchor: Option<String>,
}

impl Explorer {
    /// Name ascending, nothing selected.
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop the selection after a navigation.
    pub fn clear_selection(&mut self) {
        self.selected.clear();
        self.anchor = None;
    }

    /// Select one row and make it the shift-click anchor.
    pub fn select_only(&mut self, key: &str) {
        self.selected.clear();
        self.selected.insert(key.to_string());
        self.anchor = Some(key.to_string());
    }

    /// Select every row except `..`.
    pub fn select_all(&mut self, rows: &[Row]) {
        self.selected.clear();
        for row in rows {
            if row.kind != RowKind::Parent {
                self.selected.insert(row.key.clone());
            }
        }
        self.anchor = rows
            .iter()
            .find(|row| row.kind != RowKind::Parent)
            .map(|row| row.key.clone());
    }

    /// Select every unselected row except `..`, and deselect the rest.
    pub fn invert_selection(&mut self, rows: &[Row]) {
        let mut next = HashSet::new();
        for row in rows {
            if row.kind != RowKind::Parent && !self.selected.contains(&row.key) {
                next.insert(row.key.clone());
            }
        }
        self.anchor = next.iter().next().cloned();
        self.selected = next;
    }

    /// Whether `key` is part of the selection.
    pub fn is_selected(&self, key: &str) -> bool {
        self.selected.contains(key)
    }

    /// Row that Enter activates.
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// Current header sort.
    pub fn ordering(&self) -> (SortColumn, SortOrder) {
        (self.column, self.order)
    }

    fn toggle(&mut self, column: SortColumn) {
        if self.column == column {
            self.order = match self.order {
                SortOrder::Asc => SortOrder::Desc,
                SortOrder::Desc => SortOrder::Asc,
            };
        } else {
            self.column = column;
            self.order = SortOrder::Asc;
        }
    }

    fn click(&mut self, index: usize, rows: &[Row], modifiers: egui::Modifiers) {
        let Some(row) = rows.get(index) else {
            return;
        };
        let key = row.key.clone();
        if modifiers.command {
            if !self.selected.insert(key.clone()) {
                self.selected.remove(&key);
            }
            self.anchor = Some(key);
        } else if modifiers.shift {
            self.select_range(index, rows, &key);
        } else {
            self.selected.clear();
            self.selected.insert(key.clone());
            self.anchor = Some(key);
        }
    }

    fn select_range(&mut self, index: usize, rows: &[Row], key: &str) {
        let Some(anchor) = &self.anchor else {
            self.selected.clear();
            self.selected.insert(key.to_string());
            self.anchor = Some(key.to_string());
            return;
        };
        let Some(start) = rows.iter().position(|row| row.key == *anchor) else {
            self.selected.clear();
            self.selected.insert(key.to_string());
            self.anchor = Some(key.to_string());
            return;
        };
        let (lo, hi) = if start < index {
            (start, index)
        } else {
            (index, start)
        };
        self.selected.clear();
        for row in &rows[lo..=hi] {
            self.selected.insert(row.key.clone());
        }
    }
}

/// Paint `rows` and return the index that was double-clicked.
///
/// A header click reorders `rows` before the body is built, so the new order
/// is visible on the same frame. `scroll_to` is consumed once. Each name cell
/// uses a shell icon from `cache`, or a Lucide glyph until that icon arrives.
pub fn show(
    ui: &mut egui::Ui,
    explorer: &mut Explorer,
    rows: &mut [Row],
    scroll_to: &mut Option<usize>,
    cache: &mut IconCache,
) -> Option<usize> {
    // Cells share edges, so the selection fill is one bar instead of a strip behind each label.
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
    ui.style_mut().interaction.selectable_labels = false;
    let colors = *theme::palette(ui.ctx());

    let mut builder = TableBuilder::new(ui)
        .id_salt("shellzip-files")
        .striped(false)
        .resizable(true)
        .sense(egui::Sense::click())
        .drag_to_scroll(false)
        .auto_shrink(false)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(28.0).resizable(false))
        .column(Column::remainder().at_least(160.0).clip(true))
        .column(Column::initial(90.0).at_least(64.0).clip(true))
        .column(Column::initial(90.0).at_least(64.0).clip(true))
        .column(Column::initial(160.0).at_least(100.0).clip(true))
        .column(Column::initial(150.0).at_least(120.0).clip(true))
        .column(Column::initial(90.0).at_least(72.0).clip(true));
    if let Some(index) = scroll_to.take() {
        builder = builder.scroll_to_row(index, Some(egui::Align::Center));
    }

    let mut resorted = false;
    let table = builder.header(24.0, |mut header| {
        header.col(paint_header_cell);
        if header_clicked(
            &mut header,
            "Name",
            explorer.column == SortColumn::Name,
            explorer.order,
        ) {
            explorer.toggle(SortColumn::Name);
            resorted = true;
        }
        if header_clicked(
            &mut header,
            "Size",
            explorer.column == SortColumn::Size,
            explorer.order,
        ) {
            explorer.toggle(SortColumn::Size);
            resorted = true;
        }
        if header_clicked(
            &mut header,
            "Packed",
            explorer.column == SortColumn::Packed,
            explorer.order,
        ) {
            explorer.toggle(SortColumn::Packed);
            resorted = true;
        }
        if header_clicked(
            &mut header,
            "Type",
            explorer.column == SortColumn::Type,
            explorer.order,
        ) {
            explorer.toggle(SortColumn::Type);
            resorted = true;
        }
        if header_clicked(
            &mut header,
            "Modified",
            explorer.column == SortColumn::Modified,
            explorer.order,
        ) {
            explorer.toggle(SortColumn::Modified);
            resorted = true;
        }
        if header_clicked(
            &mut header,
            "CRC",
            explorer.column == SortColumn::Crc,
            explorer.order,
        ) {
            explorer.toggle(SortColumn::Crc);
            resorted = true;
        }
    });
    if resorted {
        model::sort_rows(rows, explorer.column, explorer.order);
    }

    let mut activate = None;
    table.body(|body| {
        body.rows(22.0, rows.len(), |mut row| {
            let index = row.index();
            let item = &rows[index];
            let selected = explorer.selected.contains(&item.key);
            row.set_selected(selected);
            let text = if selected {
                colors.select_text
            } else {
                colors.ink
            };
            let key = shell_icons::row_icon_key(item);
            // The icon lives in its own unclipped cell. Clipping the name text
            // was also clipping the SVG down to an empty square.
            row.col(|ui| {
                ui.add_space(8.0);
                if !shell_icons::paint(ui, cache, &key, icons::SIZE_ROW) {
                    let (icon, tint) = appearance::row_glyph(item.kind, selected, colors);
                    icons::show(ui, icon, tint, icons::SIZE_ROW);
                }
            });
            row.col(|ui| {
                ui.add_space(4.0);
                cell_text(ui, &item.name, text);
            });
            row.col(|ui| {
                ui.add_space(6.0);
                size_cell(ui, item, text);
            });
            row.col(|ui| {
                ui.add_space(6.0);
                packed_cell(ui, item, text);
            });
            row.col(|ui| {
                ui.add_space(6.0);
                cell_text(ui, &item.type_label, text);
            });
            row.col(|ui| {
                ui.add_space(6.0);
                cell_text(ui, &modified_text(item), text);
            });
            row.col(|ui| {
                ui.add_space(6.0);
                cell_text(ui, &crc_text(item), text);
            });
            // `row.response` is the union of every cell, including the empty
            // padding, because the labels and icons do not take the pointer.
            let response = row.response();
            if response.double_clicked() {
                activate = Some(index);
            } else if response.clicked() {
                let modifiers = response.ctx.input(|input| input.modifiers);
                explorer.click(index, rows, modifiers);
            }
        });
    });
    activate
}

fn header_clicked(
    header: &mut egui_extras::TableRow<'_, '_>,
    label: &str,
    active: bool,
    order: SortOrder,
) -> bool {
    let (_, response) = header.col(|ui| {
        paint_header_cell(ui);
        ui.add_space(6.0);
        ui.strong(label);
        if active {
            ui.add_space(6.0);
            icons::caret(
                ui,
                match order {
                    SortOrder::Asc => Caret::Up,
                    SortOrder::Desc => Caret::Down,
                },
                theme::palette(ui.ctx()).ink,
            );
        }
    });
    response.clicked()
}

fn paint_header_cell(ui: &mut egui::Ui) {
    let colors = theme::palette(ui.ctx());
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, colors.panel);
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        egui::Stroke::new(1.0_f32, colors.line),
    );
}

fn cell_text(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.add(egui::Label::new(egui::RichText::new(text).color(color)).sense(egui::Sense::empty()));
}

fn size_cell(ui: &mut egui::Ui, row: &Row, color: egui::Color32) {
    let label = size_text(row);
    let response = ui.add(
        egui::Label::new(egui::RichText::new(&label).color(color)).sense(egui::Sense::hover()),
    );
    if !label.is_empty()
        && let Some(bytes) = row.size
    {
        response.on_hover_text(format!("{bytes} bytes"));
    }
}

fn packed_cell(ui: &mut egui::Ui, row: &Row, color: egui::Color32) {
    let label = packed_text(row);
    let response = ui.add(
        egui::Label::new(egui::RichText::new(&label).color(color)).sense(egui::Sense::hover()),
    );
    if !label.is_empty()
        && let Some(bytes) = row.packed
    {
        response.on_hover_text(format!("{bytes} bytes"));
    }
}

fn size_text(row: &Row) -> String {
    if matches!(row.kind, RowKind::Directory | RowKind::Parent) {
        String::new()
    } else {
        row.size.map(model::format_size).unwrap_or_default()
    }
}

fn modified_text(row: &Row) -> String {
    row.modified.map(model::format_modified).unwrap_or_default()
}

fn packed_text(row: &Row) -> String {
    if matches!(row.kind, RowKind::Directory | RowKind::Parent) {
        String::new()
    } else {
        row.packed.map(model::format_size).unwrap_or_default()
    }
}

fn crc_text(row: &Row) -> String {
    if matches!(row.kind, RowKind::Directory | RowKind::Parent) || row.crc32.is_none() {
        String::new()
    } else {
        format_crc(row.crc32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explorer::model::{Row, RowKind, Target};

    fn file(name: &str) -> Row {
        Row {
            name: name.to_string(),
            kind: RowKind::File,
            size: Some(1),
            packed: None,
            modified: None,
            crc32: None,
            type_label: String::new(),
            key: name.to_string(),
            target: Target::Path(name.into()),
        }
    }

    #[test]
    fn invert_selection_flips_every_row_except_parent() {
        let rows = [Row::parent(), file("a.txt"), file("b.txt"), file("c.txt")];
        let mut explorer = Explorer::new();
        explorer.select_only("b.txt");
        explorer.invert_selection(&rows);
        assert!(!explorer.is_selected("b.txt"));
        assert!(explorer.is_selected("a.txt"));
        assert!(explorer.is_selected("c.txt"));
        assert!(!explorer.is_selected(crate::explorer::model::PARENT_KEY));
    }
}
