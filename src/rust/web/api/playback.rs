//! Playback and addon management API for WASM

use crate::web::registry::{banks, debug, hotreload, playhead, samples};
use wasm_bindgen::prelude::*;

/// Enable hot reload mode with callback
#[wasm_bindgen]
pub fn enable_hot_reload(callback: js_sys::Function) {
    hotreload::enable_hot_reload(callback);
}

/// Disable hot reload mode
#[wasm_bindgen]
pub fn disable_hot_reload() {
    hotreload::disable_hot_reload();
}

/// Check if hot reload is enabled
#[wasm_bindgen]
pub fn is_hot_reload_enabled() -> bool {
    hotreload::is_hot_reload_enabled()
}

/// Register a JavaScript callback for playhead events
///
/// The callback will be called for each note on/off event during audio rendering.
/// Call the returned unregister function to remove the callback.
#[wasm_bindgen]
pub fn register_playhead_callback(callback: js_sys::Function) -> JsValue {
    playhead::register_callback(callback);

    // Create and return unregister function
    let unregister = Closure::wrap(Box::new(|| {
        playhead::unregister_callback();
    }) as Box<dyn Fn()>);

    let result = unregister.as_ref().clone();
    unregister.forget();
    result
}

/// Collect all playhead events that have been generated
///
/// Returns an array of playhead events with timing and note information.
/// Events are not cleared after collection - use this for pre-scheduling.
#[wasm_bindgen]
pub fn collect_playhead_events() -> Result<JsValue, JsValue> {
    let events = playhead::get_events();
    serde_wasm_bindgen::to_value(&events)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Register an audio bank addon
///
/// Format: "/addons/banks/<publisher>/<name>" or "devalang://bank/<publisher>.<name>"
/// This is a simplified implementation that registers a bank without loading samples.
/// Samples should be registered separately using register_sample().
#[wasm_bindgen]
pub fn register_addon(path: &str) -> Result<JsValue, JsValue> {
    // Parse the path to extract publisher and name
    // Support both formats:
    // - "/addons/banks/devaloop/808"
    // - "devalang://bank/devaloop.808"

    let (full_name, alias) = if path.starts_with("devalang://bank/") {
        let parts = path.trim_start_matches("devalang://bank/");
        let name = parts.split('?').next().unwrap_or(parts);
        let alias = name.split('.').last().unwrap_or(name);
        (name.to_string(), alias.to_string())
    } else if path.starts_with("/addons/banks/") || path.contains("/banks/") {
        // Extract from path like "/addons/banks/devaloop/808"
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 3 {
            let publisher = parts[parts.len() - 2];
            let name = parts[parts.len() - 1];
            let full_name = format!("{}.{}", publisher, name);
            (full_name, name.to_string())
        } else {
            return Err(JsValue::from_str(&format!(
                "Invalid bank path format: {}",
                path
            )));
        }
    } else {
        return Err(JsValue::from_str(&format!(
            "Invalid bank path format: {}",
            path
        )));
    };

    // Register an empty bank (triggers will be added via register_sample)
    banks::register_bank(
        full_name.clone(),
        alias.clone(),
        std::collections::HashMap::new(),
    );

    debug::log_playback_debug(format!("Registered bank: {} (alias: {})", full_name, alias));

    Ok(JsValue::from_bool(true))
}

/// Register a sample with PCM data
#[wasm_bindgen]
pub fn register_sample(uri: &str, pcm: &js_sys::Float32Array) -> Result<JsValue, JsValue> {
    // Convert Float32Array to Vec<i16>
    let pcm_f32 = pcm.to_vec();
    let pcm_i16: Vec<i16> = pcm_f32
        .iter()
        .map(|&sample| (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

    let sample_count = pcm_i16.len();
    samples::register_sample(uri.to_string(), pcm_i16);
    debug::log_sample_load(format!(
        "Registered sample: {} ({} samples)",
        uri, sample_count
    ));

    Ok(JsValue::from_bool(true))
}

/// List registered banks
#[wasm_bindgen]
pub fn list_registered_banks() -> Result<JsValue, JsValue> {
    let banks_list = banks::list_banks();
    serde_wasm_bindgen::to_value(&banks_list)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// List registered samples
#[wasm_bindgen]
pub fn list_registered_samples(limit: usize) -> Result<JsValue, JsValue> {
    let sample_uris = samples::list_sample_uris();
    let limited: Vec<String> = sample_uris.into_iter().take(limit).collect();

    serde_wasm_bindgen::to_value(&limited)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get sample load events log
#[wasm_bindgen]
pub fn sample_load_events(clear: bool) -> Result<JsValue, JsValue> {
    let log = debug::get_sample_load_log(clear);
    serde_wasm_bindgen::to_value(&log)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get playback debug log
#[wasm_bindgen]
pub fn collect_playback_debug(clear: bool) -> Result<JsValue, JsValue> {
    let log = debug::get_playback_debug_log(clear);
    serde_wasm_bindgen::to_value(&log)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get playback debug state
#[wasm_bindgen]
pub fn playback_debug_state() -> Result<JsValue, JsValue> {
    let state = debug::get_debug_state();
    serde_wasm_bindgen::to_value(&state)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Enable/disable debug error logging
#[wasm_bindgen]
pub fn set_wasm_debug_errors(enable: bool) {
    debug::set_debug_errors(enable);
}

/// Get last errors (legacy string format)
#[wasm_bindgen]
pub fn collect_last_errors(clear: bool) -> Result<JsValue, JsValue> {
    let errors = debug::get_errors(clear);
    serde_wasm_bindgen::to_value(&errors)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get structured parse errors with line/column information
#[wasm_bindgen]
pub fn collect_parse_errors(clear: bool) -> Result<JsValue, JsValue> {
    let errors = debug::get_parse_errors(clear);
    serde_wasm_bindgen::to_value(&errors)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
