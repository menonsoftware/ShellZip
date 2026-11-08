//! Velopack auto-update checks against GitHub Releases.

use velopack::sources::GithubSource;
use velopack::{Error, UpdateCheck, UpdateManager};

/// Public GitHub repository that hosts Velopack release assets.
pub const GITHUB_REPO: &str = "https://github.com/menonsoftware/ShellZip";

/// Result of a background update check, shown in the status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// No newer release on the channel.
    UpToDate,
    /// Remote feed had no packages.
    EmptyFeed,
    /// An update was downloaded; the process should restart via Velopack.
    Applying {
        /// Semver of the package about to be applied.
        version: String,
    },
    /// Running from `cargo run`, a zip drop, or another non-Velopack layout.
    NotInstalled,
    /// Installed from the Microsoft Store; Partner Center delivers updates.
    StoreInstall,
    /// Network, feed, or apply failure.
    Failed(String),
}

/// Check GitHub Releases and download/apply when a newer package exists.
///
/// Call from a worker thread. On [`CheckResult::Applying`], Velopack restarts
/// the process and this function does not return.
pub fn check_and_apply() -> CheckResult {
    if crate::package::is_packaged() {
        return CheckResult::StoreInstall;
    }
    let source = GithubSource::new(GITHUB_REPO, None, false);
    let manager = match UpdateManager::new(source, None, None) {
        Ok(manager) => manager,
        Err(Error::NotInstalled(_)) => return CheckResult::NotInstalled,
        Err(err) => return CheckResult::Failed(err.to_string()),
    };

    match manager.check_for_updates() {
        Ok(UpdateCheck::NoUpdateAvailable) => CheckResult::UpToDate,
        Ok(UpdateCheck::RemoteIsEmpty) => CheckResult::EmptyFeed,
        Ok(UpdateCheck::UpdateAvailable(info)) => {
            let version = info.TargetFullRelease.Version.clone();
            if let Err(err) = manager.download_updates(&info, None) {
                return CheckResult::Failed(err.to_string());
            }
            match manager.apply_updates_and_restart(&*info) {
                Ok(()) => CheckResult::Applying { version },
                Err(err) => CheckResult::Failed(err.to_string()),
            }
        }
        Err(err) => CheckResult::Failed(err.to_string()),
    }
}

impl CheckResult {
    /// Status-line text for the window.
    pub fn status_line(&self) -> String {
        match self {
            Self::UpToDate => "ShellZip is up to date.".into(),
            Self::EmptyFeed => "No update packages are published yet.".into(),
            Self::Applying { version } => format!("Updating to {version}…"),
            Self::NotInstalled => {
                "Updates need a Velopack install (Setup or AppImage), not a cargo build.".into()
            }
            Self::StoreInstall => "Updates for this install come from the Microsoft Store.".into(),
            Self::Failed(message) => format!("Update check failed: {message}"),
        }
    }

    /// True when the text should be shown as an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed(_) | Self::NotInstalled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_lines_are_user_facing() {
        assert!(!CheckResult::UpToDate.status_line().is_empty());
        assert!(CheckResult::NotInstalled.is_error());
        assert!(!CheckResult::EmptyFeed.is_error());
    }
}
