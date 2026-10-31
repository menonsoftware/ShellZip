//! `shellzip-sfx` — extract the ZIP appended to this executable.
//!
//! Switches: `-s` silent, `-y` replace existing files, `-d <dir>` destination.
//! Without a console on Windows, a setup window chooses the folder and a
//! password dialog unlocks an encrypted archive.

#![cfg_attr(all(not(test), target_os = "windows"), windows_subsystem = "windows")]

use std::cell::Cell;
use std::path::{Path, PathBuf};

use shellzip_core::{
    ConflictChoice, Error, ExtractOpts, NopProgress, Overwrite, Password, extract, list, locate,
};

mod prompt;
mod setup;

use prompt::Choice;

fn main() {
    prompt::attach_console();
    let code = match run() {
        Ok(code) => code,
        Err(err) => {
            prompt::report(&err, prompt::gui_session());
            err.exit_code()
        }
    };
    std::process::exit(code);
}

fn run() -> Result<i32, Error> {
    let args = parse(std::env::args())?;
    if args.help {
        print_help();
        return Ok(0);
    }
    let gui = prompt::gui_session();
    let exe = std::env::current_exe().map_err(|err| Error::io("find this executable", err))?;
    if locate(&exe)?.is_none() {
        return Err(Error::Usage(
            "this executable has no ShellZip trailer; create one with `szip s` or `szip a --sfx`"
                .to_string(),
        ));
    }
    let dest = args.dest.unwrap_or(
        std::env::current_dir().map_err(|err| Error::io("find the working directory", err))?,
    );
    #[cfg(windows)]
    if gui && !args.silent {
        return setup::run(&exe, &dest, env_password()?);
    }
    if !args.silent && !args.yes && prompt::can_ask(gui) {
        let start = prompt::ask(&format!("Extract to {}?", dest.display()), gui, false)?;
        if !matches!(start, Choice::Yes) {
            return Err(Error::Usage("extraction cancelled".to_string()));
        }
    }
    let supplied = env_password()?;
    let from_env = supplied.is_some();
    let mut password = if from_env {
        supplied
    } else {
        optional_password(&exe, args.silent, gui)?
    };
    let retry = should_retry_password(from_env, args.silent, prompt::can_ask(gui));
    loop {
        let mut opts = ExtractOpts::new(&dest);
        opts.password = password;
        opts.overwrite = if args.yes {
            Overwrite::Always
        } else if args.silent || !prompt::can_ask(gui) {
            Overwrite::Never
        } else {
            Overwrite::Ask
        };
        let all = Cell::new(false);
        let decide = |path: &Path| replace_choice(path, gui, &all);
        let result = if opts.overwrite == Overwrite::Ask {
            extract(&exe, &opts, Some(&decide), &NopProgress)
        } else {
            extract(&exe, &opts, None, &NopProgress)
        };
        // Wipe the password before a success notice or another prompt.
        opts.password = None;
        match result {
            Ok(report) => {
                if !args.silent {
                    prompt::notice(
                        &format!("Extracted {} to {}", report.extracted, dest.display()),
                        gui,
                    );
                }
                return Ok(0);
            }
            Err(Error::BadPassword) if retry => {
                password = Some(required_password(
                    gui,
                    "That password was incorrect. Enter it again.",
                )?);
            }
            Err(err) => return Err(err),
        }
    }
}

fn replace_choice(path: &Path, gui: bool, all: &Cell<bool>) -> Result<ConflictChoice, Error> {
    if all.get() {
        return Ok(ConflictChoice::Overwrite);
    }
    match prompt::ask(&format!("Replace {}?", path.display()), gui, true)? {
        Choice::Yes => Ok(ConflictChoice::Overwrite),
        Choice::All => {
            all.set(true);
            Ok(ConflictChoice::Overwrite)
        }
        Choice::No => Ok(ConflictChoice::Skip),
        Choice::Cancel => Err(Error::Interrupted),
    }
}

/// Ask again after a rejected password that the person typed.
fn should_retry_password(from_env: bool, silent: bool, can_ask: bool) -> bool {
    !from_env && !silent && can_ask
}

fn optional_password(exe: &Path, silent: bool, gui: bool) -> Result<Option<Password>, Error> {
    let encrypted = match list(exe) {
        Ok((_, entries)) => entries.iter().any(|entry| entry.encrypted),
        // A 7z archive with encrypted headers cannot be listed until a password is known.
        Err(Error::Encrypted(_)) => true,
        Err(err) => return Err(err),
    };
    if !encrypted {
        return Ok(None);
    }
    if silent || !prompt::can_ask(gui) {
        return Err(Error::Encrypted(exe.display().to_string()));
    }
    required_password(gui, "Enter the password to extract this archive.").map(Some)
}

fn required_password(gui: bool, text: &str) -> Result<Password, Error> {
    let mut text = text.to_string();
    loop {
        match prompt::password(&text, gui)? {
            None => return Err(Error::Usage("extraction cancelled".to_string())),
            Some(password) if password.expose().is_empty() => {
                drop(password);
                if !gui {
                    return Err(Error::Usage("password was empty".to_string()));
                }
                text = "Enter a password.".to_string();
            }
            Some(password) => return Ok(password),
        }
    }
}

fn env_password() -> Result<Option<Password>, Error> {
    match std::env::var("SZIP_PASSWORD") {
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(Password::new(value)?)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(Error::Usage(
            "SZIP_PASSWORD is not valid Unicode".to_string(),
        )),
    }
}

#[derive(Debug)]
struct Args {
    silent: bool,
    yes: bool,
    help: bool,
    dest: Option<PathBuf>,
}

fn parse(args: impl IntoIterator<Item = String>) -> Result<Args, Error> {
    let mut parsed = Args {
        silent: false,
        yes: false,
        help: false,
        dest: None,
    };
    let mut args = args.into_iter();
    let _ = args.next();
    while let Some(arg) = args.next() {
        if arg == "-s" || arg == "--silent" {
            parsed.silent = true;
        } else if arg == "-y" || arg == "--yes" {
            parsed.yes = true;
        } else if arg == "-h" || arg == "--help" {
            parsed.help = true;
        } else if arg == "-d" || arg == "--dest" {
            let Some(dir) = args.next() else {
                return Err(Error::Usage("-d needs a directory".to_string()));
            };
            parsed.dest = Some(PathBuf::from(dir));
        } else if let Some(dir) = arg.strip_prefix("-d")
            && !dir.is_empty()
            && !dir.starts_with('-')
        {
            parsed.dest = Some(PathBuf::from(dir));
        } else if let Some(dir) = arg.strip_prefix("--dest=") {
            parsed.dest = Some(PathBuf::from(dir));
        } else {
            return Err(Error::Usage(format!("unknown switch `{arg}`")));
        }
    }
    Ok(parsed)
}

fn print_help() {
    println!(
        "\
shellzip-sfx — extract the ZIP stored in this executable

  -s, --silent     no prompts
  -y, --yes        replace existing files
  -d, --dest DIR   extract into DIR (default: the working directory)
  -h, --help       show this help

Opening the file from Explorer shows a window to choose the folder.
An encrypted archive asks for a password when you press Extract.
-s reads SZIP_PASSWORD and does not ask."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_attached_dest_and_switches() {
        let args = parse(["shellzip-sfx", "-s", "-y", "-dout"].map(str::to_string)).unwrap();
        assert!(args.silent);
        assert!(args.yes);
        assert_eq!(args.dest.unwrap(), PathBuf::from("out"));
    }

    #[test]
    fn rejects_an_unknown_switch() {
        let err = parse(["shellzip-sfx", "--nope"].map(str::to_string)).unwrap_err();
        assert_eq!(err.exit_code(), 7);
    }

    #[test]
    fn retries_a_typed_password_and_not_a_silent_one() {
        assert!(should_retry_password(false, false, true));
        assert!(!should_retry_password(true, false, true));
        assert!(!should_retry_password(false, true, true));
        assert!(!should_retry_password(false, false, false));
    }

    #[test]
    fn stub_icon_file_is_a_windows_icon() {
        let bytes = include_bytes!("../assets/shellzip.ico");
        assert_eq!(&bytes[..4], &[0, 0, 1, 0]);
        let count = u16::from_le_bytes([bytes[4], bytes[5]]);
        assert!(count >= 5, "icon sizes: {count}");
    }

    #[test]
    fn stub_banner_file_is_a_png() {
        let bytes = include_bytes!("../assets/banner.png");
        assert_eq!(
            &bytes[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }
}
