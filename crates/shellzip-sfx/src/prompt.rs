//! Start, replace, and error prompts.
//!
//! A console session reads a line. On Windows, a double-clicked stub has no
//! console, so the same questions use `MessageBoxW`.

use std::io::{self, IsTerminal, Write};

use shellzip_core::Error;

/// Answer to a yes/no question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// Replace this file, or start extracting.
    Yes,
    /// Leave this file, or decline the start question.
    No,
    /// Replace this file and every later collision. Console only.
    All,
    /// Stop the extract.
    Cancel,
}

/// Attach to the parent console so a Windows-subsystem stub can print from a terminal.
pub fn attach_console() {
    #[cfg(windows)]
    attach_parent();
}

/// Dialogs instead of stdin. CI never shows a dialog, so a failed test cannot block.
pub fn gui_session() -> bool {
    cfg!(windows) && std::env::var_os("CI").is_none() && !io::stderr().is_terminal()
}

/// Whether a person can answer a prompt.
pub fn can_ask(gui: bool) -> bool {
    gui || io::stdin().is_terminal()
}

/// Ask `text`. `allow_all` accepts `a` on the console.
///
/// # Errors
///
/// Returns [`Error::Io`] when reading the console fails.
pub fn ask(text: &str, gui: bool, allow_all: bool) -> Result<Choice, Error> {
    if gui {
        Ok(ask_gui(text))
    } else {
        ask_console(text, allow_all)
    }
}

/// Show `err`. A GUI session also gets a dialog.
pub fn report(err: &Error, gui: bool) {
    if gui {
        info(err.to_string().as_str());
    }
    let _ = writeln!(io::stderr(), "shellzip-sfx: {err}");
}

/// Show `text` after a successful extract when not silent.
pub fn notice(text: &str, gui: bool) {
    if gui {
        info(text);
    } else {
        println!("{text}");
    }
}

fn ask_console(text: &str, allow_all: bool) -> Result<Choice, Error> {
    let hint = if allow_all {
        "[y]es [n]o [a]ll [q]uit"
    } else {
        "[y/N]"
    };
    println!("{text} {hint}");
    io::stdout()
        .flush()
        .map_err(|err| Error::io("write prompt", err))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|err| Error::io("read prompt", err))?;
    while line.ends_with(['\n', '\r']) {
        line.pop();
    }
    let choice = match line.trim() {
        "y" | "Y" | "yes" => Choice::Yes,
        "a" | "A" | "all" if allow_all => Choice::All,
        "q" | "Q" | "quit" => Choice::Cancel,
        _ => Choice::No,
    };
    Ok(choice)
}

#[cfg(windows)]
fn ask_gui(text: &str) -> Choice {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IDCANCEL, IDNO, IDYES, MB_ICONQUESTION, MB_YESNOCANCEL,
    };
    match message_box(text, MB_YESNOCANCEL | MB_ICONQUESTION) {
        IDYES => Choice::Yes,
        IDNO => Choice::No,
        IDCANCEL => Choice::Cancel,
        _ => Choice::Cancel,
    }
}

#[cfg(not(windows))]
fn ask_gui(text: &str) -> Choice {
    ask_console(text, false).unwrap_or(Choice::Cancel)
}

#[cfg(windows)]
fn info(text: &str) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK};
    let _ = message_box(text, MB_OK | MB_ICONINFORMATION);
}

#[cfg(not(windows))]
fn info(text: &str) {
    println!("{text}");
}

/// `MessageBoxW` has no safe wrapper in the standard library.
#[cfg(windows)]
#[allow(unsafe_code)]
fn message_box(text: &str, style: u32) -> i32 {
    use std::ffi::OsStr;

    let body = wide(OsStr::new(text));
    let caption = wide(OsStr::new("ShellZip"));
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            body.as_ptr(),
            caption.as_ptr(),
            style,
        )
    }
}

#[cfg(windows)]
fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    text.encode_wide().chain(std::iter::once(0)).collect()
}

/// `AttachConsole` has no safe wrapper in the standard library.
#[cfg(windows)]
#[allow(unsafe_code)]
fn attach_parent() {
    unsafe {
        let _ = windows_sys::Win32::System::Console::AttachConsole(u32::MAX);
    }
}
