//! Background jobs. The window only paints; archive I/O stays off the UI thread.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::Sender;

use shellzip_core::{Entry, Error, Overwrite, ProgressSink};

use crate::explorer::Row;

/// Byte counters the window reads each frame.
pub struct SharedProgress {
    label: Mutex<String>,
    entry: Mutex<String>,
    done: AtomicU64,
    total: AtomicU64,
    stop: AtomicBool,
}

impl SharedProgress {
    /// Zeroed counters.
    pub fn new() -> Self {
        Self {
            label: Mutex::new(String::new()),
            entry: Mutex::new(String::new()),
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
            stop: AtomicBool::new(false),
        }
    }

    /// Clear counters at the start of a job.
    pub fn reset(&self) {
        *lock(&self.label) = String::new();
        *lock(&self.entry) = String::new();
        self.done.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
        self.stop.store(false, Ordering::Relaxed);
    }

    /// Ask the running job to stop. The next read returns [`Error::Interrupted`].
    pub fn request_cancel(&self) {
        self.stop.store(true, Ordering::Relaxed);
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

    fn is_cancelled(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
}

/// Result delivered back to the window.
pub enum Outcome {
    /// A listing is ready to show.
    Listed {
        /// Bumped when the user navigates, so a slow listing cannot overwrite a newer place.
        generation: u64,
        /// Archive that was opened.
        path: PathBuf,
        /// Format label, such as `ZIP` or `TAR.GZ`.
        format: String,
        /// Members in archive order.
        entries: Vec<Entry>,
    },
    /// One filesystem folder, read off the UI thread.
    Directory {
        /// Matches [`Outcome::Listed`] so an older folder read is ignored.
        generation: u64,
        /// Folder that was read.
        path: PathBuf,
        /// Immediate children. `..` is not included.
        rows: Vec<Row>,
        /// Set when the directory could not be read.
        error: Option<String>,
    },
    /// The job finished and `summary` can go in the status line.
    Finished(String),
    /// The job failed. The text is safe to show.
    Failed(String),
    /// The archive needs a password before the job can continue.
    NeedPassword(PasswordRequest),
}

/// Enough to retry a job after the user types a password.
pub struct PasswordRequest {
    /// Navigation generation that started the job.
    pub generation: u64,
    /// Archive that asked.
    pub path: PathBuf,
    /// What to run again.
    pub job: PasswordJob,
    /// The previous password was rejected.
    pub again: bool,
}

/// Work that stopped because a password was missing or wrong.
pub enum PasswordJob {
    /// Open the archive.
    List,
    /// Test checksums.
    Test,
    /// Extract into `output`.
    Extract {
        /// Destination folder.
        output: PathBuf,
        /// Drop archive directories.
        flat: bool,
        /// Collision policy.
        overwrite: Overwrite,
        /// Member names, or every member when `None`.
        names: Option<Vec<String>>,
    },
    /// Remove these members.
    Delete {
        /// Archive paths to remove.
        names: Vec<String>,
    },
}

/// Run `job` on a new thread and send one [`Outcome`].
pub fn spawn(tx: Sender<Outcome>, job: impl FnOnce() -> Result<Outcome, Error> + Send + 'static) {
    std::thread::spawn(move || {
        let outcome = match job() {
            Ok(outcome) => outcome,
            Err(Error::Interrupted) => Outcome::Finished("cancelled".to_string()),
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
