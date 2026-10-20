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
        let n = self.inner.read(buf)?;
        if n > 0 {
            self.progress.advance(n as u64);
        }
        Ok(n)
    }
}
