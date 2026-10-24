//! `szip t`.

use shellzip_core::{Error, test as test_archive};

use super::Shared;
use crate::cli::ArchiveArgs;

/// Decode every member and report checksum or encryption failures.
pub fn test(args: &ArchiveArgs, shared: &Shared) -> Result<i32, Error> {
    let report = test_archive(
        &args.archive,
        shared.password.as_ref(),
        shared.sink.as_ref(),
    )?;
    for name in &report.checksums {
        anstream::eprintln!("szip: checksum mismatch for `{name}`");
    }
    for name in &report.encrypted {
        anstream::eprintln!("szip: encrypted entry `{name}`");
    }
    let code = report.exit_code();
    if code == 0 && shared.quiet < 2 {
        println!("OK, {} entries", report.entries);
    }
    Ok(code)
}
