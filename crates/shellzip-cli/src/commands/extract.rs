//! `szip x` and `szip e`.

use std::path::Path;

use shellzip_core::{Error, ExtractOpts, Overwrite, extract as unpack};

use super::Shared;
use crate::cli::{ExtractArgs, overwrite_mode};
use crate::ui::Ask;

/// Extract `args.archive`. `flat` drops archive directories.
pub fn extract(args: &ExtractArgs, flat: bool, shared: &Shared) -> Result<i32, Error> {
    let mut opts = ExtractOpts::new(&args.output);
    opts.overwrite = overwrite_mode(args)?;
    opts.flat = flat;
    opts.keep_broken = args.keep_broken;
    opts.skip_encrypted = args.skip_encrypted;
    opts.allow_unsafe_links = args.allow_unsafe_links;
    opts.mark_of_the_web = args.mark_of_the_web;
    opts.password = shared.password.clone();
    opts.includes = args.include.clone();
    if !args.files.is_empty() {
        opts.exact_paths = Some(args.files.clone());
    }
    let ask = Ask::new();
    let report = if opts.overwrite == Overwrite::Ask {
        let decide = |path: &Path| ask.decide(path);
        unpack(&args.archive, &opts, Some(&decide), shared.sink.as_ref())?
    } else {
        unpack(&args.archive, &opts, None, shared.sink.as_ref())?
    };
    for warning in &report.warnings {
        anstream::eprintln!("szip: {warning}");
    }
    if shared.quiet < 2 {
        println!("extracted {}, skipped {}", report.extracted, report.skipped);
    }
    Ok(if report.warnings.is_empty() { 0 } else { 1 })
}
