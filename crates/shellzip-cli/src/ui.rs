//! Progress bars and the overwrite prompt.

use std::io::{IsTerminal, Write};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use shellzip_core::{ConflictChoice, Error, NopProgress, ProgressSink};

/// Byte bars for the overall job and the current member.
pub struct Bars {
    _multi: MultiProgress,
    overall: ProgressBar,
    current: ProgressBar,
}

impl Bars {
    /// Two bars on stderr, cleared when the command finishes.
    pub fn new() -> Self {
        let multi = MultiProgress::new();
        let overall = multi.add(ProgressBar::new(0));
        let current = multi.add(ProgressBar::new(0));
        let style = byte_style();
        overall.set_style(style.clone());
        current.set_style(style);
        Self {
            _multi: multi,
            overall,
            current,
        }
    }
}

impl Drop for Bars {
    fn drop(&mut self) {
        self.current.finish_and_clear();
        self.overall.finish_and_clear();
    }
}

impl ProgressSink for Bars {
    fn begin(&self, label: &str, total_bytes: u64) {
        self.overall.reset();
        self.overall.set_length(total_bytes);
        self.overall.set_message(label.to_string());
    }

    fn begin_entry(&self, name: &str, bytes: u64) {
        self.current.reset();
        self.current.set_length(bytes);
        self.current.set_message(name.to_string());
    }

    fn advance(&self, bytes: u64) {
        self.overall.inc(bytes);
        self.current.inc(bytes);
    }

    fn end_entry(&self) {
        self.current.finish_and_clear();
    }

    fn message(&self, text: &str) {
        self.overall.set_message(text.to_string());
    }
}

/// Progress for this invocation. `-q` discards the bars.
pub fn sink(quiet: u8) -> Box<dyn ProgressSink> {
    if quiet == 0 {
        Box::new(Bars::new())
    } else {
        Box::new(NopProgress)
    }
}

/// Asks on a terminal. Later answers can apply to every remaining file.
pub struct Ask {
    always: AtomicBool,
    lock: Mutex<()>,
}

impl Ask {
    /// Prompt until the user picks a standing answer.
    pub fn new() -> Self {
        Self {
            always: AtomicBool::new(false),
            lock: Mutex::new(()),
        }
    }

    /// Resolve one collision. A non-interactive stdin skips the file.
    pub fn decide(&self, path: &Path) -> Result<ConflictChoice, Error> {
        if self.always.load(Ordering::Relaxed) {
            return Ok(ConflictChoice::Overwrite);
        }
        if !std::io::stdin().is_terminal() {
            return Ok(ConflictChoice::Skip);
        }
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if self.always.load(Ordering::Relaxed) {
            return Ok(ConflictChoice::Overwrite);
        }
        anstream::eprint!(
            "{} {}? [y]es [n]o [r]ename [a]ll [q]uit ",
            style("overwrite").cyan(),
            path.display()
        );
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .map_err(|err| Error::io("read overwrite answer", err))?;
        match line.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => Ok(ConflictChoice::Overwrite),
            "r" | "rename" => Ok(ConflictChoice::Rename),
            "a" | "all" => {
                self.always.store(true, Ordering::Relaxed);
                Ok(ConflictChoice::Overwrite)
            }
            "q" | "quit" => Err(Error::Interrupted),
            _ => Ok(ConflictChoice::Skip),
        }
    }
}

fn byte_style() -> ProgressStyle {
    let template = "{msg:.dim} {bar:24.cyan/blue} {bytes:>8}/{total_bytes:<8}";
    match ProgressStyle::with_template(template) {
        Ok(style) => style.progress_chars("=> "),
        Err(_) => ProgressStyle::default_bar(),
    }
}
