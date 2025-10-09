//! Plugin bindings module
//!
//! This module exports types, macros, and utilities for writing Devalang plugins.
//! Plugin authors should use this module to create WASM plugins that can be loaded
//! into Devalang.

pub mod macros;
pub mod oscillators;
pub mod types;

// Re-export commonly used items
pub use oscillators::{ADSREnvelope, LowPassFilter, Oscillator};
pub use types::{BufferParams, Note, RenderFn, RenderFnExt, Waveform};
