//! Export API for WASM - MP3, WAV with options

use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::engine::audio::interpreter::driver::AudioInterpreter;
use crate::language::syntax::parser::driver::SimpleParser;
use crate::web::registry::banks;
use crate::web::utils::errors::to_js_error;

/// Export options for audio rendering
#[derive(Serialize, Deserialize)]
pub struct ExportOptions {
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_bpm")]
    pub bpm: f32,

    #[serde(default = "default_bit_depth")]
    pub bit_depth: u8, // 16, 24, or 32

    #[serde(default = "default_format")]
    pub format: String, // "wav", "mp3", "ogg", "flac", "opus"

    #[serde(default = "default_mp3_bitrate")]
    pub mp3_bitrate: u32, // 128, 192, 256, 320 (for MP3/OGG/Opus)

    #[serde(default = "default_quality")]
    pub quality: f32, // 0.0-10.0 (for OGG/Opus)
}

fn default_sample_rate() -> u32 {
    44100
}
fn default_bpm() -> f32 {
    120.0
}
fn default_bit_depth() -> u8 {
    16
}
fn default_format() -> String {
    "wav".to_string()
}
fn default_mp3_bitrate() -> u32 {
    192
}
fn default_quality() -> f32 {
    5.0
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bpm: 120.0,
            bit_depth: 16,
            format: "wav".to_string(),
            mp3_bitrate: 192,
            quality: 5.0,
        }
    }
}

/// Metadata about the rendered audio
#[derive(Serialize)]
pub struct RenderMetadata {
    pub duration: f32,
    pub sample_count: usize,
    pub event_count: usize,
    pub sample_rate: u32,
    pub bpm: f32,
    pub estimated_size_bytes: usize,
}

/// Get metadata for code without full rendering (fast preview)
#[wasm_bindgen]
pub fn get_render_metadata(user_code: &str, options: JsValue) -> Result<JsValue, JsValue> {
    use crate::web::registry::playhead;
    playhead::clear_events();

    // Parse options
    let opts: ExportOptions = if options.is_undefined() || options.is_null() {
        ExportOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| to_js_error(&format!("Invalid options: {}", e)))?
    };

    // Parse code
    let statements = SimpleParser::parse(user_code, std::path::PathBuf::from("wasm_input.deva"))
        .map_err(|e| to_js_error(&format!("Parse error: {:?}", e)))?;

    // Create audio interpreter (quick pass, no rendering)
    let mut interpreter = AudioInterpreter::new(opts.sample_rate);
    interpreter.bpm = opts.bpm;

    // Inject registered banks
    banks::inject_registered_banks(&mut interpreter);

    // Just collect events (no rendering)
    interpreter
        .collect_events(&statements)
        .map_err(|e| to_js_error(&format!("Event collection error: {}", e)))?;

    // Get duration and event count
    let duration = interpreter.events().total_duration();
    let event_count = interpreter.events().events.len();
    let sample_count = (duration * opts.sample_rate as f32) as usize;

    // Estimate file size based on format
    let estimated_size_bytes = match opts.format.as_str() {
        "wav" => {
            let bytes_per_sample = (opts.bit_depth / 8) as usize;
            44 + (sample_count * 2 * bytes_per_sample) // WAV header + stereo samples
        }
        "mp3" | "ogg" | "opus" => {
            // Compressed formats: (bitrate * duration) / 8
            ((opts.mp3_bitrate as f32 * duration) / 8.0) as usize
        }
        "flac" => {
            // FLAC is lossless but compressed, estimate ~50-60% of WAV size
            let bytes_per_sample = (opts.bit_depth / 8) as usize;
            let wav_size = 44 + (sample_count * 2 * bytes_per_sample);
            (wav_size as f32 * 0.55) as usize
        }
        _ => sample_count * 4, // Default to 32-bit float estimation
    };

    let metadata = RenderMetadata {
        duration,
        sample_count,
        event_count,
        sample_rate: opts.sample_rate,
        bpm: opts.bpm,
        estimated_size_bytes,
    };

    serde_wasm_bindgen::to_value(&metadata)
        .map_err(|e| to_js_error(&format!("Serialization error: {}", e)))
}

/// Export audio with format options (WAV 16/24/32 bit, MP3)
#[wasm_bindgen]
pub fn export_audio(
    user_code: &str,
    options: JsValue,
    on_progress: Option<js_sys::Function>,
) -> Result<Uint8Array, JsValue> {
    use crate::web::registry::playhead;
    playhead::clear_events();

    // Call progress callback at start
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
    }

    // Parse options
    let opts: ExportOptions = if options.is_undefined() || options.is_null() {
        ExportOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options)
            .map_err(|e| to_js_error(&format!("Invalid options: {}", e)))?
    };

    // Parse code
    let statements = SimpleParser::parse(user_code, std::path::PathBuf::from("wasm_input.deva"))
        .map_err(|e| to_js_error(&format!("Parse error: {:?}", e)))?;

    // Progress: parsing done (20%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.2));
    }

    // Create audio interpreter
    let mut interpreter = AudioInterpreter::new(opts.sample_rate);
    interpreter.bpm = opts.bpm;

    // Inject registered banks
    banks::inject_registered_banks(&mut interpreter);

    // Progress: setup done (40%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.4));
    }

    // Render audio buffer
    let buffer = interpreter
        .interpret(&statements)
        .map_err(|e| to_js_error(&format!("Render error: {}", e)))?;

    // Progress: rendering done (70%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.7));
    }

    // Convert to requested format using the encoder system
    let bytes = {
        use crate::engine::audio::encoders::{AudioFormat, EncoderOptions, encode_audio};

        let format = AudioFormat::from_str(&opts.format).ok_or_else(|| {
            to_js_error(&format!(
                "Unsupported format: '{}'. Supported formats: wav, mp3, ogg, flac, opus",
                opts.format
            ))
        })?;

        let encoder_opts = match format {
            AudioFormat::Wav => EncoderOptions::wav(opts.sample_rate, opts.bit_depth),
            AudioFormat::Mp3 => EncoderOptions::mp3(opts.sample_rate, opts.mp3_bitrate),
            AudioFormat::Ogg => EncoderOptions::ogg(opts.sample_rate, opts.quality),
            AudioFormat::Flac => EncoderOptions::flac(opts.sample_rate, opts.bit_depth),
            AudioFormat::Opus => {
                let mut opt = EncoderOptions::default();
                opt.format = AudioFormat::Opus;
                opt.sample_rate = opts.sample_rate;
                opt.bitrate_kbps = opts.mp3_bitrate;
                opt.quality = opts.quality;
                opt
            }
        };

        encode_audio(&buffer, &encoder_opts)
            .map_err(|e| to_js_error(&format!("Encoding error: {}", e)))?
    };

    // Convert to Uint8Array
    let array = Uint8Array::new_with_length(bytes.len() as u32);
    array.copy_from(&bytes);

    // Progress: complete (100%)
    if let Some(callback) = &on_progress {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(1.0));
    }

    Ok(array)
}
