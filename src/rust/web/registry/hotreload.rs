//! Hot reload support for WASM
//!
//! Allows watching code changes and automatically re-rendering audio

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    /// Hot reload callback
    static HOT_RELOAD_CALLBACK: RefCell<Option<js_sys::Function>> = RefCell::new(None);

    /// Hot reload enabled flag
    static HOT_RELOAD_ENABLED: RefCell<bool> = RefCell::new(false);
}

/// Enable hot reload with a callback
pub fn enable_hot_reload(callback: js_sys::Function) {
    HOT_RELOAD_CALLBACK.with(|cb| {
        *cb.borrow_mut() = Some(callback);
    });
    HOT_RELOAD_ENABLED.with(|enabled| {
        *enabled.borrow_mut() = true;
    });
}

/// Disable hot reload
pub fn disable_hot_reload() {
    HOT_RELOAD_ENABLED.with(|enabled| {
        *enabled.borrow_mut() = false;
    });
    HOT_RELOAD_CALLBACK.with(|cb| {
        *cb.borrow_mut() = None;
    });
}

/// Check if hot reload is enabled
pub fn is_hot_reload_enabled() -> bool {
    HOT_RELOAD_ENABLED.with(|enabled| *enabled.borrow())
}

/// Trigger hot reload callback
pub fn trigger_hot_reload() {
    HOT_RELOAD_CALLBACK.with(|cb| {
        if let Some(callback) = cb.borrow().as_ref() {
            let _ = callback.call0(&JsValue::NULL);
        }
    });
}
