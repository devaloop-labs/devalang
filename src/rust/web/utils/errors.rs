//! Error handling utilities for WASM

use wasm_bindgen::JsValue;

/// Convert a Rust error to JsValue
pub fn to_js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&format!("{}", error))
}

/// Create a JS error with a message
pub fn js_error(message: &str) -> JsValue {
    JsValue::from_str(message)
}

/// Wrap a Result in a JS-friendly format
pub fn wrap_result<T, E>(result: Result<T, E>) -> Result<JsValue, JsValue>
where
    T: serde::Serialize,
    E: std::fmt::Display,
{
    match result {
        Ok(value) => serde_wasm_bindgen::to_value(&value).map_err(|e| to_js_error(e)),
        Err(e) => Err(to_js_error(e)),
    }
}
