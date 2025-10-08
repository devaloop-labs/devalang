//! # Devalang WASM Module
//!
//! WebAssembly bindings for Devalang, enabling browser-based audio synthesis,
//! MIDI generation, and code parsing.
//!
//! This module is organized into several submodules for better maintainability:
//!
//! - `registry`: Manages registered banks, samples, and debug state
//! - `api`: Public WASM-bindgen exported functions for JS interop
//! - `utils`: Utilities for data conversion and error handling

#![cfg(all(target_arch = "wasm32", feature = "wasm"))]

pub mod api;
pub mod registry;
pub mod utils;

// Re-export main API functions for convenience
pub use api::midi::*;
pub use api::parse::*;
pub use api::playback::*;
pub use api::render::*;
