//! Command-line verbs for the desktop window and shell menus.

use std::path::PathBuf;
use std::process::ExitCode;

use shellzip_integrate::{IntegratePaths, folder_stem, integrate, unintegrate};

/// How the process was started from the shell or a shortcut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchAction {
    /// No archive argument; open the usual starting folder.
    Browse,
    /// Open one archive in the window.
    Open(PathBuf),
    /// Show the extract dialog for these archives.
    ExtractTo(Vec<PathBuf>),
    /// Extract into each archive's parent directory.
    ExtractHere(Vec<PathBuf>),
    /// Extract into a new folder named after each archive stem.
    ExtractHereNamed(Vec<PathBuf>),
}

/// Result of parsing argv before the window opens.
#[derive(Debug)]
pub enum Startup {
    /// Run integrate / unintegrate and exit.
    Exit(ExitCode),
    /// Open the desktop window.
    Run(LaunchAction),
}

/// Parse `std::env::args` after the program name.
pub fn parse(args: impl IntoIterator<Item = String>) -> Startup {
    let mut args = args.into_iter().peekable();
    match args.peek().map(String::as_str) {
        Some("--integrate") => Startup::Exit(finish(run_integrate())),
        Some("--unintegrate") => Startup::Exit(finish(run_unintegrate())),
        Some("--extract-to") => {
            args.next();
            match collect_paths(&mut args) {
                Ok(paths) => Startup::Run(LaunchAction::ExtractTo(paths)),
                Err(err) => Startup::Exit(usage(&err)),
            }
        }
        Some("--extract-here") => {
            args.next();
            match collect_paths(&mut args) {
                Ok(paths) => Startup::Run(LaunchAction::ExtractHere(paths)),
                Err(err) => Startup::Exit(usage(&err)),
            }
        }
        Some("--extract-here-named") => {
            args.next();
            match collect_paths(&mut args) {
                Ok(paths) => Startup::Run(LaunchAction::ExtractHereNamed(paths)),
                Err(err) => Startup::Exit(usage(&err)),
            }
        }
        Some(flag) if flag.starts_with('-') => {
            Startup::Exit(usage(&format!("unknown option: {flag}")))
        }
        Some(_) => {
            let path = PathBuf::from(args.next().expect("peeked"));
            Startup::Run(LaunchAction::Open(path))
        }
        None => Startup::Run(LaunchAction::Browse),
    }
}

/// Destination for `--extract-here` for one archive.
pub fn extract_here_dir(archive: &std::path::Path) -> PathBuf {
    archive
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Destination for `--extract-here-named` for one archive.
pub fn extract_here_named_dir(archive: &std::path::Path) -> PathBuf {
    extract_here_dir(archive).join(folder_stem(archive))
}

fn collect_paths(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    while let Some(next) = args.peek() {
        if next.starts_with('-') {
            break;
        }
        paths.push(PathBuf::from(args.next().expect("peeked")));
    }
    if paths.is_empty() {
        return Err("missing archive path".into());
    }
    Ok(paths)
}

fn usage(message: &str) -> ExitCode {
    eprintln!("shellzip: {message}");
    ExitCode::from(7)
}

fn finish(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("shellzip: {err}");
            ExitCode::from(2)
        }
    }
}

fn paths_for_integrate() -> IntegratePaths {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("shellzip"));
    let mut paths = IntegratePaths::from_exe(exe);
    // Prefer icons next to the binary (Velopack packDir / installed layout).
    if let Some(parent) = paths.exe.parent() {
        let beside = parent.join("icons");
        if beside.is_dir() {
            paths.icons_source = beside;
        }
        #[cfg(windows)]
        {
            let dll = parent.join("shellzip_shell.dll");
            if dll.is_file() {
                paths.shell_dll = Some(dll);
            }
        }
    }
    // Dev fallback: workspace `grx/shellzip_file_icons` when beside-exe icons are missing.
    if !paths.icons_source.is_dir() {
        let workspace_icons = PathBuf::from("grx").join("shellzip_file_icons");
        if workspace_icons.is_dir() {
            paths.icons_source = workspace_icons;
        } else if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let icons = PathBuf::from(&manifest)
                .join("..")
                .join("..")
                .join("grx")
                .join("shellzip_file_icons");
            if icons.is_dir() {
                paths.icons_source = icons;
            }
        }
    }
    paths
}

/// Install file associations for the current user.
pub fn run_integrate() -> Result<(), String> {
    integrate(&paths_for_integrate()).map_err(|err| err.to_string())
}

/// Remove file associations for the current user.
pub fn run_unintegrate() -> Result<(), String> {
    unintegrate(&paths_for_integrate()).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verbs() {
        assert!(matches!(
            parse(Vec::<String>::new()),
            Startup::Run(LaunchAction::Browse)
        ));
        assert!(matches!(
            parse(vec!["a.zip".into()]),
            Startup::Run(LaunchAction::Open(_))
        ));
        match parse(vec!["--extract-here".into(), "a.zip".into(), "b.7z".into()]) {
            Startup::Run(LaunchAction::ExtractHere(paths)) => {
                assert_eq!(paths.len(), 2);
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(
            extract_here_named_dir(std::path::Path::new("backup.tar.gz")),
            PathBuf::from(".").join("backup")
        );
    }
}
