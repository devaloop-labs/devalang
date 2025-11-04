/// Audio interpreter - executes statements and generates audio events
pub mod audio_graph;
pub mod driver;
pub mod statements;

// Re-export driver child modules so external paths like
// crate::engine::audio::interpreter::collector still work
pub use audio_graph::AudioGraph;
pub use driver::AudioInterpreter;
pub use driver::{collector, extractor, handler, renderer};

#[cfg(test)]
#[path = "test_control_flow.rs"]
mod test_control_flow;
