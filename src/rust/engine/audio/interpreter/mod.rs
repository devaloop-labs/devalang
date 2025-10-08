/// Audio interpreter - executes statements and generates audio events
pub mod driver;
pub mod statements;

// Re-export driver child modules so external paths like
// crate::engine::audio::interpreter::collector still work
pub use driver::AudioInterpreter;
pub use driver::{collector, extractor, handler, renderer};
