// Plugin bindings for plugin development (available with "plugin" feature)
#[cfg(feature = "plugin")]
pub mod bindings;

#[cfg(feature = "plugin")]
pub use bindings::*;

// Plugin loading and running (only available in CLI)
#[cfg(feature = "cli")]
pub mod loader;

#[cfg(feature = "cli")]
pub mod runner;
