#![allow(dead_code)]
#![allow(clippy::module_inception)]

pub mod engine;
pub mod language;
pub mod shared;
pub mod utils;

// Plugin development SDK (available with "plugin" feature)
#[cfg(feature = "plugin")]
pub mod plugin {
    //! Plugin development SDK for Devalang
    //!
    //! This module provides types, macros, and utilities for writing WASM plugins.
    //! Enable the "plugin" feature to use this module.

    pub use crate::engine::plugin::bindings::*;

    // Re-export macros from crate root (they are exported there due to #[macro_export])
    pub use crate::{export_plugin, export_plugin_with_state, simple_oscillator_plugin};
}

// CLI-specific modules (requires terminal, file system, etc.)
#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod platform;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod services;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod tools;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod workspace;

// WebAssembly bindings (only compiled for wasm32 target with "wasm" feature)
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod web;

#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
pub mod web {
    //! Minimal web registry stubs for plugin builds

    pub mod registry {
        pub mod samples {
            pub fn get_sample(_name: &str) -> Option<Vec<f32>> {
                None
            }
        }
        pub mod debug {
            pub fn log(_msg: &str) {}
            pub fn is_debug_errors_enabled() -> bool {
                false
            }
            pub fn push_parse_error_from_parts(
                _source: String,
                _line: usize,
                _column: usize,
                _length: usize,
                _message: String,
                _severity: String,
            ) {
            }
        }
        pub mod banks {
            use std::collections::HashMap;
            pub static REGISTERED_BANKS: once_cell::sync::Lazy<
                std::sync::Mutex<HashMap<String, String>>,
            > = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(HashMap::new()));
        }
        pub mod playhead {
            #[derive(Default)]
            pub struct PlayheadEvent {
                pub kind: String,
                pub event_type: String,
                pub midi: Vec<u8>,
                pub time: f32,
                pub velocity: f32,
                pub synth_id: String,
                pub pitch: u8,
                pub sample: Option<String>,
            }
            pub fn push_event(_event: PlayheadEvent) {}
        }
    }
}

#[cfg(all(target_arch = "wasm32", not(feature = "wasm")))]
pub mod web_sys {
    //! Minimal web_sys stubs for plugin builds
    pub mod console {
        pub fn log_1(_msg: &str) {}
        pub fn warn_1(_msg: &str) {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod web {
    pub mod registry {
        pub mod samples {
            pub fn get_sample(_name: &str) -> Option<Vec<f32>> {
                None
            }
        }
        pub mod debug {
            pub fn log(_msg: &str) {}
            pub fn is_debug_errors_enabled() -> bool {
                false
            }
            pub fn push_parse_error_from_parts(
                _source: String,
                _line: usize,
                _column: usize,
                _length: usize,
                _message: String,
                _severity: String,
            ) {
            }
        }
        pub mod banks {
            pub struct BankEntry {
                pub full_name: String,
                pub alias: String,
            }
            use std::cell::RefCell;
            thread_local! {
                pub static REGISTERED_BANKS: RefCell<Vec<BankEntry>> = RefCell::new(Vec::new());
            }
        }
        pub mod playhead {
            pub fn push_event<T>(_event: T) {}
        }
    }
}

#[cfg(all(target_arch = "wasm32", not(any(feature = "wasm", feature = "cli"))))]
pub mod midly {
    //! Minimal midly stubs for plugin builds
    pub struct Header;
    pub enum Format {
        SingleTrack,
    }
    pub enum Timing {
        Metrical(u16),
    }

    #[derive(Debug, Clone)]
    pub struct TrackEvent {
        pub delta: u28,
        pub kind: TrackEventKind,
    }

    #[derive(Debug, Clone)]
    pub struct u28(pub u32);
    impl From<u32> for u28 {
        fn from(v: u32) -> Self {
            u28(v)
        }
    }

    #[derive(Debug, Clone)]
    pub struct u24(pub u32);
    impl From<u32> for u24 {
        fn from(v: u32) -> Self {
            u24(v)
        }
    }

    #[derive(Debug, Clone)]
    pub struct u7(pub u8);
    impl From<u8> for u7 {
        fn from(v: u8) -> Self {
            u7(v)
        }
    }

    #[derive(Debug, Clone)]
    pub struct u4(pub u8);
    impl From<u8> for u4 {
        fn from(v: u8) -> Self {
            u4(v)
        }
    }

    #[derive(Debug, Clone)]
    pub enum MetaMessage {
        Tempo(u24),
        EndOfTrack,
    }

    #[derive(Debug, Clone)]
    pub enum TrackEventKind {
        Meta(MetaMessage),
        Midi { channel: u4, message: MidiMessage },
    }

    #[derive(Debug, Clone)]
    pub enum MidiMessage {
        NoteOn { key: u7, vel: u7 },
        NoteOff { key: u7, vel: u7 },
    }

    pub struct Track(Vec<TrackEvent>);
    impl From<Vec<TrackEvent>> for Track {
        fn from(events: Vec<TrackEvent>) -> Self {
            Track(events)
        }
    }

    pub struct Smf {
        pub tracks: Vec<Track>,
    }

    impl Header {
        pub fn new(_format: Format, _timing: Timing) -> Self {
            Header
        }
    }

    impl Smf {
        pub fn new(_header: Header) -> Self {
            Smf { tracks: Vec::new() }
        }
        pub fn write<W: std::io::Write>(&self, _writer: &mut W) -> std::io::Result<()> {
            Ok(())
        }
    }
}
