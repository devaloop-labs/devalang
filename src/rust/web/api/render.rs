//! Rendering API for WASM
//!
//! Audio rendering functions for web playback.

use js_sys::{Float32Array, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::parser::driver::SimpleParser;
use crate::web::registry::banks;
use crate::web::utils::errors::to_js_error;

#[derive(Serialize, Deserialize)]
pub struct RenderOptions {
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_bpm")]
    pub bpm: f32,
}

fn default_sample_rate() -> u32 {
    44100
}
fn default_bpm() -> f32 {
    120.0
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bpm: 120.0,
        }
    }
}

#[derive(Serialize)]
pub struct DebugRenderResult {
    pub audio: Vec<f32>,
    pub sample_rate: u32,
    pub duration: f32,
    pub event_count: usize,
    pub bpm: f32,
}

/// Render audio from Devalang code
/// Returns audio buffer as Float32Array
#[wasm_bindgen]
pub fn render_audio(user_code: &str, options: JsValue) -> Result<Float32Array, JsValue> {
    // Clear previous playhead events and errors
    use crate::web::registry::{debug, playhead};
    playhead::clear_events();

    // Parse options
    let opts: RenderOptions = if options.is_undefined() || options.is_null() {
        RenderOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|e| {
            if debug::is_debug_errors_enabled() {
                debug::push_parse_error_from_parts(
                    format!("Invalid options: {}", e),
                    0,
                    0,
                    "OptionsError".to_string(),
                );
            }
            to_js_error(&format!("Invalid options: {}", e))
        })?
    };

    // Parse code
    let statements = SimpleParser::parse(user_code, std::path::PathBuf::from("wasm_input.deva"))
        .map_err(|e| {
            if debug::is_debug_errors_enabled() {
                debug::push_parse_error_from_parts(
                    format!("Parse error: {:?}", e),
                    0,
                    0,
                    "ParseError".to_string(),
                );
            }
            to_js_error(&format!("Parse error: {:?}", e))
        })?;

    // Create audio interpreter
    let mut interpreter = AudioInterpreter::new(opts.sample_rate);
    interpreter.bpm = opts.bpm;

    // Inject registered banks
    banks::inject_registered_banks(&mut interpreter);

    // Render audio buffer
    let buffer = interpreter.interpret(&statements).map_err(|e| {
        let error_msg = format!("{}", e);
        if debug::is_debug_errors_enabled() {
            // Try to get statement location from interpreter
            let (line, column) = interpreter.current_statement_location().unwrap_or((0, 0));

            debug::push_parse_error_from_parts(
                error_msg.clone(),
                line,
                column,
                "RuntimeError".to_string(),
            );
        }
        to_js_error(&format!("Render error: {}", error_msg))
    })?;

    // Convert to Float32Array
    let array = Float32Array::new_with_length(buffer.len() as u32);
    array.copy_from(&buffer);

    Ok(array)
}

/// Render audio with debug information
#[wasm_bindgen]
pub fn debug_render(user_code: &str, options: JsValue) -> Result<JsValue, JsValue> {
    // Clear previous playhead events
    use crate::web::registry::playhead;
    playhead::clear_events();

    // Parse options
    let opts: RenderOptions = if options.is_undefined() || options.is_null() {
        RenderOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| to_js_error(&format!("Invalid options: {}", e)))?
    };

    // Parse code
    let statements = SimpleParser::parse(user_code, std::path::PathBuf::from("wasm_input.deva"))
        .map_err(|e| to_js_error(&format!("Parse error: {:?}", e)))?;

    // Create audio interpreter
    let mut interpreter = AudioInterpreter::new(opts.sample_rate);
    interpreter.bpm = opts.bpm;

    // Inject registered banks
    banks::inject_registered_banks(&mut interpreter);

    // Render audio buffer
    let buffer = interpreter
        .interpret(&statements)
        .map_err(|e| to_js_error(&format!("Render error: {}", e)))?;

    // Get event count from interpreter
    let event_count = interpreter.events().events.len();
    let duration = buffer.len() as f32 / opts.sample_rate as f32;

    // Build debug result
    let result = DebugRenderResult {
        audio: buffer,
        sample_rate: opts.sample_rate,
        duration,
        event_count,
        bpm: opts.bpm,
    };

    // Serialize to JS
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| to_js_error(&format!("Serialization error: {}", e)))
}

/// Render WAV file preview
#[wasm_bindgen]
pub fn render_wav_preview(
    user_code: &str,
    options: JsValue,
    on_progress: Option<js_sys::Function>,
) -> Result<Uint8Array, JsValue> {
    // Parse options
    let opts: RenderOptions = if options.is_undefined() || options.is_null() {
        RenderOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| to_js_error(&format!("Invalid options: {}", e)))?
    };

    // Call progress callback at start
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
    }

    // Parse code
    let statements = SimpleParser::parse(user_code, std::path::PathBuf::from("wasm_input.deva"))
        .map_err(|e| to_js_error(&format!("Parse error: {:?}", e)))?;

    // Progress: parsing done (25%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.25));
    }

    // Create audio interpreter
    let mut interpreter = AudioInterpreter::new(opts.sample_rate);
    interpreter.bpm = opts.bpm;

    // Inject registered banks
    banks::inject_registered_banks(&mut interpreter);

    // Progress: setup done (50%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.5));
    }

    // Render audio buffer
    let buffer = interpreter
        .interpret(&statements)
        .map_err(|e| to_js_error(&format!("Render error: {}", e)))?;

    // Progress: rendering done (75%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.75));
    }

    // Convert to WAV bytes using utility function
    use crate::web::utils::conversion::pcm_to_wav_bytes;
    let wav_bytes = pcm_to_wav_bytes(&buffer, opts.sample_rate)
        .map_err(|e| to_js_error(&format!("WAV encoding error: {}", e)))?;

    // Convert to Uint8Array
    let array = Uint8Array::new_with_length(wav_bytes.len() as u32);
    array.copy_from(&wav_bytes);

    // Progress: complete (100%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(1.0));
    }

    Ok(array)
}

/// Get code to buffer metadata with duration calculation
#[wasm_bindgen]
pub fn get_code_to_buffer_metadata(user_code: &str, options: JsValue) -> Result<JsValue, JsValue> {
    // Parse options
    let opts: RenderOptions = if options.is_undefined() || options.is_null() {
        RenderOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| to_js_error(&format!("Invalid options: {}", e)))?
    };

    // Parse code
    let statements = SimpleParser::parse(user_code, std::path::PathBuf::from("wasm_input.deva"))
        .map_err(|e| to_js_error(&format!("Parse error: {:?}", e)))?;

    // Create audio interpreter
    let mut interpreter = AudioInterpreter::new(opts.sample_rate);
    interpreter.bpm = opts.bpm;

    // Inject registered banks
    banks::inject_registered_banks(&mut interpreter);

    // Get performance.now() for WASM-compatible timing
    let render_start = js_sys::Date::now();

    // Render buffer
    let buffer = interpreter
        .interpret(&statements)
        .map_err(|e| to_js_error(&format!("Render error: {}", e)))?;

    let render_end = js_sys::Date::now();
    let render_duration = (render_end - render_start) / 1000.0; // Convert ms to seconds

    // Calculate file duration
    let file_duration = buffer.len() as f64 / (opts.sample_rate as f64 * 2.0); // Stereo: divide by 2

    // Create metadata result with durations
    #[derive(Serialize)]
    struct MetadataResult {
        #[serde(rename = "fileDuration")]
        file_duration: f64,
        #[serde(rename = "renderDuration")]
        render_duration: f64,
        statement_count: usize,
        bpm: f32,
        sample_rate: u32,
    }

    let result = MetadataResult {
        file_duration,
        render_duration,
        statement_count: statements.len(),
        bpm: opts.bpm,
        sample_rate: opts.sample_rate,
    };

    // Serialize to JS
    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| to_js_error(&format!("Serialization error: {}", e)))
}
