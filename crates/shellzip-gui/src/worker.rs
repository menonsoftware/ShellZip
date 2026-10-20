//! Background jobs. The window only paints; archive I/O stays off the UI thread.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;

use shellzip_core::{Entry, Error, ProgressSink};

/// Byte counters the window reads each frame.
pub struct SharedProgress {
    label: Mutex<String>,
    entry: Mutex<String>,
    done: AtomicU64,
    total: AtomicU64,
}

impl SharedProgress {
    /// Zeroed counters.
    pub fn new() -> Self {
        Self {
            label: Mutex::new(String::new()),
            entry: Mutex::new(String::new()),
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }
    }

    /// Clear counters at the start of a job.
    pub fn reset(&self) {
        *lock(&self.label) = String::new();
        *lock(&self.entry) = String::new();
        self.done.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
    }

    /// Current label, member name, bytes done, and total bytes.
    pub fn snapshot(&self) -> (String, String, u64, u64) {
        (
            lock(&self.label).clone(),
            lock(&self.entry).clone(),
            self.done.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }
}

impl ProgressSink for SharedProgress {
    fn begin(&self, label: &str, total_bytes: u64) {
        *lock(&self.label) = label.to_string();
        self.total.store(total_bytes, Ordering::Relaxed);
        self.done.store(0, Ordering::Relaxed);
    }

    fn begin_entry(&self, name: &str, _bytes: u64) {
        *lock(&self.entry) = name.to_string();
    }

    fn advance(&self, bytes: u64) {
        self.done.fetch_add(bytes, Ordering::Relaxed);
    }

    fn end_entry(&self) {}

    fn message(&self, text: &str) {
        *lock(&self.label) = text.to_string();
    }
}

/// Result delivered back to the window.
pub enum Outcome {
    /// A listing is ready to show.
    Listed {
        /// Archive that was opened.
        path: PathBuf,
        /// Format label, such as `ZIP` or `TAR.GZ`.
        format: String,
        /// Members in archive order.
        entries: Vec<Entry>,
    },
    /// The job finished and `summary` can go in the status line.
    Finished(String),
    /// The job failed. The text is safe to show.
    Failed(String),
}

/// Run `job` on a new thread and send one [`Outcome`].
pub fn spawn(tx: Sender<Outcome>, job: impl FnOnce() -> Result<Outcome, Error> + Send + 'static) {
    std::thread::spawn(move || {
        let outcome = match job() {
            Ok(outcome) => outcome,
            Err(err) => Outcome::Failed(err.to_string()),
        };
        let _ = tx.send(outcome);
    });
}

fn lock(mutex: &Mutex<String>) -> std::sync::MutexGuard<'_, String> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
