//! `szip a`, `u`, and `f`.

use std::io::Read;

use shellzip_core::{
    Error, SfxPayload, UpdateMode, add as pack, add_sfx, add_stdin, build, format_bytes,
};

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
    if !args.sfx && args.sfx_format != crate::cli::SfxFormatArg::Zip {
        return Err(Error::Usage("--sfx-format requires --sfx".to_string()));
    }
    let existed = args.archive.is_file();
    let updating = existed && !args.yes && mode == UpdateMode::Add && !args.sfx;
    let mut opts = compress_opts(args, mode)?;
    opts.password = shared.password.clone();
    if args.sfx {
        let stub = super::sfx::find_stub(args.sfx_stub.as_deref())?;
        let payload = match args.sfx_format {
            crate::cli::SfxFormatArg::Zip => SfxPayload::Zip,
            crate::cli::SfxFormatArg::SevenZ => SfxPayload::SevenZ,
        };
        return pack_sfx(args, &stub, &opts, payload, shared);
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
            UpdateMode::Ask | UpdateMode::Skip => "updated",
            UpdateMode::Sync => "synchronized",
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
    payload: SfxPayload,
    shared: &Shared,
) -> Result<i32, Error> {
    let mut opts = opts.clone();
    if payload == SfxPayload::SevenZ && opts.level > 0 {
        opts.sevenz.method = shellzip_core::SevenZMethod::Lzma2;
    }
    let opts = &opts;
    let report = if let Some(name) = &args.stdin {
        let scratch = std::env::temp_dir().join(format!(
            "szip-sfx-stdin-{}-{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0),
            payload.extension()
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
            let packed_bytes = build(&args.archive, stub, &scratch, payload)?;
            Ok(shellzip_core::AddReport {
                path: args.archive.clone(),
                packed_bytes,
                ..report
            })
        });
        let _ = std::fs::remove_file(&scratch);
        result?
    } else {
        add_sfx(
            &args.archive,
            stub,
            &args.files,
            opts,
            payload,
            shared.sink.as_ref(),
        )?
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
