//! Light and dark file-manager palettes. The list is flat; chrome is one step off it.
//!
//! Both palettes are installed at startup. [`Preference`] chooses which one is
//! active, and [`palette`] is what custom-painted chrome reads each frame.

use egui::{Color32, Stroke, Vec2};

/// Which palette the window uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preference {
    /// Follow the operating system. Unknown OS themes fall back to light.
    #[default]
    System,
    /// White list, gray chrome.
    Light,
    /// Dark list and chrome.
    Dark,
}

impl Preference {
    /// Token stored in `settings.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    /// Parse a settings token. Anything unrecognized is [`Self::System`].
    pub fn from_token(token: &str) -> Self {
        match token {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }

    /// Switch the active palette. Both visuals are already registered by [`apply`].
    pub fn install(self, ctx: &egui::Context) {
        ctx.set_theme(match self {
            Self::System => egui::ThemePreference::System,
            Self::Light => egui::ThemePreference::Light,
            Self::Dark => egui::ThemePreference::Dark,
        });
    }
}

/// Colors for one mode. Custom paint reads this; egui widgets read [`apply`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Page and file-list background.
    pub canvas: Color32,
    /// Menu, toolbar, address, and status background.
    pub panel: Color32,
    /// Hairline between chrome and the list.
    pub line: Color32,
    /// Secondary text.
    pub muted: Color32,
    /// Primary text.
    pub ink: Color32,
    /// Errors and the Delete glyph.
    pub danger: Color32,
    /// A passing test, and the Test glyph.
    pub ok: Color32,
    /// Selected row, the same blue as the file pane in the reference layout.
    pub select: Color32,
    /// Text and icons on a selected row.
    pub select_text: Color32,
    /// Toolbar and row hover.
    pub hover: Color32,
    /// Pressed widget fill.
    pub active: Color32,
    /// Filled directory icon. A gold outline reads as brown at 16 px.
    pub folder: Color32,
    /// Add toolbar glyph.
    pub add: Color32,
    /// Extract toolbar glyph.
    pub extract: Color32,
    /// Test toolbar glyph.
    pub test: Color32,
    /// Info toolbar glyph and links.
    pub open: Color32,
}

/// Light file manager. Selection is the Windows blue with white text.
pub const LIGHT: Palette = Palette {
    canvas: Color32::from_rgb(255, 255, 255),
    panel: Color32::from_rgb(240, 240, 240),
    line: Color32::from_rgb(210, 210, 210),
    muted: Color32::from_rgb(90, 90, 90),
    ink: Color32::from_rgb(28, 28, 28),
    danger: Color32::from_rgb(185, 28, 28),
    ok: Color32::from_rgb(22, 128, 61),
    select: Color32::from_rgb(0, 120, 215),
    select_text: Color32::WHITE,
    hover: Color32::from_rgb(229, 241, 251),
    active: Color32::from_rgb(204, 228, 247),
    folder: Color32::from_rgb(255, 196, 24),
    add: Color32::from_rgb(0, 112, 204),
    extract: Color32::from_rgb(194, 105, 10),
    test: Color32::from_rgb(22, 128, 61),
    open: Color32::from_rgb(37, 99, 235),
};

/// Dark file manager. Selection blue and folder gold stay; glyphs are brighter.
pub const DARK: Palette = Palette {
    canvas: Color32::from_rgb(0x1F, 0x1F, 0x1F),
    panel: Color32::from_rgb(0x2B, 0x2B, 0x2B),
    line: Color32::from_rgb(0x3F, 0x3F, 0x3F),
    muted: Color32::from_rgb(0xB4, 0xB4, 0xB4),
    ink: Color32::from_rgb(0xF3, 0xF3, 0xF3),
    danger: Color32::from_rgb(0xF2, 0x5C, 0x5C),
    ok: Color32::from_rgb(0x4A, 0xDE, 0x80),
    select: Color32::from_rgb(0, 120, 215),
    select_text: Color32::WHITE,
    hover: Color32::from_rgb(0x33, 0x33, 0x33),
    active: Color32::from_rgb(0x3A, 0x3A, 0x3A),
    folder: Color32::from_rgb(0xFF, 0xC4, 0x18),
    add: Color32::from_rgb(0x4D, 0xA3, 0xFF),
    extract: Color32::from_rgb(0xF0, 0xA0, 0x4B),
    test: Color32::from_rgb(0x4A, 0xDE, 0x80),
    open: Color32::from_rgb(0x7A, 0xA2, 0xFF),
};

/// The palette for the theme that is active this frame.
pub fn palette(ctx: &egui::Context) -> &'static Palette {
    match ctx.theme() {
        egui::Theme::Dark => &DARK,
        egui::Theme::Light => &LIGHT,
    }
}

/// Install both palettes and select `preference`.
pub fn apply(ctx: &egui::Context, preference: Preference) {
    ctx.options_mut(|opt| opt.fallback_theme = egui::Theme::Light);
    ctx.set_visuals_of(egui::Theme::Light, visuals(&LIGHT, egui::Visuals::light()));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(&DARK, egui::Visuals::dark()));
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 3.0);
        style.spacing.button_padding = Vec2::new(8.0, 3.0);
    });
    preference.install(ctx);
}

fn visuals(palette: &Palette, mut visuals: egui::Visuals) -> egui::Visuals {
    visuals.window_fill = palette.canvas;
    visuals.panel_fill = palette.panel;
    visuals.faint_bg_color = palette.panel;
    visuals.extreme_bg_color = palette.canvas;
    visuals.code_bg_color = palette.canvas;
    visuals.widgets.noninteractive.bg_fill = palette.panel;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, palette.ink);
    visuals.widgets.inactive.bg_fill = palette.canvas;
    visuals.widgets.inactive.weak_bg_fill = palette.canvas;
    visuals.widgets.inactive.fg_stroke.color = palette.ink;
    visuals.widgets.hovered.bg_fill = palette.hover;
    visuals.widgets.hovered.weak_bg_fill = palette.hover;
    visuals.widgets.hovered.fg_stroke.color = palette.ink;
    visuals.widgets.active.bg_fill = palette.active;
    visuals.widgets.active.weak_bg_fill = palette.hover;
    visuals.widgets.active.fg_stroke.color = palette.ink;
    visuals.widgets.open.bg_fill = palette.canvas;
    visuals.widgets.open.fg_stroke.color = palette.ink;
    visuals.selection.bg_fill = palette.select;
    visuals.selection.stroke = Stroke::new(1.0_f32, palette.select_text);
    visuals.hyperlink_color = palette.open;
    visuals.striped = false;
    visuals.image_loading_spinners = false;
    visuals.window_stroke = Stroke::new(1.0_f32, palette.line);
    visuals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_installs_both_themes() {
        let ctx = egui::Context::default();
        apply(&ctx, Preference::System);
        ctx.options(|opt| assert_eq!(opt.fallback_theme, egui::Theme::Light));

        ctx.set_theme(egui::Theme::Light);
        assert_eq!(ctx.style().visuals.panel_fill, LIGHT.panel);
        assert_eq!(ctx.style().visuals.selection.bg_fill, LIGHT.select);
        assert_eq!(palette(&ctx).ink, LIGHT.ink);

        ctx.set_theme(egui::Theme::Dark);
        assert_eq!(ctx.style().visuals.panel_fill, DARK.panel);
        assert_eq!(ctx.style().visuals.selection.bg_fill, DARK.select);
        assert_eq!(palette(&ctx).add, DARK.add);
    }

    #[test]
    fn theme_tokens_round_trip() {
        assert_eq!(Preference::from_token("light"), Preference::Light);
        assert_eq!(Preference::from_token("dark"), Preference::Dark);
        assert_eq!(Preference::from_token("system"), Preference::System);
        assert_eq!(Preference::from_token("nope"), Preference::System);
        assert_eq!(Preference::Dark.as_str(), "dark");
    }
}
