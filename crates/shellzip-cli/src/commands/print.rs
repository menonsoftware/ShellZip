//! `szip p`.

use std::io::{self, Write};

use shellzip_core::{Error, write_member};

use super::Shared;
use crate::cli::PrintArgs;

/// Write one member to stdout. Progress stays off this stream.
pub fn print_entry(args: &PrintArgs, shared: &Shared) -> Result<i32, Error> {
    let stdout = io::stdout();
    let mut locked = stdout.lock();
    write_member(
        &args.archive,
        args.file.as_deref(),
        &mut locked,
        shared.password.as_ref(),
    )?;
    locked
        .flush()
        .map_err(|err| Error::io("write stdout", err))?;
    Ok(0)
}
