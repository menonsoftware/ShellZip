//! Command implementations. Each one calls `shellzip_core` and prints a summary.

mod add;
mod delete;
mod extract;
mod list;
mod print;
mod sfx;
mod test;

pub use add::add;
pub use delete::delete;
pub use extract::extract;
pub use list::{list, list_technical};
pub use print::print_entry;
pub use sfx::sfx;
pub use test::test;

use shellzip_core::{Password, ProgressSink};

/// Quiet level, progress sink, and optional password for one invocation.
pub struct Shared {
    /// `1` hides bars. `2` also hides the success summary.
    pub quiet: u8,
    /// Bars, or a sink that discards events.
    pub sink: Box<dyn ProgressSink>,
    /// Password from `-p`, a prompt, or `SZIP_PASSWORD`. Never logged.
    pub password: Option<Password>,
}
