//! Progress reports shared by the CLI bars and the desktop window.

/// Receives byte-level progress. Implementations must be cheap and non-blocking.
pub trait ProgressSink: Send + Sync {
    /// Start an operation whose total uncompressed size is known, or `0` if unknown.
    fn begin(&self, label: &str, total_bytes: u64);
    /// Start one archive entry.
    fn begin_entry(&self, name: &str, bytes: u64);
    /// Record `bytes` of payload copied since the last call.
    fn advance(&self, bytes: u64);
    /// The current entry finished successfully.
    fn end_entry(&self);
    /// A human-readable note that is not an error.
    fn message(&self, text: &str);
    /// `true` when the caller asked the job to stop.
    fn is_cancelled(&self) -> bool {
        false
    }
    /// Block this worker while the job is paused.
    ///
    /// Called from the archive thread only. The default returns at once.
    /// A paused RAR entry still finishes its current staged copy, then waits
    /// on the next `ensure_running` check.
    fn wait_if_paused(&self) {}
}

/// Stop when the sink has been cancelled.
///
/// # Errors
///
/// Returns [`crate::Error::Interrupted`] after a cancel request.
pub fn ensure_running(progress: &dyn ProgressSink) -> Result<(), crate::Error> {
    progress.wait_if_paused();
    if progress.is_cancelled() {
        Err(crate::Error::Interrupted)
    } else {
        Ok(())
    }
}

/// Discards every progress event. Used by tests and quiet mode.
#[derive(Debug, Default, Clone, Copy)]
pub struct NopProgress;

impl ProgressSink for NopProgress {
    fn begin(&self, _label: &str, _total_bytes: u64) {}
    fn begin_entry(&self, _name: &str, _bytes: u64) {}
    fn advance(&self, _bytes: u64) {}
    fn end_entry(&self) {}
    fn message(&self, _text: &str) {}
}

/// Counts reads and forwards them to a [`ProgressSink`].
pub struct ProgressReader<'a, R> {
    inner: R,
    progress: &'a dyn ProgressSink,
}

impl<'a, R> ProgressReader<'a, R> {
    /// Wrap `inner` so each successful read advances `progress`.
    pub fn new(inner: R, progress: &'a dyn ProgressSink) -> Self {
        Self { inner, progress }
    }
}

impl<R: std::io::Read> std::io::Read for ProgressReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.progress.wait_if_paused();
        if self.progress.is_cancelled() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "interrupted",
            ));
        }
        let n = self.inner.read(buf)?;
        if n > 0 {
            self.progress.advance(n as u64);
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct CountWait {
        waits: AtomicUsize,
    }

    impl ProgressSink for CountWait {
        fn begin(&self, _label: &str, _total_bytes: u64) {}
        fn begin_entry(&self, _name: &str, _bytes: u64) {}
        fn advance(&self, _bytes: u64) {}
        fn end_entry(&self) {}
        fn message(&self, _text: &str) {}
        fn wait_if_paused(&self) {
            self.waits.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn reader_and_ensure_running_honor_pause() {
        let sink = CountWait {
            waits: AtomicUsize::new(0),
        };
        ensure_running(&sink).unwrap();
        let mut reader = ProgressReader::new(Cursor::new(b"ab"), &sink);
        let mut buf = [0; 8];
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert!(sink.waits.load(Ordering::Relaxed) >= 2);
    }
}
