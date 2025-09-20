use devalang_types::Value;
use std::collections::HashMap;

pub mod dsp;
pub mod params;

// Minimal safe facade: parse the setup then call DSP renderer.

pub fn insert_note_impl(
    engine: &mut crate::core::audio::engine::AudioEngine,
    owner: Option<String>,
    waveform: String,
    freq: f32,
    amp: f32,
    start_time_ms: f32,
    duration_ms: f32,
    synth_params: HashMap<String, Value>,
    note_params: HashMap<String, Value>,
    automation: Option<HashMap<String, Value>>,
) -> Vec<(usize, usize)> {
    let setup = params::build_note_setup(
        engine,
        &waveform,
        freq,
        amp,
        start_time_ms,
        duration_ms,
        &synth_params,
        &note_params,
        &automation,
    );

    let ranges = dsp::render_notes_into_buffer(
        engine,
        &waveform,
        freq,
        amp,
        start_time_ms,
        duration_ms,
        synth_params,
        note_params,
        automation,
        setup,
    );

    if let Some(owner_name) = owner {
        for (start, len) in ranges.iter() {
            engine.record_last_note_range(&owner_name, *start, *len);
        }
    }

    ranges
}
