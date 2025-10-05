//! MIDI export API for WASM

use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::parser::driver::SimpleParser;
use crate::web::registry::banks;
use crate::web::utils::errors::to_js_error;

#[derive(Serialize, Deserialize)]
pub struct MidiOptions {
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_bpm")]
    pub bpm: f32,
    #[serde(default = "default_time_sig_num")]
    pub time_signature_num: u8,
    #[serde(default = "default_time_sig_den")]
    pub time_signature_den: u8,
}

fn default_sample_rate() -> u32 {
    44100
}
fn default_bpm() -> f32 {
    120.0
}
fn default_time_sig_num() -> u8 {
    4
}
fn default_time_sig_den() -> u8 {
    4
}

impl Default for MidiOptions {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bpm: 120.0,
            time_signature_num: 4,
            time_signature_den: 4,
        }
    }
}

/// Render MIDI from Devalang code
/// Returns MIDI file as Uint8Array
#[wasm_bindgen]
pub fn render_midi_array(
    user_code: &str,
    options: JsValue,
    on_progress: Option<js_sys::Function>,
) -> Result<Uint8Array, JsValue> {
    // Parse options
    let opts: MidiOptions = if options.is_undefined() || options.is_null() {
        MidiOptions::default()
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

    // Run interpreter to collect events
    let _ = interpreter
        .interpret(&statements)
        .map_err(|e| to_js_error(&format!("Interpret error: {}", e)))?;

    // Progress: interpretation done (75%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.75));
    }

    // Get collected events
    let events = &interpreter.events().events;

    // Convert to MIDI bytes using engine function
    use crate::engine::audio::midi::events_to_midi_bytes;
    let midi_bytes = events_to_midi_bytes(events, opts.bpm)
        .map_err(|e| to_js_error(&format!("MIDI export error: {}", e)))?;

    // Convert to Uint8Array
    let array = Uint8Array::new_with_length(midi_bytes.len() as u32);
    array.copy_from(&midi_bytes);

    // Progress: complete (100%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(1.0));
    }

    Ok(array)
}

/// Export MIDI file (browser download)
/// This is just a convenience wrapper that returns the same data as render_midi_array
#[wasm_bindgen]
pub fn export_midi_file(
    user_code: &str,
    options: JsValue,
    on_progress: Option<js_sys::Function>,
) -> Result<Uint8Array, JsValue> {
    // Reuse render_midi_array implementation
    render_midi_array(user_code, options, on_progress)
}
