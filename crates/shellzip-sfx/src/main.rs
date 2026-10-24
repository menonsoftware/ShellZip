//! `shellzip-sfx` — extract the ZIP appended to this executable.
//!
//! Switches: `-s` silent, `-y` replace existing files, `-d <dir>` destination.
//! Without a console on Windows, the same choices are dialogs.

#![cfg_attr(all(not(test), target_os = "windows"), windows_subsystem = "windows")]

use std::cell::Cell;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use shellzip_core::{
    ConflictChoice, Error, ExtractOpts, NopProgress, Overwrite, Password, extract, list, locate,
};

mod prompt;

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
    if !args.silent && !args.yes && prompt::can_ask(gui) {
        let start = prompt::ask(&format!("Extract to {}?", dest.display()), gui, false)?;
        if !matches!(start, Choice::Yes) {
            return Err(Error::Usage("extraction cancelled".to_string()));
        }
    }
    let password = password_for(&exe, args.silent, gui)?;
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
    let report = if opts.overwrite == Overwrite::Ask {
        extract(&exe, &opts, Some(&decide), &NopProgress)?
    } else {
        extract(&exe, &opts, None, &NopProgress)?
    };
    if !args.silent {
        prompt::notice(
            &format!("Extracted {} to {}", report.extracted, dest.display()),
            gui,
        );
    }
    Ok(0)
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

fn password_for(exe: &Path, silent: bool, gui: bool) -> Result<Option<Password>, Error> {
    if let Some(password) = env_password()? {
        return Ok(Some(password));
    }
    let (_opened, entries) = list(exe)?;
    if !entries.iter().any(|entry| entry.encrypted) {
        return Ok(None);
    }
    if silent || gui || !io::stdin().is_terminal() {
        return Err(Error::Encrypted(exe.display().to_string()));
    }
    eprint!("Password: ");
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
    if line.is_empty() {
        return Err(Error::Usage("password was empty".to_string()));
    }
    Ok(Some(Password::new(line)?))
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

An encrypted ZIP reads SZIP_PASSWORD. A console session can type a password.
A Windows dialog cannot, so set SZIP_PASSWORD before opening the file."
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
}
