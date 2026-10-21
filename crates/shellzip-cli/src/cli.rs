//! Command line for `szip a`, `x`, `e`, `l`, and `t`.

use std::path::PathBuf;

use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell, generate};
use shellzip_core::{CompressOpts, Error, Overwrite, StoreList, ZipMethod};

use crate::commands::{self, Shared};
use crate::ui;

/// Create and extract ZIP, tar, gzip, and zstd archives.
#[derive(Debug, Parser)]
#[command(name = "szip", version, about, propagate_version = true)]
pub struct Cli {
    /// Command.
    #[command(subcommand)]
    pub command: Command,
    /// More logging. Repeat for a wider trace.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,
    /// Hide progress. Repeat to hide the summary as well. `l` still prints.
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub quiet: u8,
}

/// `szip` commands implemented in this milestone.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add files to an archive. An existing ZIP keeps its other files unless `-y` is passed.
    #[command(name = "a")]
    Add(AddArgs),
    /// Extract an archive, recreating directories.
    #[command(name = "x")]
    Extract(ExtractArgs),
    /// Extract files into one directory, without archive paths.
    #[command(name = "e")]
    ExtractFlat(ExtractArgs),
    /// List archive members.
    #[command(name = "l")]
    List(ArchiveArgs),
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
    #[arg(required = true)]
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
    /// Replace an existing archive instead of updating a ZIP.
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,
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
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let code = if err.exit_code() == 0 { 0 } else { 7 };
            let _ = err.print();
            return code;
        }
    };
    init_tracing(cli.verbose);
    match dispatch(&cli) {
        Ok(code) => code,
        Err(err) => {
            anstream::eprintln!("szip: {err}");
            err.exit_code()
        }
    }
}

fn dispatch(cli: &Cli) -> Result<i32, Error> {
    let shared = Shared {
        quiet: cli.quiet,
        sink: ui::sink(cli.quiet),
    };
    match &cli.command {
        Command::Add(args) => commands::add(args, &shared),
        Command::Extract(args) => commands::extract(args, false, &shared),
        Command::ExtractFlat(args) => commands::extract(args, true, &shared),
        Command::List(args) => commands::list(args),
        Command::Test(args) => commands::test(args, &shared),
        Command::Complete { shell } => {
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "szip", &mut std::io::stdout());
            Ok(0)
        }
    }
}

pub(crate) fn compress_opts(args: &AddArgs) -> Result<CompressOpts, Error> {
    if args.store_ext.is_some() && args.no_store_list {
        return Err(Error::Usage(
            "pass either --store-ext or --no-store-list".to_string(),
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

fn init_tracing(verbose: u8) {
    let filter = match verbose {
        0 => "warn",
        1 => "shellzip_core=debug,warn",
        _ => "debug",
    };
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}
