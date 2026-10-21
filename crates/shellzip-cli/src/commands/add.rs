//! `szip a`.

use shellzip_core::{Error, add as pack, format_bytes};

use super::Shared;
use crate::cli::{AddArgs, compress_opts};

/// Pack `args.files` into `args.archive`.
pub fn add(args: &AddArgs, shared: &Shared) -> Result<i32, Error> {
    let updating = args.archive.is_file() && !args.yes;
    let opts = compress_opts(args)?;
    let report = pack(&args.archive, &args.files, &opts, shared.sink.as_ref())?;
    if shared.quiet < 2 {
        let verb = if updating { "updated" } else { "created" };
        println!(
            "{verb} {} ({} entries, {})",
            report.path.display(),
            report.entries,
            format_bytes(report.packed_bytes)
        );
    }
    Ok(0)
}
