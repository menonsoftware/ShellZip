//! `szip a`, `u`, and `f`.

use std::io::Read;

use shellzip_core::{Error, UpdateMode, add as pack, add_sfx, add_stdin, build, format_bytes};

use super::Shared;
use crate::cli::{AddArgs, compress_opts};

/// Pack `args.files` or stdin into `args.archive`.
pub fn add(args: &AddArgs, mode: UpdateMode, shared: &Shared) -> Result<i32, Error> {
    if args.sfx && mode != UpdateMode::Add {
        return Err(Error::Usage(
            "--sfx is only valid with `szip a`".to_string(),
        ));
    }
    if args.sfx_stub.is_some() && !args.sfx {
        return Err(Error::Usage("--sfx-stub requires --sfx".to_string()));
    }
    let existed = args.archive.is_file();
    let updating = existed && !args.yes && mode == UpdateMode::Add && !args.sfx;
    let mut opts = compress_opts(args, mode)?;
    opts.password = shared.password.clone();
    if args.sfx {
        let stub = super::sfx::find_stub(args.sfx_stub.as_deref())?;
        return pack_sfx(args, &stub, &opts, shared);
    }
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

fn pack_sfx(
    args: &AddArgs,
    stub: &std::path::Path,
    opts: &shellzip_core::CompressOpts,
    shared: &Shared,
) -> Result<i32, Error> {
    let report = if let Some(name) = &args.stdin {
        let scratch = std::env::temp_dir().join(format!(
            "szip-sfx-stdin-{}-{}.zip",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        ));
        let stdin = std::io::stdin();
        let mut locked = stdin.lock();
        let written = add_stdin(
            &scratch,
            name,
            &mut locked as &mut dyn Read,
            opts,
            shared.sink.as_ref(),
        );
        let result = written.and_then(|report| {
            let packed_bytes = build(&args.archive, stub, &scratch)?;
            Ok(shellzip_core::AddReport {
                path: args.archive.clone(),
                packed_bytes,
                ..report
            })
        });
        let _ = std::fs::remove_file(&scratch);
        result?
    } else {
        add_sfx(&args.archive, stub, &args.files, opts, shared.sink.as_ref())?
    };
    if shared.quiet < 2 {
        println!(
            "created {} ({} entries, {})",
            report.path.display(),
            report.entries,
            format_bytes(report.packed_bytes)
        );
    }
    Ok(0)
}
