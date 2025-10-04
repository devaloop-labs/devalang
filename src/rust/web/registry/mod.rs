//! Registry module for managing banks, samples, and debug state in WASM
//!
//! This module provides thread-local storage for:
//! - Registered audio banks (with triggers)
//! - Loaded audio samples (PCM data)
//! - Debug logs (sample loading, playback)
//! - Error tracking
//! - Playhead events for UI feedback
//! - Hot reload support

pub mod banks;
pub mod debug;
pub mod hotreload;
pub mod playhead;
pub mod samples;

pub use banks::*;
pub use debug::*;
pub use hotreload::*;
pub use playhead::*;
pub use samples::*;
