//! `szip a`, `u`, and `f`.

use std::io::Read;

use shellzip_core::{Error, UpdateMode, add as pack, add_stdin, format_bytes};

use super::Shared;
use crate::cli::{AddArgs, compress_opts};

/// Pack `args.files` or stdin into `args.archive`.
pub fn add(args: &AddArgs, mode: UpdateMode, shared: &Shared) -> Result<i32, Error> {
    let existed = args.archive.is_file();
    let updating = existed && !args.yes && mode == UpdateMode::Add;
    let mut opts = compress_opts(args, mode)?;
    opts.password = shared.password.clone();
    let report = if let Some(name) = &args.stdin {
        let stdin = std::io::stdin();
        let mut locked = stdin.lock();
        add_stdin(
            &args.archive,
            name,
            &mut locked as &mut dyn Read,
            &opts,
            shared.sink.as_ref(),
        )?
    } else {
        pack(&args.archive, &args.files, &opts, shared.sink.as_ref())?
    };
    if shared.quiet < 2 {
        let verb = match mode {
            UpdateMode::Add if updating => "updated",
            UpdateMode::Add => "created",
            UpdateMode::Update if existed => "updated",
            UpdateMode::Update => "created",
            UpdateMode::Freshen => "freshened",
        };
        println!(
            "{verb} {} ({} entries, {})",
            report.path.display(),
            report.entries,
            format_bytes(report.packed_bytes)
        );
    }
    Ok(0)
}
