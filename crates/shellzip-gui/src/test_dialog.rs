//! Test progress and the results window that stays open until Close.

use shellzip_core::TestReport;

use crate::chrome;
use crate::icons::Icon;
use crate::progress_dialog;
use crate::theme;
use crate::worker::ProgressSnapshot;

/// What the test window wants the shell to do.
pub(crate) enum TestCommand {
    /// Stop the running test and close this window.
    Cancel,
    /// Dismiss the results.
    Close,
}

struct Results {
    summary: String,
    bad: Vec<String>,
}

/// Running test, then the report.
pub(crate) struct TestDialog {
    path: String,
    results: Option<Results>,
}

impl TestDialog {
    /// Progress page for `path`.
    pub(crate) fn running(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            results: None,
        }
    }

    /// Switch to the results page. The window stays up.
    pub(crate) fn finish(&mut self, report: &TestReport) {
        self.results = Some(Results {
            summary: summary_line(report),
            bad: bad_names(report),
        });
    }

    /// Paint progress or results. `snapshot` is the live worker state.
    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        snapshot: Option<&ProgressSnapshot>,
    ) -> Option<TestCommand> {
        let mut command = None;
        let finished = self.results.is_some();
        let title = if finished { "Test results" } else { "Testing" };
        let result = chrome::modal(ctx, "shellzip-test", title, |ui| {
            ui.set_min_width(440.0);
            ui.label(&self.path);
            ui.add_space(8.0);
            if let Some(results) = &self.results {
                ui.label(egui::RichText::new(&results.summary).strong());
                if !results.bad.is_empty() {
                    ui.add_space(6.0);
                    // Fixed list. Filling leftover height and then painting Close
                    // makes this window taller on every frame when the list is long.
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), 220.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("shellzip-test-names")
                                .auto_shrink([false, false])
                                .max_height(220.0)
                                .show(ui, |ui| {
                                    for name in &results.bad {
                                        ui.label(name);
                                    }
                                });
                        },
                    );
                }
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if chrome::labeled_button(
                            ui,
                            Icon::X,
                            theme::palette(ui.ctx()).ink,
                            "Close",
                        )
                        .clicked()
                        {
                            command = Some(TestCommand::Close);
                        }
                    });
                });
            } else {
                ui.label("Current file");
                let entry = snapshot
                    .map(|snapshot| snapshot.entry.as_str())
                    .filter(|entry| !entry.is_empty())
                    .unwrap_or(if snapshot.is_some() {
                        "Working..."
                    } else {
                        "Waiting for a password."
                    });
                ui.label(entry);
                let (entry_done, entry_total, done, total, elapsed) = snapshot
                    .map(|snapshot| {
                        (
                            snapshot.entry_done,
                            snapshot.entry_total,
                            snapshot.done,
                            snapshot.total,
                            snapshot.elapsed,
                        )
                    })
                    .unwrap_or((0, 0, 0, 0, std::time::Duration::ZERO));
                progress_dialog::bar(ui, entry_done, entry_total);
                ui.add_space(8.0);
                ui.label("Total");
                progress_dialog::bar(ui, done, total);
                ui.add_space(8.0);
                ui.label(format!(
                    "Elapsed {}",
                    progress_dialog::format_clock(elapsed)
                ));
                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if chrome::labeled_button(
                            ui,
                            Icon::X,
                            theme::palette(ui.ctx()).danger,
                            "Cancel",
                        )
                        .clicked()
                        {
                            command = Some(TestCommand::Cancel);
                        }
                    });
                });
            }
        });
        if command.is_none() && result.escape {
            return Some(if finished {
                TestCommand::Close
            } else {
                TestCommand::Cancel
            });
        }
        command
    }
}

/// One line for the results title and the status bar.
pub(crate) fn summary_line(report: &TestReport) -> String {
    let checksums = report.checksums.len();
    let encrypted = report.encrypted.len();
    if checksums > 0 && encrypted > 0 {
        format!(
            "Checksum failed for {} {}; encrypted, {} {}",
            checksums,
            noun(checksums),
            encrypted,
            noun(encrypted),
        )
    } else if checksums > 0 {
        format!("Checksum failed for {checksums} {}", noun(checksums))
    } else if encrypted > 0 {
        format!("Encrypted, {encrypted} {}", noun(encrypted))
    } else {
        format!("OK, {} entries", report.entries)
    }
}

/// Names to scroll. Checksum failures come first.
pub(crate) fn bad_names(report: &TestReport) -> Vec<String> {
    let mut names = Vec::new();
    for name in &report.checksums {
        names.push(format!("checksum: {name}"));
    }
    for name in &report.encrypted {
        names.push(format!("encrypted: {name}"));
    }
    names
}

fn noun(count: usize) -> &'static str {
    if count == 1 { "entry" } else { "entries" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_line_keeps_checksum_and_encrypted_names() {
        let ok = TestReport {
            entries: 4,
            ..TestReport::default()
        };
        assert_eq!(summary_line(&ok), "OK, 4 entries");
        assert!(bad_names(&ok).is_empty());

        let bad = TestReport {
            entries: 3,
            checksums: vec!["a.txt".to_string()],
            encrypted: vec!["b.txt".to_string()],
            ..TestReport::default()
        };
        assert_eq!(
            summary_line(&bad),
            "Checksum failed for 1 entry; encrypted, 1 entry"
        );
        assert_eq!(
            bad_names(&bad),
            vec![
                "checksum: a.txt".to_string(),
                "encrypted: b.txt".to_string()
            ]
        );
    }
}
