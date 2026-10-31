//! Ask for a new file or folder name in the same directory.

use std::path::PathBuf;

use crate::chrome;
use crate::icons::Icon;
use crate::theme;

/// What the new leaf name replaces.
#[derive(Clone)]
pub(crate) enum RenameTarget {
    /// A filesystem path. The parent directory stays the same.
    Disk(PathBuf),
    /// Archive member path, or a folder prefix, without a trailing slash.
    Member(String),
}

/// New-name prompt. [`RenameDialog::show`] returns an action when the user finishes.
pub(crate) struct RenameDialog {
    current: String,
    next: String,
    target: RenameTarget,
    notice: Option<String>,
}

/// The user confirmed a leaf name, or dismissed the prompt.
pub(crate) enum RenameAction {
    /// `leaf` passed [`reject_new_name`].
    Submit { target: RenameTarget, leaf: String },
    /// Cancel or the window close button.
    Cancel,
}

impl RenameDialog {
    /// Prefill the field with the name shown in the file pane.
    pub(crate) fn new(current: impl Into<String>, target: RenameTarget) -> Self {
        let current = current.into();
        Self {
            next: current.clone(),
            current,
            target,
            notice: None,
        }
    }

    /// Paint the prompt.
    pub(crate) fn show(&mut self, ctx: &egui::Context) -> Option<RenameAction> {
        let mut open = true;
        let mut action = None;
        egui::Window::new("Rename")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(380.0);
                ui.label(format!("Rename {}", self.current));
                ui.add_space(6.0);
                ui.label("New name");
                let field = ui.add(
                    egui::TextEdit::singleline(&mut self.next)
                        .desired_width(f32::INFINITY)
                        .id_salt("shellzip-rename"),
                );
                field.request_focus();
                if let Some(notice) = &self.notice {
                    ui.add_space(4.0);
                    ui.colored_label(theme::palette(ui.ctx()).danger, notice);
                }
                ui.add_space(8.0);
                ui.separator();
                let submit = ui.input(|input| input.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let colors = theme::palette(ui.ctx());
                        if chrome::labeled_button(ui, Icon::X, colors.ink, "Cancel").clicked() {
                            action = Some(RenameAction::Cancel);
                        }
                        if chrome::labeled_button(ui, Icon::Check, colors.ok, "OK").clicked()
                            || submit
                        {
                            match reject_new_name(self.next.trim()) {
                                Some(message) => self.notice = Some(message.to_string()),
                                None => {
                                    action = Some(RenameAction::Submit {
                                        target: self.target.clone(),
                                        leaf: self.next.trim().to_string(),
                                    });
                                }
                            }
                        }
                    });
                });
            });
        if action.is_none() && !open {
            return Some(RenameAction::Cancel);
        }
        action
    }
}

/// Reject an empty leaf, a slash, a backslash, or a name containing `..`.
pub(crate) fn reject_new_name(name: &str) -> Option<&'static str> {
    let name = name.trim();
    if name.is_empty() {
        return Some("Enter a new name.");
    }
    if name.contains('/') || name.contains('\\') {
        return Some("The name cannot contain slashes or backslashes.");
    }
    if name.contains("..") {
        return Some("The name cannot contain '..'.");
    }
    None
}

/// Member path after replacing the last component with `leaf`.
pub(crate) fn renamed_member(from: &str, leaf: &str) -> String {
    let from = from.trim_end_matches(['/', '\\']);
    match from.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => format!("{parent}/{leaf}"),
        _ => leaf.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_slashes_and_dotdot() {
        assert!(reject_new_name("").is_some());
        assert!(reject_new_name("   ").is_some());
        assert!(reject_new_name("a/b").is_some());
        assert!(reject_new_name(r"a\b").is_some());
        assert!(reject_new_name("..").is_some());
        assert!(reject_new_name("foo..bar").is_some());
        assert!(reject_new_name("notes.txt").is_none());
    }

    #[test]
    fn renamed_member_keeps_the_parent_prefix() {
        assert_eq!(renamed_member("docs/images", "photos"), "docs/photos");
        assert_eq!(renamed_member("notes.txt", "read.txt"), "read.txt");
        assert_eq!(renamed_member("docs/images/", "photos"), "docs/photos");
    }
}
