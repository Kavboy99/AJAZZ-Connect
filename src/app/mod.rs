//! Application state without the interface: the device session, diagnostics, settings and the
//! event channel from short-lived worker threads. Knows no brands and no drivers by name.

mod backup_discovery;
pub mod backup_session;
pub mod diagnostics;
mod discovery;
pub mod events;
pub mod identity;
pub mod session;
pub mod settings;
pub mod watch;

pub use diagnostics::Diagnostics;
pub use events::{Event, Events};
pub use session::Session;
pub use settings::{Settings, Theme};
pub use watch::{Due, Watch};
pub mod catalog;

pub mod battery;
