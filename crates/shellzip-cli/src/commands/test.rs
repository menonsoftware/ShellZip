//! `szip t`.

use shellzip_core::{Error, test_with};

use super::Shared;
use crate::cli::TestArgs;

/// Decode every member and report checksum or encryption failures.
pub fn test(args: &TestArgs, shared: &Shared) -> Result<i32, Error> {
    let limits = args.limits.read_limits();
    let report = test_with(
        &args.archive,
        shared.password.as_ref(),
        shared.sink.as_ref(),
        &limits,
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
