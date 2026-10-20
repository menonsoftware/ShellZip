//! Light file-manager palette. The list is white; chrome is a flat gray.

use egui::{Color32, Stroke, Vec2};

/// Page and file-list background.
pub const CANVAS: Color32 = Color32::from_rgb(255, 255, 255);
/// Menu, toolbar, address, and status background.
pub const PANEL: Color32 = Color32::from_rgb(240, 240, 240);
/// Hairline between chrome and the list.
pub const LINE: Color32 = Color32::from_rgb(210, 210, 210);
/// Secondary text.
pub const MUTED: Color32 = Color32::from_rgb(90, 90, 90);
/// Primary text.
pub const INK: Color32 = Color32::from_rgb(28, 28, 28);
/// Errors.
pub const DANGER: Color32 = Color32::from_rgb(185, 28, 28);
/// A passing test.
pub const OK: Color32 = Color32::from_rgb(22, 128, 61);
/// Selected row, the same blue as the file pane in the reference layout.
pub const SELECT: Color32 = Color32::from_rgb(0, 120, 215);
/// Text and icons on a selected row.
pub const SELECT_TEXT: Color32 = Color32::WHITE;
/// Toolbar and row hover.
pub const HOVER: Color32 = Color32::from_rgb(229, 241, 251);
/// Folder glyph.
pub const FOLDER: Color32 = Color32::from_rgb(214, 158, 36);
/// Add toolbar glyph.
pub const ADD: Color32 = Color32::from_rgb(0, 112, 204);
/// Extract toolbar glyph.
pub const EXTRACT: Color32 = Color32::from_rgb(194, 105, 10);
/// Test toolbar glyph.
pub const TEST: Color32 = Color32::from_rgb(22, 128, 61);
/// Open toolbar glyph.
pub const OPEN: Color32 = Color32::from_rgb(37, 99, 235);

/// Flat light visuals. Selection is a solid blue with white text.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::light();
    visuals.window_fill = CANVAS;
    visuals.panel_fill = PANEL;
    visuals.faint_bg_color = PANEL;
    visuals.extreme_bg_color = CANVAS;
    visuals.code_bg_color = CANVAS;
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, INK);
    visuals.widgets.inactive.bg_fill = CANVAS;
    visuals.widgets.inactive.weak_bg_fill = CANVAS;
    visuals.widgets.inactive.fg_stroke.color = INK;
    visuals.widgets.hovered.bg_fill = HOVER;
    visuals.widgets.hovered.weak_bg_fill = HOVER;
    visuals.widgets.hovered.fg_stroke.color = INK;
    visuals.widgets.active.bg_fill = Color32::from_rgb(204, 228, 247);
    visuals.widgets.active.weak_bg_fill = HOVER;
    visuals.widgets.active.fg_stroke.color = INK;
    visuals.widgets.open.bg_fill = CANVAS;
    visuals.widgets.open.fg_stroke.color = INK;
    visuals.selection.bg_fill = SELECT;
    visuals.selection.stroke = Stroke::new(1.0_f32, SELECT_TEXT);
    visuals.hyperlink_color = OPEN;
    visuals.striped = false;
    visuals.image_loading_spinners = false;
    visuals.window_stroke = Stroke::new(1.0_f32, LINE);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(8.0, 3.0);
    style.spacing.button_padding = Vec2::new(8.0, 3.0);
    ctx.set_style(style);
}
