//! Zinc palette shared by the panels.

use egui::Color32;

/// zinc-950, used for the deepest background.
pub const CANVAS: Color32 = Color32::from_rgb(9, 9, 11);
/// zinc-900.
pub const PANEL: Color32 = Color32::from_rgb(24, 24, 27);
/// zinc-800.
pub const RAISED: Color32 = Color32::from_rgb(39, 39, 42);
/// zinc-400, secondary text and icons.
pub const MUTED: Color32 = Color32::from_rgb(161, 161, 170);
/// zinc-100, primary text and icons.
pub const INK: Color32 = Color32::from_rgb(244, 244, 245);
/// red-400, errors.
pub const DANGER: Color32 = Color32::from_rgb(248, 113, 113);
/// green-400, a passing test.
pub const OK: Color32 = Color32::from_rgb(74, 222, 128);

/// Dark zinc visuals. Widgets stay flat; there is no accent gradient.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = PANEL;
    visuals.panel_fill = PANEL;
    visuals.faint_bg_color = RAISED;
    visuals.extreme_bg_color = CANVAS;
    visuals.code_bg_color = CANVAS;
    visuals.widgets.noninteractive.bg_fill = PANEL;
    visuals.widgets.noninteractive.fg_stroke.color = INK;
    visuals.widgets.inactive.bg_fill = RAISED;
    visuals.widgets.inactive.fg_stroke.color = INK;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(63, 63, 70);
    visuals.widgets.hovered.fg_stroke.color = INK;
    visuals.widgets.active.bg_fill = Color32::from_rgb(82, 82, 91);
    visuals.widgets.active.fg_stroke.color = INK;
    visuals.selection.bg_fill = Color32::from_rgb(63, 63, 70);
    visuals.selection.stroke.color = INK;
    ctx.set_visuals(visuals);
}
