//! Folder dialog for a Linux self-extractor started without a terminal.
//!
//! A terminal session keeps the console prompts. A double-clicked package asks
//! zenity (GNOME) or kdialog (KDE) for the destination, then reports the
//! result. Arguments are passed as a list, never through a shell. `-s` does
//! not open a dialog. `SZIP_SFX_DIALOG` forces the dialog in tests.

use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use shellzip_core::{Error, ExtractOpts, NopProgress, Overwrite, Password, extract, list};
use zeroize::Zeroize;

/// A desktop tool that can ask for a folder.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Zenity,
    Kdialog,
}

struct Tool {
    kind: Kind,
    path: PathBuf,
}

/// Whether this launch should ask with zenity or kdialog.
pub(crate) fn should_ask() -> bool {
    let forced = std::env::var_os("SZIP_SFX_DIALOG").is_some();
    if std::env::var_os("CI").is_some() && !forced {
        return false;
    }
    if !forced && (io::stdin().is_terminal() || io::stderr().is_terminal()) {
        return false;
    }
    if !forced
        && std::env::var_os("DISPLAY").is_none()
        && std::env::var_os("WAYLAND_DISPLAY").is_none()
    {
        return false;
    }
    find_tool().is_some()
}

/// Show `text`. A terminal session already prints it, and CI must not block.
pub(crate) fn tell(text: &str, error: bool) {
    if io::stderr().is_terminal() {
        return;
    }
    if std::env::var_os("CI").is_some() && std::env::var_os("SZIP_SFX_DIALOG").is_none() {
        return;
    }
    if let Some(tool) = find_tool() {
        let _ = message(&tool, text, error);
        return;
    }
    if let Some(path) = executable("notify-send") {
        let _ = Command::new(path).arg("ShellZip").arg(text).status();
    }
}

/// Ask for a folder and extract `exe` there.
///
/// Cancel returns `Ok(0)` and writes nothing. A dialog failure is [`Error::Io`].
///
/// # Errors
///
/// Returns the extract error when the payload cannot be written. The caller
/// shows that text; this function only shows the success notice.
pub(crate) fn run(exe: &Path, dest: &Path, supplied: Option<Password>) -> Result<i32, Error> {
    let tool = find_tool().ok_or_else(|| {
        Error::Usage("no folder dialog is available; rerun with -d or install zenity".to_string())
    })?;
    let Some(chosen) = pick_dir(&tool, dest)? else {
        return Ok(0);
    };
    let from_env = supplied.is_some();
    let mut password = if from_env {
        supplied
    } else {
        match password_for(exe, &tool)? {
            PasswordAsk::Cancel => return Ok(0),
            PasswordAsk::None => None,
            PasswordAsk::Value(password) => Some(password),
        }
    };
    loop {
        let mut opts = ExtractOpts::new(&chosen);
        opts.password = password.take();
        opts.overwrite = Overwrite::Always;
        let result = extract(exe, &opts, None, &NopProgress);
        opts.password = None;
        match result {
            Ok(report) => {
                let _ = message(
                    &tool,
                    &format!("Extracted {} to {}", report.extracted, chosen.display()),
                    false,
                );
                return Ok(0);
            }
            Err(Error::BadPassword) if !from_env => {
                match prompt_password(&tool, "That password was incorrect. Enter it again.")? {
                    None => return Ok(0),
                    Some(value) => password = Some(value),
                }
            }
            Err(err) => return Err(err),
        }
    }
}

enum PasswordAsk {
    None,
    Cancel,
    Value(Password),
}

fn password_for(exe: &Path, tool: &Tool) -> Result<PasswordAsk, Error> {
    let encrypted = match list(exe) {
        Ok((_, entries)) => entries.iter().any(|entry| entry.encrypted),
        Err(Error::Encrypted(_)) => true,
        Err(err) => return Err(err),
    };
    if !encrypted {
        return Ok(PasswordAsk::None);
    }
    match prompt_password(tool, "Enter the password to extract this archive.")? {
        None => Ok(PasswordAsk::Cancel),
        Some(password) => Ok(PasswordAsk::Value(password)),
    }
}

fn prompt_password(tool: &Tool, mut text: &str) -> Result<Option<Password>, Error> {
    loop {
        match ask_password(tool, text)? {
            None => return Ok(None),
            Some(password) if password.expose().is_empty() => {
                drop(password);
                text = "Enter a password.";
            }
            Some(password) => return Ok(Some(password)),
        }
    }
}

fn find_tool() -> Option<Tool> {
    if let Some(path) = executable("zenity") {
        return Some(Tool {
            kind: Kind::Zenity,
            path,
        });
    }
    if let Some(path) = executable("kdialog") {
        return Some(Tool {
            kind: Kind::Kdialog,
            path,
        });
    }
    None
}

fn executable(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn pick_dir(tool: &Tool, initial: &Path) -> Result<Option<PathBuf>, Error> {
    let output = spawn(tool, &pick_args(tool.kind, initial))?;
    match interpret(output.status.code(), output.stdout)? {
        None => Ok(None),
        Some(bytes) => {
            let path = trim_output(bytes);
            if path.is_empty() {
                return Err(Error::Usage(
                    "the folder dialog returned an empty path".to_string(),
                ));
            }
            Ok(Some(PathBuf::from(path)))
        }
    }
}

fn ask_password(tool: &Tool, prompt: &str) -> Result<Option<Password>, Error> {
    let output = spawn(tool, &password_args(tool.kind, prompt))?;
    match interpret(output.status.code(), output.stdout)? {
        None => Ok(None),
        Some(bytes) => Ok(Some(password_from_output(bytes)?)),
    }
}

fn message(tool: &Tool, text: &str, error: bool) -> Result<(), Error> {
    let output = spawn(tool, &message_args(tool.kind, text, error))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::io(
            "show extract dialog",
            io::Error::other("dialog did not close"),
        ))
    }
}

fn spawn(tool: &Tool, args: &[OsString]) -> Result<Output, Error> {
    Command::new(&tool.path)
        .args(args)
        .output()
        .map_err(|err| Error::io(format!("run {}", tool.path.display()), err))
}

fn pick_args(kind: Kind, initial: &Path) -> Vec<OsString> {
    match kind {
        Kind::Zenity => args([
            "--file-selection",
            "--directory",
            "--title",
            "ShellZip self-extracting archive",
            "--filename",
        ])
        .into_iter()
        .chain(std::iter::once(with_slash(initial)))
        .collect(),
        Kind::Kdialog => args(["--title", "ShellZip self-extracting archive"])
            .into_iter()
            .chain([
                OsString::from("--getexistingdirectory"),
                initial.as_os_str().to_os_string(),
            ])
            .collect(),
    }
}

fn password_args(kind: Kind, prompt: &str) -> Vec<OsString> {
    match kind {
        Kind::Zenity => args(["--password", "--title"])
            .into_iter()
            .chain(std::iter::once(OsString::from(prompt)))
            .collect(),
        Kind::Kdialog => args(["--title", "ShellZip"])
            .into_iter()
            .chain([OsString::from("--password"), OsString::from(prompt)])
            .collect(),
    }
}

fn message_args(kind: Kind, text: &str, error: bool) -> Vec<OsString> {
    match kind {
        Kind::Zenity => {
            let which = if error { "--error" } else { "--info" };
            args([which, "--title", "ShellZip", "--no-markup", "--text"])
                .into_iter()
                .chain(std::iter::once(OsString::from(text)))
                .collect()
        }
        Kind::Kdialog => {
            let which = if error { "--error" } else { "--msgbox" };
            args(["--title", "ShellZip"])
                .into_iter()
                .chain([OsString::from(which), OsString::from(text)])
                .collect()
        }
    }
}

fn args<const N: usize>(parts: [&str; N]) -> Vec<OsString> {
    parts.into_iter().map(OsString::from).collect()
}

fn with_slash(path: &Path) -> OsString {
    let mut text = path.as_os_str().to_os_string();
    text.push("/");
    text
}

fn interpret(code: Option<i32>, stdout: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
    match code {
        Some(0) => Ok(Some(stdout)),
        Some(1) => Ok(None),
        Some(code) => Err(Error::io(
            "show extract dialog",
            io::Error::other(format!("dialog exited {code}")),
        )),
        None => Err(Error::io(
            "show extract dialog",
            io::Error::other("dialog was interrupted"),
        )),
    }
}

fn trim_output(mut bytes: Vec<u8>) -> OsString {
    use std::os::unix::ffi::OsStringExt;

    while matches!(bytes.last(), Some(b'\n' | b'\r')) {
        bytes.pop();
    }
    OsString::from_vec(bytes)
}

fn password_from_output(bytes: Vec<u8>) -> Result<Password, Error> {
    use std::os::unix::ffi::OsStringExt;

    match trim_output(bytes).into_string() {
        Ok(text) => Password::new(text),
        Err(raw) => {
            let mut bytes = raw.into_vec();
            bytes.zeroize();
            Err(Error::Usage("password is not valid Unicode".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn zenity_opens_the_starting_folder() {
        let args = pick_args(Kind::Zenity, Path::new("/tmp/out"));
        assert!(args.iter().any(|arg| arg == OsStr::new("--directory")));
        assert!(args.iter().any(|arg| arg == OsStr::new("/tmp/out/")));
    }

    #[test]
    fn kdialog_asks_for_an_existing_directory() {
        let args = pick_args(Kind::Kdialog, Path::new("/tmp/out"));
        assert!(
            args.iter()
                .any(|arg| arg == OsStr::new("--getexistingdirectory"))
        );
        assert!(args.iter().any(|arg| arg == OsStr::new("/tmp/out")));
    }

    #[test]
    fn cancel_is_not_an_error() {
        let output = interpret(Some(1), b"ignored".to_vec()).unwrap();
        assert!(output.is_none());
    }

    #[test]
    fn success_trims_the_folder() {
        let bytes = interpret(Some(0), b"/tmp/out\n".to_vec()).unwrap().unwrap();
        assert_eq!(trim_output(bytes), OsStr::new("/tmp/out"));
    }

    #[test]
    fn a_failed_dialog_is_an_error() {
        let err = interpret(Some(2), Vec::new()).unwrap_err();
        assert_eq!(err.exit_code(), 2, "{err}");
    }
}
