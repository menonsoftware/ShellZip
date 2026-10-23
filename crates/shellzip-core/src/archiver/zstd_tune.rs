//! Thread count and window size for a zstd encoder.

use std::io::Write;

use crate::error::Error;
use crate::options::CompressOpts;

/// Enable zstd's worker pool and pick `windowLog` from [`CompressOpts`].
pub(crate) fn tune<W: Write>(
    encoder: &mut zstd::stream::write::Encoder<'_, W>,
    opts: &CompressOpts,
) -> Result<(), Error> {
    let workers = u32::try_from(opts.thread_count())
        .unwrap_or(u32::MAX)
        .min(64);
    encoder
        .multithread(workers)
        .map_err(|err| Error::io("zstd threads", err))?;
    let log = opts.zstd_window_log();
    encoder
        .window_log(log)
        .map_err(|err| Error::io("zstd window", err))?;
    if opts.zstd_long.is_some() || log >= 27 {
        encoder
            .long_distance_matching(true)
            .map_err(|err| Error::io("zstd long match", err))?;
    }
    Ok(())
}
