//! Command line for `szip`.

use std::path::PathBuf;

use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use shellzip_core::{CompressOpts, Error, Overwrite, StoreList, UpdateMode, ZipMethod};

use crate::commands::{self, Shared};
use crate::ui;

/// Create and extract ZIP, tar, gzip, zstd, xz, and bzip2 archives.
#[derive(Debug, Parser)]
#[command(name = "szip", version, about, propagate_version = true)]
pub struct Cli {
    /// Command.
    #[command(subcommand)]
    pub command: Command,
    /// More logging. Repeat for a wider trace.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
    /// Hide progress. Repeat to hide the summary as well. `l` and `lt` still print.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub quiet: u8,
    /// Append a tracing log to FILE (`-ilog`).
    #[arg(long = "ilog", value_name = "FILE", global = true)]
    pub ilog: Option<PathBuf>,
    /// Also write the tracing log to stderr. With `-ilog`, stderr is used only when this is set.
    #[arg(long = "ierr", global = true)]
    pub ierr: bool,
}

/// `szip` commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add files to an archive. An existing ZIP keeps its other files unless `-y` is passed.
    #[command(name = "a")]
    Add(AddArgs),
    /// Add files that are new or newer than the archived copy. ZIP only.
    #[command(name = "u")]
    Update(AddArgs),
    /// Replace archived files only when the source is newer. ZIP only.
    #[command(name = "f")]
    Freshen(AddArgs),
    /// Delete ZIP members that match a mask.
    #[command(name = "d")]
    Delete(DeleteArgs),
    /// Extract an archive, recreating directories.
    #[command(name = "x")]
    Extract(ExtractArgs),
    /// Extract files into one directory, without archive paths.
    #[command(name = "e")]
    ExtractFlat(ExtractArgs),
    /// List archive members.
    #[command(name = "l")]
    List(ArchiveArgs),
    /// Technical list: size, packed, ratio, time, attributes, CRC32, method, and host OS.
    #[command(name = "lt")]
    ListTechnical(ListArgs),
    /// Write one member to stdout.
    #[command(name = "p")]
    Print(PrintArgs),
    /// Test checksums and declared sizes.
    #[command(name = "t")]
    Test(ArchiveArgs),
    /// Write a shell completion script to stdout.
    Complete {
        /// Shell to generate.
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// `szip a <archive> <files…>`.
#[derive(Debug, Parser)]
pub struct AddArgs {
    /// Archive to create or update. `.zip`, `.tar`, `.tar.gz`, `.tgz`, `.tar.zst`, `.tzst`, `.gz`, `.zst`.
    pub archive: PathBuf,
    /// Files and directories to pack.
    #[arg(required_unless_present = "stdin")]
    pub files: Vec<PathBuf>,
    /// Compression level, 0 (store) through 5 (maximum).
    #[arg(short = 'm', long = "level", default_value_t = 3, value_parser = clap::value_parser!(u8).range(0..=5))]
    pub level: u8,
    /// ZIP method used above level 0. Ignored for tar and single-file streams.
    #[arg(long, value_enum, default_value = "deflate")]
    pub method: MethodArg,
    /// Drop the top directory name from stored paths.
    #[arg(long = "ep1")]
    pub ep1: bool,
    /// Do not descend into directories.
    #[arg(long = "no-recurse")]
    pub no_recurse: bool,
    /// Store the target of a symlink instead of the link.
    #[arg(long = "follow-symlinks")]
    pub follow_symlinks: bool,
    /// Exclude glob, matched against the archive path. Repeatable.
    #[arg(short = 'x', long = "exclude")]
    pub exclude: Vec<String>,
    /// Include glob. Repeatable. When set, only matches are packed.
    #[arg(short = 'n', long = "include")]
    pub include: Vec<String>,
    /// Semicolon-separated extensions to store uncompressed, replacing the default list.
    #[arg(long = "store-ext", value_name = "LIST")]
    pub store_ext: Option<String>,
    /// Compress every ZIP member, including the usual already-compressed types.
    #[arg(long = "no-store-list")]
    pub no_store_list: bool,
    /// Replace an existing archive instead of updating a ZIP. Only for `a`.
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
    /// Worker threads for ZIP entries and for zstd. Default is the CPU count, capped at 64.
    #[arg(long = "threads", value_name = "N", value_parser = clap::value_parser!(u16).range(1..=64))]
    pub threads: Option<u16>,
    /// Zstd window log, from 10 to 31. Default follows the total input size.
    #[arg(long = "long", value_name = "LOG2", value_parser = clap::value_parser!(u32).range(10..=31))]
    pub long_window: Option<u32>,
    /// Read one member from stdin instead of a file path.
    ///
    /// `--stdin` stores it as `stdin`. `--stdin=NAME` stores `NAME` (`-siNAME` is the same switch).
    /// `.gz` and `.zst` are streamed. `.tar`, `.tar.gz`, and `.tar.zst` copy stdin to a temporary
    /// file first, because the tar header stores the size. `.zip` is written to a seekable
    /// temporary file; ZIP local headers record sizes, so a pipe is not a ZIP destination.
    #[arg(
        long = "stdin",
        value_name = "NAME",
        num_args = 0..=1,
        default_missing_value = "stdin",
        require_equals = true
    )]
    pub stdin: Option<String>,
}

/// `szip d <archive> <masks…>`.
#[derive(Debug, Parser)]
pub struct DeleteArgs {
    /// ZIP archive to rewrite.
    pub archive: PathBuf,
    /// Names or globs to remove (`*.log`, `dir/file.txt`).
    #[arg(required = true)]
    pub masks: Vec<String>,
}

/// `szip lt`.
#[derive(Debug, Parser)]
pub struct ListArgs {
    /// Archive to read.
    pub archive: PathBuf,
    /// BLAKE2sp-256 of each file payload.
    #[arg(long = "htb")]
    pub htb: bool,
}

/// `szip p <archive> [member]`.
#[derive(Debug, Parser)]
pub struct PrintArgs {
    /// Archive to read.
    pub archive: PathBuf,
    /// Member to write. Omit when the archive has a single file.
    pub file: Option<String>,
}

/// `szip x` and `szip e`.
#[derive(Debug, Parser)]
pub struct ExtractArgs {
    /// Archive to read.
    pub archive: PathBuf,
    /// Member names to extract. Omit to extract everything selected by `-n`.
    pub files: Vec<String>,
    /// Directory that receives the files.
    #[arg(short = 'o', long = "output", default_value = ".")]
    pub output: PathBuf,
    /// What to do when a destination file exists.
    #[arg(long, value_enum, default_value = "ask")]
    pub overwrite: OverwriteArg,
    /// Replace existing files.
    #[arg(long = "o+")]
    pub replace: bool,
    /// Leave existing files in place.
    #[arg(long = "o-")]
    pub skip_existing: bool,
    /// Write `name (n).ext` beside an existing file.
    #[arg(long = "or")]
    pub rename_existing: bool,
    /// Replace existing files without asking. Same as `--o+`.
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
    /// Keep a partial file when decoding fails.
    #[arg(long = "kb", visible_alias = "keep-broken")]
    pub keep_broken: bool,
    /// Include glob. Repeatable.
    #[arg(short = 'n', long = "include")]
    pub include: Vec<String>,
}

/// Archive path shared by `l` and `t`.
#[derive(Debug, Parser)]
pub struct ArchiveArgs {
    /// Archive to read.
    pub archive: PathBuf,
}

/// `--method` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MethodArg {
    /// Deflate.
    Deflate,
    /// Zstd (ZIP method 93, or a zstd frame for `.tar.zst` and `.zst`).
    Zstd,
}

impl From<MethodArg> for ZipMethod {
    fn from(method: MethodArg) -> Self {
        match method {
            MethodArg::Deflate => Self::Deflate,
            MethodArg::Zstd => Self::Zstd,
        }
    }
}

/// `--overwrite` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OverwriteArg {
    /// Prompt when stdin is a terminal. Otherwise skip.
    Ask,
    /// Replace the file.
    Always,
    /// Leave the file in place.
    Skip,
    /// Pick a new name.
    Rename,
}

impl From<OverwriteArg> for Overwrite {
    fn from(mode: OverwriteArg) -> Self {
        match mode {
            OverwriteArg::Ask => Self::Ask,
            OverwriteArg::Always => Self::Always,
            OverwriteArg::Skip => Self::Never,
            OverwriteArg::Rename => Self::Rename,
        }
    }
}

/// Parse arguments, run one command, and return the process exit code.
pub fn run() -> i32 {
    let cli = match Cli::try_parse_from(normalize_args(std::env::args())) {
        Ok(cli) => cli,
        Err(err) => {
            let code = if err.exit_code() == 0 { 0 } else { 7 };
            let _ = err.print();
            return code;
        }
    };
    let guard = match init_tracing(&cli) {
        Ok(guard) => guard,
        Err(err) => {
            anstream::eprintln!("szip: {err}");
            return err.exit_code();
        }
    };
    let code = match dispatch(&cli) {
        Ok(code) => code,
        Err(err) => {
            anstream::eprintln!("szip: {err}");
            err.exit_code()
        }
    };
    drop(guard);
    code
}

fn dispatch(cli: &Cli) -> Result<i32, Error> {
    let shared = Shared {
        quiet: cli.quiet,
        sink: ui::sink(cli.quiet),
    };
    match &cli.command {
        Command::Add(args) => commands::add(args, UpdateMode::Add, &shared),
        Command::Update(args) => commands::add(args, UpdateMode::Update, &shared),
        Command::Freshen(args) => commands::add(args, UpdateMode::Freshen, &shared),
        Command::Delete(args) => commands::delete(args, &shared),
        Command::Extract(args) => commands::extract(args, false, &shared),
        Command::ExtractFlat(args) => commands::extract(args, true, &shared),
        Command::List(args) => commands::list(args),
        Command::ListTechnical(args) => commands::list_technical(args),
        Command::Print(args) => commands::print_entry(args),
        Command::Test(args) => commands::test(args, &shared),
        Command::Complete { shell } => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "szip", &mut std::io::stdout());
            Ok(0)
        }
    }
}

pub(crate) fn compress_opts(args: &AddArgs, update: UpdateMode) -> Result<CompressOpts, Error> {
    if args.store_ext.is_some() && args.no_store_list {
        return Err(Error::Usage(
            "pass either --store-ext or --no-store-list".to_string(),
        ));
    }
    if update != UpdateMode::Add && args.yes {
        return Err(Error::Usage(
            "-y replaces the archive and is only valid with `szip a`".to_string(),
        ));
    }
    if args.stdin.is_some() && !args.files.is_empty() {
        return Err(Error::Usage(
            "do not pass files together with --stdin".to_string(),
        ));
    }
    let store_exts = if args.no_store_list {
        StoreList {
            extensions: Vec::new(),
        }
    } else if let Some(list) = &args.store_ext {
        StoreList::parse(list)
    } else {
        StoreList::default()
    };
    Ok(CompressOpts {
        level: args.level,
        recurse: !args.no_recurse,
        ep1: args.ep1,
        follow_symlinks: args.follow_symlinks,
        zip_method: args.method.into(),
        store_exts,
        includes: args.include.clone(),
        excludes: args.exclude.clone(),
        force: args.yes,
        update,
        threads: args.threads.unwrap_or(0),
        zstd_long: args.long_window,
        input_bytes: None,
    })
}

pub(crate) fn overwrite_mode(args: &ExtractArgs) -> Result<Overwrite, Error> {
    let flag = match (args.replace, args.skip_existing, args.rename_existing) {
        (false, false, false) => None,
        (true, false, false) => Some(Overwrite::Always),
        (false, true, false) => Some(Overwrite::Never),
        (false, false, true) => Some(Overwrite::Rename),
        _ => {
            return Err(Error::Usage(
                "choose only one of --o+, --o-, and --or".to_string(),
            ));
        }
    };
    if args.yes {
        if flag.is_some_and(|mode| mode != Overwrite::Always) {
            return Err(Error::Usage(
                "-y replaces files and conflicts with --o- or --or".to_string(),
            ));
        }
        return Ok(Overwrite::Always);
    }
    if let Some(mode) = flag {
        if args.overwrite != OverwriteArg::Ask {
            return Err(Error::Usage(
                "pass either --overwrite or one of --o+, --o-, --or".to_string(),
            ));
        }
        return Ok(mode);
    }
    Ok(args.overwrite.into())
}

fn init_tracing(cli: &Cli) -> Result<Option<tracing_appender::non_blocking::WorkerGuard>, Error> {
    let filter = match cli.verbose {
        0 => "warn",
        1 => "shellzip_core=debug,warn",
        _ => "debug",
    };
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_ansi(false);
    let Some(path) = &cli.ilog else {
        let _ = builder.with_writer(std::io::stderr).try_init();
        return Ok(None);
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .map_err(|err| Error::io(format!("create {}", parent.display()), err))?;
    }
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| Error::io(format!("open {}", path.display()), err))?;
    let (writer, guard) = tracing_appender::non_blocking(file);
    if cli.ierr {
        use tracing_subscriber::fmt::writer::MakeWriterExt;
        let _ = builder.with_writer(writer.and(std::io::stderr)).try_init();
    } else {
        let _ = builder.with_writer(writer).try_init();
    }
    Ok(Some(guard))
}

/// Accept WinRAR-style switches (`-ilog=`, `-ierr`, `-htb`, `-siNAME`, `-mtN`, `-long=`).
fn normalize_args(args: impl Iterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for arg in args {
        if let Some(rest) = arg.strip_prefix("-ilog=") {
            out.push(format!("--ilog={rest}"));
        } else if arg == "-ilog" {
            out.push("--ilog".to_string());
        } else if arg == "-ierr" {
            out.push("--ierr".to_string());
        } else if arg == "-htb" {
            out.push("--htb".to_string());
        } else if arg == "-si" {
            out.push("--stdin".to_string());
        } else if let Some(name) = arg.strip_prefix("-si")
            && !name.is_empty()
            && !name.starts_with('-')
        {
            out.push(format!("--stdin={name}"));
        } else if let Some(rest) = arg.strip_prefix("-mt")
            && (rest.is_empty() || rest.chars().all(|ch| ch.is_ascii_digit()))
        {
            if rest.is_empty() {
                out.push("--threads".to_string());
            } else {
                out.push(format!("--threads={rest}"));
            }
        } else if arg == "-long" {
            out.push("--long".to_string());
        } else if let Some(rest) = arg.strip_prefix("-long=") {
            out.push(format!("--long={rest}"));
        } else {
            out.push(arg);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_winrar_switches() {
        let args = [
            "szip",
            "a",
            "-sihello.txt",
            "-mt4",
            "-ilog=run.log",
            "-ierr",
            "-htb",
            "-long=27",
        ]
        .into_iter()
        .map(str::to_string);
        assert_eq!(
            normalize_args(args),
            [
                "szip",
                "a",
                "--stdin=hello.txt",
                "--threads=4",
                "--ilog=run.log",
                "--ierr",
                "--htb",
                "--long=27",
            ]
        );
    }
}
