//! `szip d`.

use shellzip_core::{Error, delete as delete_members};

use super::Shared;
use crate::cli::DeleteArgs;

/// Remove members that match `args.masks`.
pub fn delete(args: &DeleteArgs, shared: &Shared) -> Result<i32, Error> {
    let report = delete_members(
        &args.archive,
        &args.masks,
        shared.password.as_ref(),
        shared.sink.as_ref(),
    )?;
    if shared.quiet < 2 {
        println!(
            "deleted {} from {} ({} kept)",
            report.removed,
            report.path.display(),
            report.kept
        );
    }
    Ok(0)
}
