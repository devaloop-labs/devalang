pub mod loader;
/// Preprocessor module - handles statement resolution, variable expansion, and module loading
pub mod resolver;

pub use resolver::{resolve_statement, resolve_value};
