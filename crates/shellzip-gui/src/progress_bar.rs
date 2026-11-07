//! Segmented progress bars in the ShellZip copper-rust palette.
//!
//! Layout follows classic Windows XP / Vista chunk bars: discrete blocks with
//! a thin gap, soft vertical shading, and a quiet chrome track. Colors come
//! from [`theme::RUST`], not the stock egui fill.

use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::{self, RUST};

/// Lighter face on the top of each chunk.
const CHUNK_HIGHLIGHT: Color32 = Color32::from_rgb(0xE8, 0x8A, 0x55);
/// Deeper face on the bottom of each chunk.
const CHUNK_SHADE: Color32 = Color32::from_rgb(0x8F, 0x42, 0x1F);

const CHUNK_WIDTH: f32 = 7.0;
const CHUNK_GAP: f32 = 1.5;
const BAR_HEIGHT: f32 = 16.0;
const TRACK_INSET: f32 = 2.0;

/// Paint a full-width bar for `done` of `total`. A zero total marquee-animates.
pub fn bytes(ui: &mut Ui, done: u64, total: u64) {
    let (fraction, indeterminate) = fraction(done, total);
    show(ui, fraction, indeterminate, ui.available_width());
}

/// Paint a bar of `desired_width` (status chrome uses a fixed width).
pub fn show(ui: &mut Ui, fraction: f32, indeterminate: bool, desired_width: f32) {
    let height = BAR_HEIGHT.min(ui.spacing().interact_size.y.max(12.0));
    let (rect, _response) =
        ui.allocate_exact_size(Vec2::new(desired_width, height), Sense::hover());
    paint(ui, rect, fraction.clamp(0.0, 1.0), indeterminate);
    if indeterminate {
        ui.ctx().request_repaint();
    }
}

/// Vertical chunk meter used by the archive Info ratio column.
pub fn vertical(ui: &mut Ui, fraction: f32, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    paint_vertical(ui, rect, fraction.clamp(0.0, 1.0));
}

fn fraction(done: u64, total: u64) -> (f32, bool) {
    if total == 0 {
        (0.0, true)
    } else {
        ((done as f32 / total as f32).clamp(0.0, 1.0), false)
    }
}

fn paint(ui: &Ui, rect: Rect, fraction: f32, indeterminate: bool) {
    let colors = theme::palette(ui.ctx());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, colors.field);
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, colors.line),
        StrokeKind::Inside,
    );

    let inner = rect.shrink2(Vec2::splat(TRACK_INSET));
    if inner.width() < CHUNK_WIDTH || inner.height() <= 0.0 {
        return;
    }

    let pitch = CHUNK_WIDTH + CHUNK_GAP;
    let capacity = ((inner.width() + CHUNK_GAP) / pitch).floor().max(1.0) as i32;
    let time = ui.input(|input| input.time);

    if indeterminate {
        let window = (capacity / 5).clamp(3, 8);
        let cycle = capacity + window;
        let head = ((time * 10.0).rem_euclid(f64::from(cycle))) as i32;
        for index in 0..window {
            let slot = head - index;
            if (0..capacity).contains(&slot) {
                paint_chunk(&painter, chunk_rect(inner, slot));
            }
        }
        return;
    }

    let filled = ((fraction * capacity as f32).round() as i32).clamp(0, capacity);
    for slot in 0..filled {
        paint_chunk(&painter, chunk_rect(inner, slot));
    }
}

fn chunk_rect(inner: Rect, slot: i32) -> Rect {
    let pitch = CHUNK_WIDTH + CHUNK_GAP;
    let left = inner.left() + slot as f32 * pitch;
    Rect::from_min_size(
        Pos2::new(left, inner.top()),
        Vec2::new(CHUNK_WIDTH, inner.height()),
    )
}

fn paint_vertical(ui: &Ui, rect: Rect, fraction: f32) {
    let colors = theme::palette(ui.ctx());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, colors.field);
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, colors.line),
        StrokeKind::Inside,
    );
    let inner = rect.shrink2(Vec2::splat(TRACK_INSET));
    if inner.height() < CHUNK_WIDTH {
        return;
    }
    let pitch = CHUNK_WIDTH + CHUNK_GAP;
    let capacity = ((inner.height() + CHUNK_GAP) / pitch).floor().max(1.0) as i32;
    let filled = ((fraction * capacity as f32).round() as i32).clamp(0, capacity);
    for slot in 0..filled {
        let bottom = inner.bottom() - slot as f32 * pitch;
        let top = bottom - CHUNK_WIDTH;
        paint_chunk(
            &painter,
            Rect::from_min_max(
                Pos2::new(inner.left(), top),
                Pos2::new(inner.right(), bottom),
            ),
        );
    }
}

fn paint_chunk(painter: &egui::Painter, rect: Rect) {
    let mid_y = rect.center().y;
    let top = Rect::from_min_max(rect.left_top(), Pos2::new(rect.right(), mid_y));
    let bottom = Rect::from_min_max(Pos2::new(rect.left(), mid_y), rect.right_bottom());
    painter.rect_filled(top, 0.0, CHUNK_HIGHLIGHT);
    painter.rect_filled(bottom, 0.0, CHUNK_SHADE);
    // Thin mid band keeps the brand copper readable at small sizes.
    let band = Rect::from_center_size(rect.center(), Vec2::new(rect.width(), rect.height() * 0.35));
    painter.rect_filled(band, 0.0, RUST);
    painter.hline(
        rect.x_range(),
        rect.top() + 0.5,
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, 48)),
    );
}

#[cfg(test)]
mod tests {
    use super::{CHUNK_GAP, CHUNK_WIDTH, fraction};

    #[test]
    fn unknown_totals_are_indeterminate() {
        assert_eq!(fraction(0, 0), (0.0, true));
        assert_eq!(fraction(10, 0), (0.0, true));
        assert_eq!(fraction(5, 10), (0.5, false));
        assert_eq!(fraction(10, 10), (1.0, false));
    }

    #[test]
    fn chunk_pitch_leaves_a_visible_gap() {
        const {
            assert!(CHUNK_WIDTH > CHUNK_GAP);
            assert!(CHUNK_GAP >= 1.0);
        }
    }
}
