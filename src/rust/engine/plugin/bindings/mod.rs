//! Plugin bindings module
//! 
//! This module exports types, macros, and utilities for writing Devalang plugins.
//! Plugin authors should use this module to create WASM plugins that can be loaded
//! into Devalang.

pub mod macros;
pub mod types;
pub mod oscillators;

// Re-export commonly used items
pub use types::{Note, BufferParams, RenderFn, RenderFnExt, Waveform};
pub use oscillators::{Oscillator, ADSREnvelope, LowPassFilter};
