//! Background jobs. The window only paints; archive I/O stays off the UI thread.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use shellzip_core::{Entry, Error, ExtractUpdate, Overwrite, ProgressSink};

use crate::explorer::Row;
use crate::settings::DeleteArchive;

/// Identifies one worker result so a browse listing cannot finish an extract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JobToken {
    /// Monotonic id. Directory reads use `0`.
    pub id: u64,
    /// Navigation generation that started the job.
    pub generation: u64,
}

/// Counters and timing the window reads each frame.
pub struct ProgressSnapshot {
    /// Operation label, such as `extract notes.zip`.
    pub label: String,
    /// Current member name.
    pub entry: String,
    /// Payload bytes finished for the whole job.
    pub done: u64,
    /// Uncompressed size of the job, or `0` when it is unknown.
    pub total: u64,
    /// Payload bytes finished for the current member.
    pub entry_done: u64,
    /// Uncompressed size of the current member, or `0` when it is unknown.
    pub entry_total: u64,
    /// Time spent running, excluding pause.
    pub elapsed: Duration,
    /// The worker is parked until [`SharedProgress::set_paused`] is cleared.
    pub paused: bool,
}

struct Clock {
    started: Option<Instant>,
    pause_started: Option<Instant>,
    paused_total: Duration,
    paused: bool,
}

struct PauseGate {
    paused: bool,
}

/// Byte counters the window reads each frame.
pub struct SharedProgress {
    label: Mutex<String>,
    entry: Mutex<String>,
    done: AtomicU64,
    total: AtomicU64,
    entry_done: AtomicU64,
    entry_total: AtomicU64,
    stop: AtomicBool,
    paused_hint: AtomicBool,
    clock: Mutex<Clock>,
    gate: Mutex<PauseGate>,
    resume: Condvar,
}

impl SharedProgress {
    /// Zeroed counters.
    pub fn new() -> Self {
        Self {
            label: Mutex::new(String::new()),
            entry: Mutex::new(String::new()),
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
            entry_done: AtomicU64::new(0),
            entry_total: AtomicU64::new(0),
            stop: AtomicBool::new(false),
            paused_hint: AtomicBool::new(false),
            clock: Mutex::new(Clock {
                started: None,
                pause_started: None,
                paused_total: Duration::ZERO,
                paused: false,
            }),
            gate: Mutex::new(PauseGate { paused: false }),
            resume: Condvar::new(),
        }
    }

    /// Clear counters at the start of a job.
    pub fn reset(&self) {
        *lock(&self.label) = String::new();
        *lock(&self.entry) = String::new();
        self.done.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
        self.entry_done.store(0, Ordering::Relaxed);
        self.entry_total.store(0, Ordering::Relaxed);
        self.stop.store(false, Ordering::Relaxed);
        {
            let mut clock = lock(&self.clock);
            clock.started = None;
            clock.pause_started = None;
            clock.paused_total = Duration::ZERO;
            clock.paused = false;
        }
        self.set_paused(false);
    }

    /// Ask the running job to stop. The next read returns [`Error::Interrupted`].
    pub fn request_cancel(&self) {
        self.stop.store(true, Ordering::Release);
        let _gate = lock(&self.gate);
        self.resume.notify_all();
    }

    /// Park or release the worker. Pausing does not change the saved settings.
    pub fn set_paused(&self, paused: bool) {
        {
            let mut clock = lock(&self.clock);
            if paused {
                if clock.pause_started.is_none() {
                    clock.pause_started = Some(Instant::now());
                }
            } else if let Some(started) = clock.pause_started.take() {
                clock.paused_total = clock.paused_total.saturating_add(started.elapsed());
            }
            clock.paused = paused;
        }
        let mut gate = lock(&self.gate);
        gate.paused = paused;
        self.paused_hint.store(paused, Ordering::Release);
        self.resume.notify_all();
    }

    /// Current label, member, byte counters, and active elapsed time.
    pub fn snapshot(&self) -> ProgressSnapshot {
        let (elapsed, paused) = {
            let clock = lock(&self.clock);
            (active_elapsed(&clock), clock.paused)
        };
        ProgressSnapshot {
            label: lock(&self.label).clone(),
            entry: lock(&self.entry).clone(),
            done: self.done.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            entry_done: self.entry_done.load(Ordering::Relaxed),
            entry_total: self.entry_total.load(Ordering::Relaxed),
            elapsed,
            paused,
        }
    }
}

fn active_elapsed(clock: &Clock) -> Duration {
    let Some(started) = clock.started else {
        return Duration::ZERO;
    };
    let mut paused = clock.paused_total;
    if let Some(at) = clock.pause_started {
        paused = paused.saturating_add(at.elapsed());
    }
    started.elapsed().saturating_sub(paused)
}

impl ProgressSink for SharedProgress {
    fn begin(&self, label: &str, total_bytes: u64) {
        *lock(&self.label) = label.to_string();
        self.total.store(total_bytes, Ordering::Relaxed);
        self.done.store(0, Ordering::Relaxed);
        let mut clock = lock(&self.clock);
        if clock.started.is_none() {
            clock.started = Some(Instant::now());
        }
    }

    fn begin_entry(&self, name: &str, bytes: u64) {
        *lock(&self.entry) = name.to_string();
        self.entry_total.store(bytes, Ordering::Relaxed);
        self.entry_done.store(0, Ordering::Relaxed);
    }

    fn advance(&self, bytes: u64) {
        self.done.fetch_add(bytes, Ordering::Relaxed);
        self.entry_done.fetch_add(bytes, Ordering::Relaxed);
    }

    fn end_entry(&self) {}

    fn message(&self, text: &str) {
        *lock(&self.label) = text.to_string();
    }

    fn is_cancelled(&self) -> bool {
        self.stop.load(Ordering::Acquire)
    }

    fn wait_if_paused(&self) {
        if !self.paused_hint.load(Ordering::Acquire) {
            return;
        }
        let mut gate = lock(&self.gate);
        while gate.paused && !self.stop.load(Ordering::Acquire) {
            gate = self
                .resume
                .wait(gate)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}

/// Result delivered back to the window.
pub enum Outcome {
    /// A listing is ready to show.
    Listed {
        /// Job that produced the listing. An add or delete uses its operation id.
        token: JobToken,
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
    Finished {
        /// Job that finished.
        token: JobToken,
        /// Status line.
        summary: String,
    },
    /// An extract finished. The window may open Explorer or delete the archive.
    Extracted {
        /// Job that finished.
        token: JobToken,
        /// Status line.
        summary: String,
        /// Destination folder.
        output: PathBuf,
        /// Archive that was unpacked.
        archive: PathBuf,
        /// Open `output` in Explorer.
        show_in_explorer: bool,
        /// Whether to delete `archive`.
        delete: DeleteArchive,
        /// Send the archive to the recycle bin.
        recycle: bool,
        /// No warnings were reported.
        clean: bool,
        /// Every member was selected.
        full: bool,
        /// At least one file was written.
        wrote: bool,
    },
    /// The job failed. The text is safe to show.
    Failed {
        /// Job that failed.
        token: JobToken,
        /// Status line.
        message: String,
    },
    /// The archive needs a password before the job can continue.
    NeedPassword(PasswordRequest),
}

/// Enough to retry a job after the user types a password.
pub struct PasswordRequest {
    /// Worker id, so a stale prompt cannot restart a newer job.
    pub id: u64,
    /// Navigation generation that started the job.
    pub generation: u64,
    /// Archive that asked.
    pub path: PathBuf,
    /// What to run again.
    pub job: PasswordJob,
    /// The previous password was rejected.
    pub again: bool,
    /// Restore the progress window after a correct password.
    pub foreground: bool,
}

/// Work that stopped because a password was missing or wrong.
pub enum PasswordJob {
    /// Open the archive.
    List,
    /// Test checksums.
    Test,
    /// Extract into `output`.
    Extract(ExtractJob),
    /// Remove these members.
    Delete {
        /// Archive paths to remove.
        names: Vec<String>,
    },
}

/// Extract choices that survive a password retry.
pub struct ExtractJob {
    /// Destination folder.
    pub output: PathBuf,
    /// Replace, update, or freshen.
    pub update: ExtractUpdate,
    /// Collision policy.
    pub overwrite: Overwrite,
    /// Drop archive directories.
    pub flat: bool,
    /// Open archive folder removed from each name.
    pub strip_prefix: String,
    /// Keep a partial file when decoding fails.
    pub keep_broken: bool,
    /// Restore modification times.
    pub restore_mtime: bool,
    /// Open the destination in Explorer afterwards.
    pub show_in_explorer: bool,
    /// Delete the archive after a clean full extract.
    pub delete: DeleteArchive,
    /// Move a deleted archive to the recycle bin.
    pub recycle: bool,
    /// Member names, or every member when `None`.
    pub names: Option<Vec<String>>,
}

/// Run `job` on a new thread and send one [`Outcome`].
pub fn spawn(
    tx: Sender<Outcome>,
    token: JobToken,
    job: impl FnOnce() -> Result<Outcome, Error> + Send + 'static,
) {
    std::thread::spawn(move || {
        let outcome = match job() {
            Ok(outcome) => outcome,
            Err(Error::Interrupted) => Outcome::Finished {
                token,
                summary: "cancelled".to_string(),
            },
            Err(err) => Outcome::Failed {
                token,
                message: err.to_string(),
            },
        };
        let _ = tx.send(outcome);
    });
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use shellzip_core::ProgressSink;

    use super::*;

    #[test]
    fn entry_counters_follow_the_current_member() {
        let progress = SharedProgress::new();
        progress.begin("extract notes.zip", 100);
        progress.begin_entry("a.txt", 40);
        progress.advance(10);
        let first = progress.snapshot();
        assert_eq!(first.done, 10);
        assert_eq!(first.total, 100);
        assert_eq!(first.entry_done, 10);
        assert_eq!(first.entry_total, 40);
        assert_eq!(first.entry, "a.txt");
        progress.begin_entry("b.txt", 60);
        progress.advance(5);
        let second = progress.snapshot();
        assert_eq!(second.done, 15);
        assert_eq!(second.entry_done, 5);
        assert_eq!(second.entry_total, 60);
        assert_eq!(second.entry, "b.txt");
    }

    #[test]
    fn pause_blocks_until_continue() {
        let progress = Arc::new(SharedProgress::new());
        progress.set_paused(true);
        let worker = Arc::clone(&progress);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            worker.wait_if_paused();
            let _ = tx.send(worker.is_cancelled());
        });
        thread::sleep(Duration::from_millis(80));
        assert!(rx.try_recv().is_err());
        progress.set_paused(false);
        assert!(!rx.recv_timeout(Duration::from_secs(2)).expect("resumed"));
    }

    #[test]
    fn cancel_during_pause_unblocks() {
        let progress = Arc::new(SharedProgress::new());
        progress.set_paused(true);
        let worker = Arc::clone(&progress);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            worker.wait_if_paused();
            let _ = tx.send(worker.is_cancelled());
        });
        thread::sleep(Duration::from_millis(80));
        progress.request_cancel();
        assert!(rx.recv_timeout(Duration::from_secs(2)).expect("cancelled"));
    }

    #[test]
    fn elapsed_time_skips_the_pause() {
        let progress = SharedProgress::new();
        progress.begin("add notes.zip", 1000);
        progress.set_paused(true);
        let at_pause = progress.snapshot().elapsed;
        thread::sleep(Duration::from_millis(150));
        let still = progress.snapshot().elapsed;
        assert!(
            still.saturating_sub(at_pause) < Duration::from_millis(80),
            "paused elapsed grew from {at_pause:?} to {still:?}"
        );
        assert!(progress.snapshot().paused);
        progress.set_paused(false);
        assert!(!progress.snapshot().paused);
    }
}
