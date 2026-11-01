//! Start, replace, password, and error prompts.
//!
//! A console session reads a line. On Windows, a double-clicked stub has no
//! console, so the same questions use `MessageBoxW` and a password dialog.

use std::io::{self, IsTerminal, Write};

use shellzip_core::{Error, Password};

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

/// Read a password. `Ok(None)` means the dialog was cancelled.
///
/// An empty password means the person confirmed a blank value. Callers decide
/// whether that is an error. The secret is wiped when the [`Password`] is dropped.
///
/// # Errors
///
/// Returns [`Error::Io`] when the console or the password dialog cannot be used,
/// and [`Error::Usage`] when the text cannot be stored as a password.
pub fn password(text: &str, gui: bool) -> Result<Option<Password>, Error> {
    if gui {
        password_gui(text)
    } else {
        password_console(text)
    }
}

fn password_console(text: &str) -> Result<Option<Password>, Error> {
    eprint!("{text} ");
    io::stderr()
        .flush()
        .map_err(|err| Error::io("write password prompt", err))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|err| Error::io("read password", err))?;
    while line.ends_with(['\n', '\r']) {
        line.pop();
    }
    Ok(Some(Password::new(line)?))
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

#[cfg(windows)]
fn password_gui(text: &str) -> Result<Option<Password>, Error> {
    let words = dword_words(&password_dialog_template(text));
    let mut slot = PasswordSlot {
        units: [0; PASSWORD_UNITS],
        len: 0,
    };
    let code = show_password_dialog(&words, &mut slot)?;
    match code {
        1 => seal_slot(&mut slot).map(Some),
        2 => Ok(None),
        _ => Err(Error::io(
            "show password dialog",
            io::Error::other(format!("dialog closed with status {code}")),
        )),
    }
}

#[cfg(not(windows))]
fn password_gui(text: &str) -> Result<Option<Password>, Error> {
    password_console(text)
}

/// Copy the dialog text into a [`Password`] and wipe the UTF-16 buffer.
///
/// [`Password`] drops wipe the UTF-8 copy. The slot must not keep its own.
#[cfg(windows)]
fn seal_slot(slot: &mut PasswordSlot) -> Result<Password, Error> {
    use zeroize::Zeroize;

    let text = password_text(slot);
    slot.units.zeroize();
    slot.len.zeroize();
    Password::new(text?)
}

#[cfg(windows)]
fn password_text(slot: &PasswordSlot) -> Result<String, Error> {
    let len = usize::try_from(slot.len).unwrap_or(0).min(slot.units.len());
    String::from_utf16(&slot.units[..len])
        .map_err(|_| Error::Usage("password is not valid Unicode".to_string()))
}

/// Edit-control id inside [`password_dialog_template`].
#[cfg(any(windows, test))]
const PASSWORD_ID: u16 = 100;

/// Characters the password edit accepts, including the terminating NUL.
#[cfg(windows)]
const PASSWORD_UNITS: usize = 1024;

/// `ES_PASSWORD`. Masks the edit control. Declared here so the template does
/// not depend on the common-controls feature.
#[cfg(any(windows, test))]
const ES_PASSWORD: u32 = 0x0020;

/// In-memory `DLGTEMPLATE` for a password question.
///
/// Dialog units. The edit control uses [`ES_PASSWORD`]. Item records are
/// aligned to 4 bytes, which `DialogBoxIndirectParamW` requires.
///
/// Window and dialog styles are the Win32 values. They live here so the
/// template can be checked without linking the controls feature.
#[cfg(any(windows, test))]
fn password_dialog_template(prompt: &str) -> Vec<u8> {
    const WS_POPUP: u32 = 0x8000_0000;
    const WS_CAPTION: u32 = 0x00C0_0000;
    const WS_SYSMENU: u32 = 0x0008_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_BORDER: u32 = 0x0080_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const DS_SETFONT: u32 = 0x0000_0040;
    const DS_MODALFRAME: u32 = 0x0000_0080;
    const DS_SETFOREGROUND: u32 = 0x0000_0200;
    const DS_CENTER: u32 = 0x0000_0800;
    const BS_DEFPUSHBUTTON: u32 = 0x0000_0001;

    let style = WS_POPUP
        | WS_CAPTION
        | WS_SYSMENU
        | DS_MODALFRAME
        | DS_CENTER
        | DS_SETFONT
        | DS_SETFOREGROUND;
    let mut buf = Vec::with_capacity(256);
    push_u32(&mut buf, style);
    push_u32(&mut buf, 0);
    push_u16(&mut buf, 4);
    push_u16(&mut buf, 0);
    push_u16(&mut buf, 0);
    push_u16(&mut buf, 240);
    push_u16(&mut buf, 76);
    push_u16(&mut buf, 0);
    push_u16(&mut buf, 0);
    push_wide(&mut buf, "ShellZip");
    push_u16(&mut buf, 9);
    push_wide(&mut buf, "Segoe UI");
    push_item(
        &mut buf,
        DialogControl {
            style: WS_CHILD | WS_VISIBLE,
            x: 12,
            y: 10,
            width: 216,
            height: 12,
            id: 0xFFFF,
            class: CLASS_STATIC,
            title: prompt,
        },
    );
    push_item(
        &mut buf,
        DialogControl {
            style: WS_CHILD | WS_VISIBLE | WS_BORDER | WS_TABSTOP | ES_PASSWORD | ES_AUTOHSCROLL,
            x: 12,
            y: 28,
            width: 216,
            height: 14,
            id: PASSWORD_ID,
            class: CLASS_EDIT,
            title: "",
        },
    );
    push_item(
        &mut buf,
        DialogControl {
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON,
            x: 108,
            y: 52,
            width: 52,
            height: 14,
            id: 1,
            class: CLASS_BUTTON,
            title: "OK",
        },
    );
    push_item(
        &mut buf,
        DialogControl {
            // `BS_PUSHBUTTON` is zero, so the visible style is the window style.
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            x: 168,
            y: 52,
            width: 60,
            height: 14,
            id: 2,
            class: CLASS_BUTTON,
            title: "Cancel",
        },
    );
    buf
}

/// `ES_AUTOHSCROLL` so a long password can be typed in the edit control.
#[cfg(any(windows, test))]
const ES_AUTOHSCROLL: u32 = 0x0080;

/// Predefined dialog class atoms: button, edit, static.
#[cfg(any(windows, test))]
const CLASS_BUTTON: u16 = 0x0080;
#[cfg(any(windows, test))]
const CLASS_EDIT: u16 = 0x0081;
#[cfg(any(windows, test))]
const CLASS_STATIC: u16 = 0x0082;

#[cfg(any(windows, test))]
struct DialogControl<'a> {
    style: u32,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    id: u16,
    class: u16,
    title: &'a str,
}

#[cfg(any(windows, test))]
fn push_item(buf: &mut Vec<u8>, item: DialogControl<'_>) {
    while !buf.len().is_multiple_of(4) {
        buf.push(0);
    }
    push_u32(buf, item.style);
    push_u32(buf, 0);
    push_u16(buf, item.x);
    push_u16(buf, item.y);
    push_u16(buf, item.width);
    push_u16(buf, item.height);
    push_u16(buf, item.id);
    push_u16(buf, 0xFFFF);
    push_u16(buf, item.class);
    push_wide(buf, item.title);
    push_u16(buf, 0);
}

#[cfg(any(windows, test))]
fn push_u16(buf: &mut Vec<u8>, value: u16) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[cfg(any(windows, test))]
fn push_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[cfg(any(windows, test))]
fn push_wide(buf: &mut Vec<u8>, text: &str) {
    for unit in text.encode_utf16() {
        push_u16(buf, unit);
    }
    push_u16(buf, 0);
}

/// Copy `bytes` into `u32`s so the dialog template is 4-byte aligned.
#[cfg(any(windows, test))]
fn dword_words(bytes: &[u8]) -> Vec<u32> {
    let mut padded = bytes.to_vec();
    while !padded.len().is_multiple_of(4) {
        padded.push(0);
    }
    let (chunks, _) = padded.as_chunks::<4>();
    chunks.iter().copied().map(u32::from_le_bytes).collect()
}

#[cfg(windows)]
struct PasswordSlot {
    units: [u16; PASSWORD_UNITS],
    len: u32,
}

#[cfg(windows)]
impl Drop for PasswordSlot {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.units.zeroize();
        self.len.zeroize();
    }
}

#[cfg(windows)]
#[allow(unsafe_code)] // DialogBoxIndirectParamW has no safe wrapper
fn show_password_dialog(template: &[u32], slot: &mut PasswordSlot) -> Result<isize, Error> {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{DLGTEMPLATE, DialogBoxIndirectParamW};

    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    if instance.is_null() {
        return Err(Error::io(
            "show password dialog",
            io::Error::last_os_error(),
        ));
    }
    let code = unsafe {
        DialogBoxIndirectParamW(
            instance,
            template.as_ptr().cast::<DLGTEMPLATE>(),
            std::ptr::null_mut(),
            Some(password_dialog_proc),
            std::ptr::from_mut(slot) as isize,
        )
    };
    if code == -1 {
        return Err(Error::io(
            "show password dialog",
            io::Error::last_os_error(),
        ));
    }
    Ok(code)
}

#[cfg(windows)]
#[allow(unsafe_code)] // dialog procedure is FFI
unsafe extern "system" fn password_dialog_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    wparam: windows_sys::Win32::Foundation::WPARAM,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> isize {
    use windows_sys::Win32::Foundation::WPARAM;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EndDialog, GWLP_USERDATA, SetWindowLongPtrW, WM_CLOSE, WM_COMMAND, WM_INITDIALOG,
    };

    let command_id = |wparam: WPARAM| u16::try_from(wparam & 0xffff).unwrap_or(0);
    match msg {
        WM_INITDIALOG => {
            unsafe {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, lparam);
            }
            1
        }
        WM_COMMAND if command_id(wparam) == 1 => {
            unsafe {
                accept_password(hwnd);
            }
            1
        }
        WM_COMMAND if command_id(wparam) == 2 => {
            unsafe {
                let _ = EndDialog(hwnd, 2);
            }
            1
        }
        WM_CLOSE => {
            unsafe {
                let _ = EndDialog(hwnd, 2);
            }
            1
        }
        _ => 0,
    }
}

#[cfg(windows)]
#[allow(unsafe_code)] // reads the password edit through the dialog procedure
unsafe fn accept_password(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EndDialog, GWLP_USERDATA, GetDlgItemTextW, GetWindowLongPtrW, SetDlgItemTextW,
    };

    let slot = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut PasswordSlot;
    if !slot.is_null() {
        let len = unsafe {
            GetDlgItemTextW(
                hwnd,
                i32::from(PASSWORD_ID),
                (*slot).units.as_mut_ptr(),
                i32::try_from(PASSWORD_UNITS).unwrap_or(i32::MAX),
            )
        };
        unsafe {
            (*slot).len = len;
        }
    }
    let empty = [0u16];
    unsafe {
        let _ = SetDlgItemTextW(hwnd, i32::from(PASSWORD_ID), empty.as_ptr());
        let _ = EndDialog(hwnd, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_dialog_has_a_masked_edit_and_ok_cancel() {
        let bytes = password_dialog_template("Enter the password");
        let cdit = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(cdit, 4);
        assert_eq!(&bytes[22..24], &u16::to_le_bytes(u16::from(b'S')));

        let controls = dialog_controls(&bytes);
        assert_eq!(controls.len(), usize::from(cdit));
        assert!(
            controls.iter().any(|control| {
                control.class == CLASS_EDIT
                    && control.id == PASSWORD_ID
                    && control.style & ES_PASSWORD != 0
                    && control.style & 0x1000_0000 != 0
            }),
            "masked edit missing: {controls:?}"
        );
        assert!(
            controls
                .iter()
                .any(|control| control.class == CLASS_BUTTON && control.id == 1)
        );
        assert!(
            controls
                .iter()
                .any(|control| control.class == CLASS_BUTTON && control.id == 2)
        );
        assert!(
            controls
                .iter()
                .any(|control| control.title == "Enter the password")
        );
    }

    #[test]
    fn dword_words_keep_the_template_bytes() {
        let bytes = password_dialog_template("Password");
        let words = dword_words(&bytes);
        let mut back = Vec::new();
        for word in words {
            back.extend_from_slice(&word.to_le_bytes());
        }
        assert_eq!(&back[..bytes.len()], bytes.as_slice());
        assert!(back.len().is_multiple_of(4));
    }

    #[cfg(windows)]
    #[test]
    #[allow(unsafe_code)] // creates a hidden dialog to validate the template
    fn windows_accepts_the_password_dialog_template() {
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateDialogIndirectParamW, DLGTEMPLATE, DestroyWindow,
        };

        let _lock = dialog_test_lock();
        let words = dword_words(&password_dialog_template("Enter the password"));
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        assert!(!instance.is_null());
        let hwnd = unsafe {
            CreateDialogIndirectParamW(
                instance,
                words.as_ptr().cast::<DLGTEMPLATE>(),
                std::ptr::null_mut(),
                Some(password_dialog_proc),
                0,
            )
        };
        if hwnd.is_null() {
            panic!(
                "password dialog template was rejected: {}",
                std::io::Error::last_os_error()
            );
        }
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
    }

    #[cfg(windows)]
    #[test]
    #[allow(unsafe_code)] // drives the password dialog without a person
    fn password_dialog_returns_what_was_typed() {
        use std::ffi::OsStr;
        use std::time::{Duration, Instant};

        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, PostMessageW, SetDlgItemTextW, WM_CLOSE, WM_COMMAND,
        };

        let _lock = dialog_test_lock();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(password_gui("Enter the password"));
        });

        let class = super::wide(OsStr::new("#32770"));
        let title = super::wide(OsStr::new("ShellZip"));
        let started = Instant::now();
        let hwnd = loop {
            let hwnd = unsafe { FindWindowW(class.as_ptr(), title.as_ptr()) };
            if !hwnd.is_null() {
                break hwnd;
            }
            if started.elapsed() > Duration::from_secs(3) {
                panic!("password dialog did not appear");
            }
            std::thread::sleep(Duration::from_millis(10));
        };

        let typed = super::wide(OsStr::new("sfx-dialog-password"));
        unsafe {
            let _ = SetDlgItemTextW(hwnd, i32::from(PASSWORD_ID), typed.as_ptr());
            let _ = PostMessageW(hwnd, WM_COMMAND, 1, 0);
        }

        let result = match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(result) => result,
            Err(_) => {
                unsafe {
                    let _ = PostMessageW(hwnd, WM_CLOSE, 0, 0);
                }
                panic!("password dialog did not return");
            }
        };
        let password = result.expect("dialog").expect("typed password");
        assert_eq!(password.expose(), "sfx-dialog-password");
        drop(password);
    }

    #[cfg(windows)]
    fn dialog_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[derive(Debug)]
    struct Control {
        style: u32,
        id: u16,
        class: u16,
        title: String,
    }

    fn dialog_controls(bytes: &[u8]) -> Vec<Control> {
        let mut index = 22;
        index = skip_wide(bytes, index);
        index += 2;
        index = skip_wide(bytes, index);
        let mut controls = Vec::new();
        while index < bytes.len() {
            while !index.is_multiple_of(4) {
                index += 1;
            }
            if index >= bytes.len() {
                break;
            }
            let style = read_u32(bytes, index);
            index += 8;
            index += 8;
            let id = read_u16(bytes, index);
            index += 2;
            assert_eq!(read_u16(bytes, index), 0xFFFF, "class atom marker");
            index += 2;
            let class = read_u16(bytes, index);
            index += 2;
            let (title, next) = take_wide(bytes, index);
            index = next;
            let extra = usize::from(read_u16(bytes, index));
            index += 2 + extra;
            controls.push(Control {
                style,
                id,
                class,
                title,
            });
        }
        controls
    }

    fn read_u16(bytes: &[u8], index: usize) -> u16 {
        u16::from_le_bytes([bytes[index], bytes[index + 1]])
    }

    fn read_u32(bytes: &[u8], index: usize) -> u32 {
        u32::from_le_bytes(bytes[index..index + 4].try_into().unwrap())
    }

    fn skip_wide(bytes: &[u8], index: usize) -> usize {
        take_wide(bytes, index).1
    }

    fn take_wide(bytes: &[u8], mut index: usize) -> (String, usize) {
        let mut units = Vec::new();
        loop {
            let unit = read_u16(bytes, index);
            index += 2;
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        (String::from_utf16(&units).unwrap(), index)
    }
}
