#[cfg(not(target_arch = "wasm32"))]
mod non_wasm;

#[cfg(target_arch = "wasm32")]
mod wasm32;

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm::WasmPluginRunner;

#[cfg(target_arch = "wasm32")]
pub use wasm32::WasmPluginRunner;
