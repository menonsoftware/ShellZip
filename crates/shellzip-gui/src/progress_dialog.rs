//! Progress window for extract and add.
//!
//! The file list stays locked while this window is open. Background hides it
//! and leaves the shell usable. Closing the window does the same as Background.

use std::time::Duration;

use crate::chrome;
use crate::explorer::format_size;
use crate::icons::Icon;
use crate::theme;
use crate::worker::ProgressSnapshot;

/// What the progress window wants the shell to do.
pub enum ProgressCommand {
    /// Park the worker.
    Pause,
    /// Release the worker.
    Resume,
    /// Hide this window and unlock the file list.
    Background,
    /// Stop the job.
    Cancel,
}

/// Paint the centered progress window. `None` means the user has not chosen yet.
pub fn show(
    ctx: &egui::Context,
    title: &str,
    path: &str,
    snapshot: &ProgressSnapshot,
) -> Option<ProgressCommand> {
    let mut command = None;
    let result = chrome::modal(ctx, "shellzip-progress", title, |ui| {
        ui.set_min_width(440.0);
        ui.label(path);
        ui.add_space(8.0);
        ui.label("Current file");
        let entry = if snapshot.entry.is_empty() {
            "Working..."
        } else {
            snapshot.entry.as_str()
        };
        ui.label(entry);
        bar(ui, snapshot.entry_done, snapshot.entry_total);
        ui.add_space(8.0);
        ui.label("Total");
        bar(ui, snapshot.done, snapshot.total);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let elapsed = if snapshot.paused {
                format!("Elapsed {} (paused)", format_clock(snapshot.elapsed))
            } else {
                format!("Elapsed {}", format_clock(snapshot.elapsed))
            };
            ui.label(elapsed);
            if let Some(left) = remaining(snapshot.done, snapshot.total, snapshot.elapsed) {
                ui.label(format!("Remaining {}", format_clock(left)));
            }
        });
        ui.label(bytes_line(snapshot));
        ui.add_space(8.0);
        ui.separator();
        ui.horizontal(|ui| {
            let pause = if snapshot.paused { "Continue" } else { "Pause" };
            if ui.button(pause).clicked() {
                command = Some(if snapshot.paused {
                    ProgressCommand::Resume
                } else {
                    ProgressCommand::Pause
                });
            }
            if ui.button("Background").clicked() {
                command = Some(ProgressCommand::Background);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if chrome::labeled_button(ui, Icon::X, theme::palette(ui.ctx()).danger, "Cancel")
                    .clicked()
                {
                    command = Some(ProgressCommand::Cancel);
                }
            });
        });
    });
    if command.is_none() && result.escape {
        return Some(ProgressCommand::Background);
    }
    command
}

/// Current or total progress. A zero total animates.
pub(crate) fn bar(ui: &mut egui::Ui, done: u64, total: u64) {
    let (fraction, unknown) = fraction(done, total);
    ui.add(
        egui::ProgressBar::new(fraction)
            .fill(theme::RUST)
            .animate(unknown)
            .desired_width(ui.available_width()),
    );
}

fn fraction(done: u64, total: u64) -> (f32, bool) {
    if total == 0 {
        (0.0, true)
    } else {
        ((done as f32 / total as f32).clamp(0.0, 1.0), false)
    }
}

fn bytes_line(snapshot: &ProgressSnapshot) -> String {
    let amount = if snapshot.total == 0 {
        format!("{} processed", format_size(snapshot.done))
    } else {
        format!(
            "{} of {}",
            format_size(snapshot.done),
            format_size(snapshot.total)
        )
    };
    match format_speed(snapshot.done, snapshot.elapsed) {
        Some(speed) => format!("{amount}  {speed}"),
        None => amount,
    }
}

fn remaining(done: u64, total: u64, elapsed: Duration) -> Option<Duration> {
    if total == 0 || done == 0 || done >= total {
        return None;
    }
    let left = elapsed.as_millis().saturating_mul(u128::from(total - done)) / u128::from(done);
    u64::try_from(left).ok().map(Duration::from_millis)
}

fn format_speed(done: u64, elapsed: Duration) -> Option<String> {
    let millis = elapsed.as_millis();
    if millis < 200 || done == 0 {
        return None;
    }
    let per_sec = u128::from(done).saturating_mul(1000) / millis;
    let per_sec = u64::try_from(per_sec).unwrap_or(u64::MAX);
    Some(format!("{}/s", format_size(per_sec)))
}

/// `HH:MM:SS` elapsed clock.
pub(crate) fn format_clock(elapsed: Duration) -> String {
    let total = elapsed.as_secs();
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

/// Fraction and whether the bar should animate because the total is unknown.
pub fn job_fraction(done: u64, total: u64) -> (f32, bool) {
    fraction(done, total)
}

/// One line for the status bar: the current file, or the operation label.
pub fn status_text(snapshot: &ProgressSnapshot) -> String {
    if snapshot.paused {
        let name = if snapshot.entry.is_empty() {
            snapshot.label.as_str()
        } else {
            snapshot.entry.as_str()
        };
        return format!("Paused {name}");
    }
    if snapshot.entry.is_empty() {
        snapshot.label.clone()
    } else {
        snapshot.entry.clone()
    }
}
