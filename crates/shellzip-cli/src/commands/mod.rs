//! Command implementations. Each one calls `shellzip_core` and prints a summary.

mod add;
mod extract;
mod list;
mod test;

pub use add::add;
pub use extract::extract;
pub use list::list;
pub use test::test;

use shellzip_core::ProgressSink;

/// Quiet level and the progress sink for one invocation.
pub struct Shared {
    /// `1` hides bars. `2` also hides the success summary.
    pub quiet: u8,
    /// Bars, or a sink that discards events.
    pub sink: Box<dyn ProgressSink>,
}
