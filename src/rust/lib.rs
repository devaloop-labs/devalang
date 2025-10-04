#![allow(dead_code)]
#![allow(clippy::module_inception)]

pub mod engine;
pub mod language;
pub mod shared;

// CLI-specific modules (requires terminal, file system, etc.)
#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod platform;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod services;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod tools;

#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]
pub mod workspace;

// WebAssembly bindings (only compiled for wasm32 target)
#[cfg(target_arch = "wasm32")]
pub mod web;
