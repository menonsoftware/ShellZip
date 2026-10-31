//! Setup window for a double-clicked self-extractor.
//!
//! The window shows the ShellZip banner, instructions, a destination folder,
//! Browse, a progress bar, and Extract / Cancel.
//! Browse opens the system Browse For Folder dialog.

/// Child-control ids. Extract is `IDOK` (1) and Cancel is `IDCANCEL` (2).
const ID_INTRO: i32 = 101;
const ID_DEST: i32 = 103;
const ID_BROWSE: i32 = 104;
const ID_PROGRESS: i32 = 106;

const INTRO: &str = "\
\u{2022} Press Extract to start extraction.\r\n\
\r\n\
\u{2022} Use Browse to select the destination folder from the folder tree. The path can also be typed.\r\n\
\r\n\
\u{2022} If the destination folder does not exist, it is created before extraction.\r\n";

/// In-memory `DLGTEMPLATE` for the setup window.
///
/// The progress bar uses the common-controls class `msctls_progress32`.
/// That class is registered before the dialog is created.
fn setup_template() -> Vec<u8> {
    const WS_POPUP: u32 = 0x8000_0000;
    const WS_CAPTION: u32 = 0x00C0_0000;
    const WS_SYSMENU: u32 = 0x0008_0000;
    const WS_MINIMIZEBOX: u32 = 0x0002_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_BORDER: u32 = 0x0080_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const WS_VSCROLL: u32 = 0x0020_0000;
    const DS_SETFONT: u32 = 0x0000_0040;
    const DS_MODALFRAME: u32 = 0x0000_0080;
    const DS_SETFOREGROUND: u32 = 0x0000_0200;
    const DS_CENTER: u32 = 0x0000_0800;
    const BS_DEFPUSHBUTTON: u32 = 0x0000_0001;
    const SS_OWNERDRAW: u32 = 0x0000_000D;
    const ES_MULTILINE: u32 = 0x0000_0004;
    const ES_AUTOVSCROLL: u32 = 0x0000_0040;
    const ES_READONLY: u32 = 0x0000_0800;
    const CBS_DROPDOWN: u32 = 0x0000_0002;
    const CBS_AUTOHSCROLL: u32 = 0x0000_0040;
    const CBS_HASSTRINGS: u32 = 0x0000_0200;
    const PBS_SMOOTH: u32 = 0x0000_0001;
    const CLASS_BUTTON: u16 = 0x0080;
    const CLASS_EDIT: u16 = 0x0081;
    const CLASS_STATIC: u16 = 0x0082;
    const CLASS_COMBO: u16 = 0x0085;

    let style = WS_POPUP
        | WS_CAPTION
        | WS_SYSMENU
        | WS_MINIMIZEBOX
        | DS_MODALFRAME
        | DS_CENTER
        | DS_SETFONT
        | DS_SETFOREGROUND;
    let mut buf = Vec::with_capacity(512);
    push_u32(&mut buf, style);
    push_u32(&mut buf, 0);
    push_u16(&mut buf, 9);
    push_u16(&mut buf, 0);
    push_u16(&mut buf, 0);
    push_u16(&mut buf, 340);
    push_u16(&mut buf, 350);
    push_u16(&mut buf, 0);
    push_u16(&mut buf, 0);
    push_wide(&mut buf, "ShellZip self-extracting archive");
    push_u16(&mut buf, 9);
    push_wide(&mut buf, "Segoe UI");

    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE | SS_OWNERDRAW,
            x: 0,
            y: 0,
            width: 340,
            height: 176,
            id: 100,
            class: CLASS_STATIC,
            title: "",
        },
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE | WS_BORDER | ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL,
            x: 8,
            y: 184,
            width: 324,
            height: 76,
            id: ID_INTRO as u16,
            class: CLASS_EDIT,
            title: "",
        },
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE,
            x: 8,
            y: 266,
            width: 210,
            height: 10,
            id: 102,
            class: CLASS_STATIC,
            title: "Destination folder",
        },
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | WS_VSCROLL
                | CBS_DROPDOWN
                | CBS_AUTOHSCROLL
                | CBS_HASSTRINGS,
            x: 8,
            y: 278,
            width: 228,
            height: 70,
            id: ID_DEST as u16,
            class: CLASS_COMBO,
            title: "",
        },
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            x: 242,
            y: 278,
            width: 90,
            height: 14,
            id: ID_BROWSE as u16,
            class: CLASS_BUTTON,
            title: "Browse...",
        },
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE,
            x: 8,
            y: 298,
            width: 210,
            height: 10,
            id: 105,
            class: CLASS_STATIC,
            title: "Extraction progress",
        },
    );
    push_named(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE | PBS_SMOOTH,
            x: 8,
            y: 310,
            width: 324,
            height: 12,
            id: ID_PROGRESS as u16,
            class: 0,
            title: "",
        },
        "msctls_progress32",
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_DEFPUSHBUTTON,
            x: 188,
            y: 326,
            width: 68,
            height: 16,
            id: 1,
            class: CLASS_BUTTON,
            title: "Extract",
        },
    );
    push_atom(
        &mut buf,
        Control {
            style: WS_CHILD | WS_VISIBLE | WS_TABSTOP,
            x: 264,
            y: 326,
            width: 68,
            height: 16,
            id: 2,
            class: CLASS_BUTTON,
            title: "Cancel",
        },
    );
    buf
}

struct Control {
    style: u32,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    id: u16,
    class: u16,
    title: &'static str,
}

fn push_atom(buf: &mut Vec<u8>, item: Control) {
    align4(buf);
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

fn push_named(buf: &mut Vec<u8>, item: Control, class_name: &str) {
    align4(buf);
    push_u32(buf, item.style);
    push_u32(buf, 0);
    push_u16(buf, item.x);
    push_u16(buf, item.y);
    push_u16(buf, item.width);
    push_u16(buf, item.height);
    push_u16(buf, item.id);
    push_wide(buf, class_name);
    push_wide(buf, item.title);
    push_u16(buf, 0);
}

fn align4(buf: &mut Vec<u8>) {
    while !buf.len().is_multiple_of(4) {
        buf.push(0);
    }
}

fn push_u16(buf: &mut Vec<u8>, value: u16) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn push_wide(buf: &mut Vec<u8>, text: &str) {
    for unit in text.encode_utf16() {
        push_u16(buf, unit);
    }
    push_u16(buf, 0);
}

fn dword_words(bytes: &[u8]) -> Vec<u32> {
    let mut padded = bytes.to_vec();
    while !padded.len().is_multiple_of(4) {
        padded.push(0);
    }
    let (chunks, _) = padded.as_chunks::<4>();
    chunks.iter().copied().map(u32::from_le_bytes).collect()
}

#[cfg(windows)]
#[cfg(windows)]
#[allow(unsafe_code, unsafe_op_in_unsafe_fn)]
mod win {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::mpsc;

    use shellzip_core::{Error, ExtractOpts, Overwrite, Password, ProgressSink, extract, list};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EndDialog, GWLP_USERDATA, GetDlgItem, GetParent, GetWindowLongPtrW, GetWindowTextLengthW,
        GetWindowTextW, PostMessageW, SendMessageW, SetTimer, SetWindowLongPtrW, SetWindowTextW,
        WM_CLOSE, WM_COMMAND, WM_DESTROY, WM_DRAWITEM, WM_INITDIALOG, WM_TIMER,
    };

    use super::{ID_BROWSE, ID_DEST, ID_INTRO, ID_PROGRESS, INTRO, dword_words, setup_template};

    const WM_EXTRACT_DONE: u32 = 0x8000 + 1;
    const PBM_SETPOS: u32 = 0x0402;
    const PBM_SETRANGE32: u32 = 0x0406;
    const CB_ADDSTRING: u32 = 0x0143;
    const WM_SETICON: u32 = 0x0080;
    const TIMER_PROGRESS: usize = 1;

    /// Show the setup window and extract when the person presses Extract.
    ///
    /// Cancel closes the window without an error. A password is requested only
    /// when the archive is encrypted and `password` is empty.
    pub(crate) fn run(
        exe: &Path,
        dest: &Path,
        password: Option<Password>,
    ) -> Result<i32, shellzip_core::Error> {
        let encrypted = match list(exe) {
            Ok((_, entries)) => entries.iter().any(|entry| entry.encrypted),
            Err(shellzip_core::Error::Encrypted(_)) => true,
            Err(err) => return Err(err),
        };
        let com = ComInit::new();
        init_common_controls();
        let mut state = SetupState {
            exe: exe.to_path_buf(),
            hint: dest.to_path_buf(),
            chosen: dest.to_path_buf(),
            encrypted,
            password,
            progress: Arc::new(BarProgress::default()),
            inbox: None,
            worker: None,
            busy: false,
            succeeded: false,
            extracted: 0,
            small_icon: std::ptr::null_mut(),
            big_icon: std::ptr::null_mut(),
            banner: decode_banner(),
        };
        let shown = show(&mut state);
        finish_worker(&mut state);
        let succeeded = state.succeeded;
        let extracted = state.extracted;
        let chosen = state.chosen.clone();
        drop(state);
        drop(com);
        shown?;
        if succeeded {
            crate::prompt::notice(
                &format!("Extracted {extracted} to {}", chosen.display()),
                true,
            );
        }
        Ok(0)
    }

    struct ComInit(bool);

    impl ComInit {
        fn new() -> Self {
            let hr =
                unsafe { windows_sys::Win32::System::Com::CoInitializeEx(std::ptr::null(), 2) };
            Self(hr >= 0)
        }
    }

    impl Drop for ComInit {
        fn drop(&mut self) {
            if self.0 {
                unsafe { windows_sys::Win32::System::Com::CoUninitialize() }
            }
        }
    }

    pub(super) fn init_common_controls() {
        use windows_sys::Win32::UI::Controls::{INITCOMMONCONTROLSEX, InitCommonControlsEx};

        let info = INITCOMMONCONTROLSEX {
            dwSize: u32::try_from(size_of::<INITCOMMONCONTROLSEX>()).unwrap_or(8),
            dwICC: 0x0020, // ICC_PROGRESS_CLASS
        };
        unsafe {
            InitCommonControlsEx(&info);
        }
    }

    struct SetupState {
        exe: PathBuf,
        hint: PathBuf,
        chosen: PathBuf,
        encrypted: bool,
        password: Option<Password>,
        progress: Arc<BarProgress>,
        inbox: Option<mpsc::Receiver<Result<shellzip_core::ExtractReport, Error>>>,
        worker: Option<std::thread::JoinHandle<()>>,
        busy: bool,
        succeeded: bool,
        extracted: u64,
        small_icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON,
        big_icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON,
        banner: Option<Banner>,
    }

    impl Drop for SetupState {
        fn drop(&mut self) {
            unsafe {
                destroy_icon(self.small_icon);
                destroy_icon(self.big_icon);
            }
        }
    }

    #[derive(Default)]
    struct BarProgress {
        total: AtomicU64,
        done: AtomicU64,
        cancel: AtomicBool,
    }

    impl ProgressSink for BarProgress {
        fn begin(&self, _label: &str, total_bytes: u64) {
            self.total.store(total_bytes, Ordering::Relaxed);
            self.done.store(0, Ordering::Relaxed);
        }

        fn begin_entry(&self, _name: &str, _bytes: u64) {}

        fn advance(&self, bytes: u64) {
            self.done.fetch_add(bytes, Ordering::Relaxed);
        }

        fn end_entry(&self) {}

        fn message(&self, _text: &str) {}

        fn is_cancelled(&self) -> bool {
            self.cancel.load(Ordering::Acquire)
        }
    }

    fn show(state: &mut SetupState) -> Result<isize, Error> {
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows_sys::Win32::UI::WindowsAndMessaging::{DLGTEMPLATE, DialogBoxIndirectParamW};

        let words = dword_words(&setup_template());
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        if instance.is_null() {
            return Err(Error::io(
                "show the extract window",
                std::io::Error::last_os_error(),
            ));
        }
        let code = unsafe {
            DialogBoxIndirectParamW(
                instance,
                words.as_ptr().cast::<DLGTEMPLATE>(),
                std::ptr::null_mut(),
                Some(setup_proc),
                std::ptr::from_mut(state) as isize,
            )
        };
        if code == -1 {
            return Err(Error::io(
                "show the extract window",
                std::io::Error::last_os_error(),
            ));
        }
        Ok(code)
    }

    fn finish_worker(state: &mut SetupState) {
        state.progress.cancel.store(true, Ordering::Release);
        if let Some(worker) = state.worker.take() {
            let _ = worker.join();
        }
    }

    #[allow(unsafe_code)] // the dialog stores this pointer for its lifetime
    unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut SetupState> {
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *mut SetupState;
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr })
        }
    }

    pub(super) unsafe extern "system" fn setup_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> isize {
        match msg {
            WM_INITDIALOG => {
                if lparam != 0 {
                    unsafe {
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, lparam);
                        if let Some(state) = state_mut(hwnd) {
                            prepare_window(hwnd, state);
                        }
                    }
                }
                1
            }
            WM_DRAWITEM => {
                unsafe { paint_banner(lparam) }
                1
            }
            WM_COMMAND => {
                let id = u16::try_from(wparam & 0xffff).unwrap_or(0);
                unsafe { on_command(hwnd, id) }
                1
            }
            WM_TIMER => {
                unsafe {
                    paint_progress(hwnd);
                    finish_if_ready(hwnd);
                }
                1
            }
            WM_EXTRACT_DONE => {
                unsafe {
                    paint_progress(hwnd);
                    finish_if_ready(hwnd);
                }
                1
            }
            WM_CLOSE => {
                unsafe { request_close(hwnd) }
                1
            }
            WM_DESTROY => {
                unsafe {
                    let _ = windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(
                        hwnd,
                        TIMER_PROGRESS,
                    );
                }
                1
            }
            _ => 0,
        }
    }

    #[allow(unsafe_code)]
    unsafe fn prepare_window(hwnd: HWND, state: &mut SetupState) {
        set_text(hwnd, ID_INTRO, INTRO);
        install_icons(hwnd, state);
        let combo = GetDlgItem(hwnd, ID_DEST);
        add_choice(combo, &state.hint);
        if let Some(dir) = state.exe.parent()
            && dir != state.hint
        {
            add_choice(combo, dir);
        }
        set_window_text(combo, state.hint.as_os_str());
        let bar = GetDlgItem(hwnd, ID_PROGRESS);
        SendMessageW(bar, PBM_SETRANGE32, 0, 100);
    }

    #[allow(unsafe_code)]
    unsafe fn on_command(hwnd: HWND, id: u16) {
        match id {
            1 => on_extract(hwnd),
            2 => request_close(hwnd),
            id if i32::from(id) == ID_BROWSE => on_browse(hwnd),
            _ => {}
        }
    }

    #[allow(unsafe_code)]
    unsafe fn on_browse(hwnd: HWND) {
        if state_mut(hwnd).is_some_and(|state| state.busy) {
            return;
        }
        let current = read_dest(hwnd).unwrap_or_else(|_| PathBuf::new());
        let Some(picked) = browse_folder(hwnd, &current) else {
            return;
        };
        let combo = GetDlgItem(hwnd, ID_DEST);
        set_window_text(combo, picked.as_os_str());
    }

    #[allow(unsafe_code)]
    unsafe fn on_extract(hwnd: HWND) {
        if state_mut(hwnd).is_some_and(|state| state.busy) {
            return;
        }
        if state_mut(hwnd).is_some_and(|state| state.encrypted && state.password.is_none()) {
            match crate::prompt::password("Enter the password to extract this archive.", true) {
                Ok(Some(password)) if !password.expose().is_empty() => {
                    if let Some(state) = state_mut(hwnd) {
                        state.password = Some(password);
                    }
                }
                Ok(Some(password)) => {
                    drop(password);
                    crate::prompt::report(&Error::Usage("password was empty".to_string()), true);
                    return;
                }
                Ok(None) => return,
                Err(err) => {
                    crate::prompt::report(&err, true);
                    return;
                }
            }
        }
        if let Err(err) = start_extract(hwnd) {
            crate::prompt::report(&err, true);
        }
    }

    #[allow(unsafe_code)]
    unsafe fn start_extract(hwnd: HWND) -> Result<(), Error> {
        let dest = read_dest(hwnd)?;
        prepare_folder(&dest)?;
        let Some(state) = state_mut(hwnd) else {
            return Err(Error::Usage("extract window is not ready".to_string()));
        };
        if state.busy {
            return Ok(());
        }
        let password = state.password.take();
        let exe = state.exe.clone();
        let progress = Arc::clone(&state.progress);
        progress.cancel.store(false, Ordering::Release);
        progress.done.store(0, Ordering::Relaxed);
        progress.total.store(0, Ordering::Relaxed);
        state.chosen = dest.clone();
        state.busy = true;
        let (tx, rx) = mpsc::channel();
        state.inbox = Some(rx);
        if let Some(previous) = state.worker.take() {
            let _ = previous.join();
        }
        let ui = hwnd as isize;
        state.worker = Some(std::thread::spawn(move || {
            let mut opts = ExtractOpts::new(&dest);
            opts.password = password;
            opts.overwrite = Overwrite::Always;
            let result = extract(&exe, &opts, None, progress.as_ref());
            opts.password = None;
            let _ = tx.send(result);
            unsafe {
                let _ = PostMessageW(ui as HWND, WM_EXTRACT_DONE, 0, 0);
            }
        }));
        set_busy(hwnd, true);
        SetTimer(hwnd, TIMER_PROGRESS, 100, None);
        Ok(())
    }

    fn prepare_folder(dest: &Path) -> Result<(), Error> {
        if dest.as_os_str().is_empty() {
            return Err(Error::Usage("Enter a destination folder.".to_string()));
        }
        if dest.is_file() {
            return Err(Error::Usage("Choose a folder, not a file.".to_string()));
        }
        std::fs::create_dir_all(dest)
            .map_err(|err| Error::io(format!("create {}", dest.display()), err))
    }

    #[allow(unsafe_code)]
    unsafe fn finish_if_ready(hwnd: HWND) {
        let result = {
            let Some(state) = state_mut(hwnd) else {
                return;
            };
            if !state.busy {
                return;
            }
            let Some(inbox) = state.inbox.as_ref() else {
                return;
            };
            match inbox.try_recv() {
                Ok(result) => {
                    state.busy = false;
                    result
                }
                Err(_) => return,
            }
        };
        set_busy(hwnd, false);
        let _ = windows_sys::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_PROGRESS);
        match result {
            Ok(report) => {
                if let Some(state) = state_mut(hwnd) {
                    state.password = None;
                    state.extracted = report.extracted;
                    state.succeeded = true;
                }
                EndDialog(hwnd, 1);
            }
            Err(Error::BadPassword) => retry_password(hwnd),
            Err(Error::Interrupted) => {
                EndDialog(hwnd, 2);
            }
            Err(err) => crate::prompt::report(&err, true),
        }
    }

    #[allow(unsafe_code)]
    unsafe fn retry_password(hwnd: HWND) {
        if let Some(state) = state_mut(hwnd) {
            state.password = None;
        }
        match crate::prompt::password("That password was incorrect. Enter it again.", true) {
            Ok(Some(password)) if !password.expose().is_empty() => {
                if let Some(state) = state_mut(hwnd) {
                    state.password = Some(password);
                }
                if let Err(err) = start_extract(hwnd) {
                    crate::prompt::report(&err, true);
                }
            }
            Ok(Some(password)) => drop(password),
            Ok(None) => {}
            Err(err) => crate::prompt::report(&err, true),
        }
    }

    #[allow(unsafe_code)]
    unsafe fn request_close(hwnd: HWND) {
        let busy = state_mut(hwnd).is_some_and(|state| {
            if state.busy {
                state.progress.cancel.store(true, Ordering::Release);
            }
            state.busy
        });
        if !busy {
            EndDialog(hwnd, 2);
        }
    }

    #[allow(unsafe_code)]
    unsafe fn paint_progress(hwnd: HWND) {
        let pos = state_mut(hwnd).map(|state| percent(state)).unwrap_or(0);
        let bar = GetDlgItem(hwnd, ID_PROGRESS);
        if !bar.is_null() {
            SendMessageW(bar, PBM_SETPOS, pos, 0);
        }
    }

    fn percent(state: &SetupState) -> usize {
        let total = state.progress.total.load(Ordering::Relaxed);
        let done = state.progress.done.load(Ordering::Relaxed);
        let Some(percent) = done.saturating_mul(100).checked_div(total) else {
            return 0;
        };
        usize::try_from(percent).unwrap_or(0).min(100)
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnableWindow(hwnd: HWND, enable: i32) -> i32;
    }

    unsafe fn enable_window(hwnd: HWND, enabled: i32) {
        EnableWindow(hwnd, enabled);
    }

    #[allow(unsafe_code)]
    unsafe fn set_busy(hwnd: HWND, busy: bool) {
        let enabled = i32::from(!busy);
        for id in [ID_DEST, ID_BROWSE, 1] {
            let child = GetDlgItem(hwnd, id);
            if !child.is_null() {
                enable_window(child, enabled);
            }
        }
    }

    #[allow(unsafe_code)]
    unsafe fn browse_folder(owner: HWND, initial: &Path) -> Option<PathBuf> {
        use windows_sys::Win32::System::Com::CoTaskMemFree;
        use windows_sys::Win32::UI::Shell::{
            BROWSEINFOW, SHBrowseForFolderW, SHGetPathFromIDListW,
        };

        let title = wide_str("Select the destination folder");
        let initial_wide = wide_os(initial.as_os_str());
        let selection = if initial.as_os_str().is_empty() {
            0
        } else {
            initial_wide.as_ptr() as isize
        };
        let mut display = [0u16; 260];
        let info = BROWSEINFOW {
            hwndOwner: owner,
            pidlRoot: std::ptr::null_mut(),
            pszDisplayName: display.as_mut_ptr(),
            lpszTitle: title.as_ptr(),
            ulFlags: 0x0001 | 0x0040, // BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE
            lpfn: Some(browse_initialized),
            lParam: selection,
            iImage: 0,
        };
        let pidl = SHBrowseForFolderW(&info);
        if pidl.is_null() {
            return None;
        }
        let mut raw = [0u16; 260];
        let ok = SHGetPathFromIDListW(pidl, raw.as_mut_ptr());
        CoTaskMemFree(pidl.cast());
        if ok == 0 {
            return None;
        }
        let len = raw.iter().position(|unit| *unit == 0).unwrap_or(raw.len());
        let path = PathBuf::from(OsString::from_wide(&raw[..len]));
        (!path.as_os_str().is_empty()).then_some(path)
    }

    #[allow(unsafe_code)]
    unsafe extern "system" fn browse_initialized(
        hwnd: HWND,
        msg: u32,
        _lparam: LPARAM,
        data: LPARAM,
    ) -> i32 {
        const BFFM_INITIALIZED: u32 = 1;
        const BFFM_SETSELECTIONW: u32 = 0x0400 + 103;
        if msg == BFFM_INITIALIZED && data != 0 {
            unsafe {
                SendMessageW(hwnd, BFFM_SETSELECTIONW, 1, data);
            }
        }
        0
    }

    #[allow(unsafe_code)]
    unsafe fn read_dest(hwnd: HWND) -> Result<PathBuf, Error> {
        let combo = GetDlgItem(hwnd, ID_DEST);
        if combo.is_null() {
            return Err(Error::Usage("destination field is missing".to_string()));
        }
        window_path(combo)
    }

    #[allow(unsafe_code)]
    unsafe fn window_path(hwnd: HWND) -> Result<PathBuf, Error> {
        let wide_len = GetWindowTextLengthW(hwnd);
        if wide_len <= 0 {
            return Ok(PathBuf::new());
        }
        let wide_len = usize::try_from(wide_len).unwrap_or(0);
        let mut buf = vec![0u16; wide_len.saturating_add(1)];
        let capacity =
            i32::try_from(buf.len()).map_err(|_| Error::Usage("path is too long".to_string()))?;
        let copied = GetWindowTextW(hwnd, buf.as_mut_ptr(), capacity);
        let copied = usize::try_from(copied).unwrap_or(0).min(buf.len());
        Ok(PathBuf::from(OsString::from_wide(&buf[..copied])))
    }

    /// `DRAWITEMSTRUCT`. Field order matches the Win32 layout, including padding.
    #[repr(C)]
    struct DrawItem {
        ctl_type: u32,
        ctl_id: u32,
        item_id: u32,
        item_action: u32,
        item_state: u32,
        hwnd_item: HWND,
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        rc_item: RECT,
        item_data: usize,
    }

    /// Bottom-up BGRA pixels for the setup banner.
    struct Banner {
        width: i32,
        height: i32,
        pixels: Vec<u8>,
    }

    fn decode_banner() -> Option<Banner> {
        let rgba = image::load_from_memory(include_bytes!("../assets/banner.png"))
            .ok()?
            .into_rgba8();
        let width = i32::try_from(rgba.width()).ok()?;
        let height = i32::try_from(rgba.height()).ok()?;
        if width <= 0 || height <= 0 {
            return None;
        }
        let stride = usize::try_from(width).ok()?.checked_mul(4)?;
        let rows = usize::try_from(height).ok()?;
        let mut pixels = rgba.into_raw();
        if pixels.len() != stride.checked_mul(rows)? {
            return None;
        }
        for px in pixels.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
        }
        for y in 0..rows / 2 {
            let top = y.saturating_mul(stride);
            let bottom = (rows - 1 - y).saturating_mul(stride);
            let (head, tail) = pixels.split_at_mut(bottom);
            head[top..top + stride].swap_with_slice(&mut tail[..stride]);
        }
        Some(Banner {
            width,
            height,
            pixels,
        })
    }

    #[allow(unsafe_code)]
    unsafe fn paint_banner(lparam: LPARAM) {
        if lparam == 0 {
            return;
        }
        let draw = unsafe { &*(lparam as *const DrawItem) };
        if draw.ctl_id != 100 {
            return;
        }
        fill_band(
            draw.hdc,
            draw.rc_item,
            draw.rc_item.top,
            draw.rc_item.bottom,
            rgb(14, 15, 19),
        );
        let parent = unsafe { GetParent(draw.hwnd_item) };
        let Some(state) = (unsafe { state_mut(parent) }) else {
            return;
        };
        let Some(banner) = state.banner.as_ref() else {
            return;
        };
        blit_banner(draw, banner);
    }

    #[allow(unsafe_code)]
    unsafe fn blit_banner(draw: &DrawItem, banner: &Banner) {
        use windows_sys::Win32::Graphics::Gdi::{
            BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HALFTONE, RGBQUAD, SRCCOPY,
            SetBrushOrgEx, SetStretchBltMode, StretchDIBits,
        };

        let dest = fitted(draw.rc_item, banner.width, banner.height);
        let dest_w = dest.right.saturating_sub(dest.left);
        let dest_h = dest.bottom.saturating_sub(dest.top);
        if dest_w <= 0 || dest_h <= 0 {
            return;
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or(40),
                biWidth: banner.width,
                biHeight: banner.height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };
        unsafe {
            SetStretchBltMode(draw.hdc, HALFTONE);
            SetBrushOrgEx(draw.hdc, 0, 0, std::ptr::null_mut());
            StretchDIBits(
                draw.hdc,
                dest.left,
                dest.top,
                dest_w,
                dest_h,
                0,
                0,
                banner.width,
                banner.height,
                banner.pixels.as_ptr().cast(),
                &info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
    }

    fn fitted(rc: RECT, src_w: i32, src_h: i32) -> RECT {
        let bounds_w = i64::from(rc.right.saturating_sub(rc.left));
        let bounds_h = i64::from(rc.bottom.saturating_sub(rc.top));
        let src_w = i64::from(src_w);
        let src_h = i64::from(src_h);
        if bounds_w <= 0 || bounds_h <= 0 || src_w <= 0 || src_h <= 0 {
            return rc;
        }
        let (draw_w, draw_h) = if bounds_w * src_h <= bounds_h * src_w {
            (bounds_w, bounds_w * src_h / src_w)
        } else {
            (bounds_h * src_w / src_h, bounds_h)
        };
        let draw_w = i32::try_from(draw_w).unwrap_or(0);
        let draw_h = i32::try_from(draw_h).unwrap_or(0);
        let left = rc.left + (rc.right.saturating_sub(rc.left).saturating_sub(draw_w)) / 2;
        let top = rc.top + (rc.bottom.saturating_sub(rc.top).saturating_sub(draw_h)) / 2;
        RECT {
            left,
            top,
            right: left.saturating_add(draw_w),
            bottom: top.saturating_add(draw_h),
        }
    }

    #[allow(unsafe_code)]
    unsafe fn fill_band(
        dc: windows_sys::Win32::Graphics::Gdi::HDC,
        mut rc: RECT,
        top: i32,
        bottom: i32,
        color: u32,
    ) {
        use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, DeleteObject, FillRect};

        rc.top = top;
        rc.bottom = bottom;
        let brush = CreateSolidBrush(color);
        if !brush.is_null() {
            FillRect(dc, &rc, brush);
            DeleteObject(brush);
        }
    }

    fn rgb(red: u32, green: u32, blue: u32) -> u32 {
        red | (green << 8) | (blue << 16)
    }

    #[allow(unsafe_code)]
    unsafe fn install_icons(hwnd: HWND, state: &mut SetupState) {
        state.small_icon = load_icon(16);
        state.big_icon = load_icon(32);
        SendMessageW(hwnd, WM_SETICON, 0, state.small_icon as isize);
        SendMessageW(hwnd, WM_SETICON, 1, state.big_icon as isize);
    }

    #[allow(unsafe_code)]
    unsafe fn load_icon(px: i32) -> windows_sys::Win32::UI::WindowsAndMessaging::HICON {
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows_sys::Win32::UI::WindowsAndMessaging::LoadImageW;

        let instance = GetModuleHandleW(std::ptr::null());
        LoadImageW(instance, 1_usize as _, 1, px, px, 0)
            as windows_sys::Win32::UI::WindowsAndMessaging::HICON
    }

    #[allow(unsafe_code)]
    unsafe fn destroy_icon(icon: windows_sys::Win32::UI::WindowsAndMessaging::HICON) {
        if !icon.is_null() {
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon(icon);
        }
    }

    #[allow(unsafe_code)]
    unsafe fn add_choice(combo: HWND, path: &Path) {
        let wide = wide_os(path.as_os_str());
        SendMessageW(combo, CB_ADDSTRING, 0, wide.as_ptr() as isize);
    }

    #[allow(unsafe_code)]
    unsafe fn set_text(hwnd: HWND, id: i32, text: &str) {
        let wide = wide_str(text);
        windows_sys::Win32::UI::WindowsAndMessaging::SetDlgItemTextW(hwnd, id, wide.as_ptr());
    }

    #[allow(unsafe_code)]
    unsafe fn set_window_text(hwnd: HWND, text: &std::ffi::OsStr) {
        if hwnd.is_null() {
            return;
        }
        let wide = wide_os(text);
        SetWindowTextW(hwnd, wide.as_ptr());
    }

    fn wide_str(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn wide_os(text: &std::ffi::OsStr) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        text.encode_wide().chain(std::iter::once(0)).collect()
    }

    #[test]
    fn banner_pixels_are_bgra() {
        let banner = decode_banner().expect("banner");
        assert_eq!((banner.width, banner.height), (1280, 720));
        assert_eq!(banner.pixels.len(), 1280 * 720 * 4);
    }
}

#[cfg(windows)]
pub(crate) use win::run;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_dialog_has_browse_and_a_progress_bar() {
        let bytes = setup_template();
        let count = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(count, 9);
        assert!(utf16_contains(&bytes, "ShellZip self-extracting archive"));
        assert!(utf16_contains(&bytes, "Browse..."));
        assert!(utf16_contains(&bytes, "Extract"));
        assert!(utf16_contains(&bytes, "Destination folder"));
        assert!(utf16_contains(&bytes, "msctls_progress32"));
    }

    fn utf16_contains(bytes: &[u8], text: &str) -> bool {
        let needle: Vec<u8> = text
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect();
        bytes
            .windows(needle.len())
            .any(|window| window == needle.as_slice())
    }

    #[cfg(windows)]
    #[test]
    #[allow(unsafe_code)] // creates a hidden dialog to validate the template
    fn windows_accepts_the_setup_dialog_template() {
        use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateDialogIndirectParamW, DLGTEMPLATE, DestroyWindow,
        };

        super::win::init_common_controls();
        let words = dword_words(&setup_template());
        let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
        assert!(!instance.is_null());
        let hwnd = unsafe {
            CreateDialogIndirectParamW(
                instance,
                words.as_ptr().cast::<DLGTEMPLATE>(),
                std::ptr::null_mut(),
                Some(super::win::setup_proc),
                0,
            )
        };
        if hwnd.is_null() {
            panic!(
                "setup dialog template was rejected: {}",
                std::io::Error::last_os_error()
            );
        }
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
    }
}
