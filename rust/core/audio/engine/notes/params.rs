use devalang_types::Value;
use std::collections::HashMap;

pub struct FilterSpec {
    pub kind: String,
    pub cutoff: f32,
}

pub struct FilterState {
    pub prev_l: f32,
    pub prev_r: f32,
    pub prev_in_l: f32,
    pub prev_in_r: f32,
    pub prev_out_l: f32,
    pub prev_out_r: f32,
}

pub struct NoteSetup {
    pub sample_rate: f32,
    pub channels: usize,
    pub total_samples: usize,
    pub start_sample: usize,
    pub attack_samples: usize,
    pub decay_samples: usize,
    pub release_samples: usize,
    pub sustain_level: f32,
    pub pluck_click: f32,
    pub pluck_click_samples: usize,
    pub drive: f32,
    pub filters: Vec<FilterSpec>,
    pub filter_states: Vec<FilterState>,
    pub lfo_rate: f32,
    pub lfo_depth: f32,
    pub lfo_target: Option<String>,
    pub voices: usize,
    pub unison_detune: f32,
    pub volume_env: HashMap<String, Value>,
    pub pan_env: HashMap<String, Value>,
    pub pitch_env: HashMap<String, Value>,
}

pub fn build_note_setup(
    engine: &mut crate::core::audio::engine::AudioEngine,
    _waveform: &str,
    freq: f32,
    amp: f32,
    start_time_ms: f32,
    mut duration_ms: f32,
    synth_params: &HashMap<String, Value>,
    note_params: &HashMap<String, Value>,
    automation: &Option<HashMap<String, Value>>,
) -> NoteSetup {
    use crate::core::audio::engine::helpers;

    // Extract ADSR and normalize units
    let attack = engine.extract_f32(synth_params, "attack").unwrap_or(0.0);
    let decay = engine.extract_f32(synth_params, "decay").unwrap_or(0.0);
    let sustain = engine.extract_f32(synth_params, "sustain").unwrap_or(1.0);
    let release = engine.extract_f32(synth_params, "release").unwrap_or(0.0);
    let _sustain_level = if sustain > 1.0 {
        (sustain / 100.0).clamp(0.0, 1.0)
    } else {
        sustain.clamp(0.0, 1.0)
    };

    if let Some(g) = engine.extract_f32(note_params, "gate") {
        if g > 0.0 && g <= 1.0 {
            duration_ms = duration_ms * g;
        } else if g > 1.0 {
            duration_ms = duration_ms * (g / 100.0);
        }
    } else if let Some(g) = engine.extract_f32(synth_params, "gate") {
        if g > 0.0 && g <= 1.0 {
            duration_ms = duration_ms * g;
        } else if g > 1.0 {
            duration_ms = duration_ms * (g / 100.0);
        }
    }

    let velocity = engine.extract_f32(note_params, "velocity").unwrap_or(1.0);

    let detune_cents = engine
        .extract_f32(note_params, "detune")
        .or(engine.extract_f32(synth_params, "detune"))
        .unwrap_or(0.0);

    let _lowpass_cut = engine
        .extract_f32(note_params, "lowpass")
        .or(engine.extract_f32(synth_params, "lowpass"))
        .unwrap_or(0.0);

    let _amplitude = (i16::MAX as f32) * amp.clamp(0.0, 1.0) * velocity.clamp(0.0, 1.0);

    let _freq_start = freq;
    let mut _freq_end = freq;
    let _amp_start = amp * velocity.clamp(0.0, 1.0);
    let mut _amp_end = _amp_start;

    let glide = engine
        .extract_boolean(note_params, "glide")
        .unwrap_or(false);
    let slide = engine
        .extract_boolean(note_params, "slide")
        .unwrap_or(false);
    if glide {
        if let Some(Value::Number(target_freq)) = note_params.get("target_freq") {
            _freq_end = *target_freq;
        } else {
            _freq_end = freq * 1.5;
        }
    }
    if slide {
        if let Some(Value::Number(target_amp)) = note_params.get("target_amp") {
            _amp_end = *target_amp * velocity.clamp(0.0, 1.0);
        } else {
            _amp_end = _amp_start * 0.5;
        }
    }

    let sample_rate = engine.sample_rate as f32;
    let channels = engine.channels as usize;

    let total_samples = ((duration_ms / 1000.0) * sample_rate) as usize;
    let start_sample = ((start_time_ms / 1000.0) * sample_rate) as usize;

    // MIDI event
    let midi_note_f = 69.0 + 12.0 * (_freq_start / 440.0).log2();
    let midi_note = midi_note_f.round().clamp(0.0, 127.0) as u8;
    let midi_vel = (velocity.clamp(0.0, 1.0) * 127.0).round().clamp(0.0, 127.0) as u8;
    engine
        .midi_events
        .push(crate::core::audio::engine::driver::MidiNoteEvent {
            key: midi_note,
            vel: midi_vel,
            start_ms: start_time_ms as u32,
            duration_ms: duration_ms as u32,
            channel: 0,
        });

    let _detune_factor = (2.0_f32).powf(detune_cents / 1200.0);

    let (_volume_env, _pan_env, _pitch_env) = helpers::env_maps_from_automation(automation);

    let attack_s = if attack > 10.0 {
        attack / 1000.0
    } else {
        attack
    };
    let decay_s = if decay > 10.0 { decay / 1000.0 } else { decay };
    let release_s = if release > 10.0 {
        release / 1000.0
    } else {
        release
    };
    let sustain_level = _sustain_level;

    let attack_samples = (attack_s * sample_rate) as usize;
    let decay_samples = (decay_s * sample_rate) as usize;
    let release_samples = (release_s * sample_rate) as usize;

    // optional pluck click
    let pluck_click = engine
        .extract_f32(note_params, "pluck_click")
        .or(engine.extract_f32(synth_params, "pluck_click"))
        .unwrap_or(0.0);
    let pluck_click_ms = engine
        .extract_f32(note_params, "pluck_click_ms")
        .or(engine.extract_f32(synth_params, "pluck_click_ms"))
        .unwrap_or(10.0);
    let pluck_click_samples = ((pluck_click_ms / 1000.0) * sample_rate) as usize;

    let drive = engine
        .extract_f32(note_params, "drive")
        .or(engine.extract_f32(synth_params, "drive"))
        .unwrap_or(0.0);

    // parse filter specs
    let mut raw_filters: Vec<HashMap<String, Value>> = Vec::new();
    if let Some(Value::Array(arr)) = synth_params.get("filters") {
        for v in arr {
            if let Value::Map(m) = v {
                raw_filters.push(m.clone());
            }
        }
    }
    if let Some(Value::Array(arr)) = note_params.get("filters") {
        for v in arr {
            if let Value::Map(m) = v {
                raw_filters.push(m.clone());
            }
        }
    }

    let mut filters: Vec<FilterSpec> = Vec::new();
    let mut filter_states: Vec<FilterState> = Vec::new();
    for rf in raw_filters.into_iter() {
        let kind = rf
            .get("type")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                Value::Identifier(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "lowpass".to_string());
        let cutoff = rf
            .get("cutoff")
            .and_then(|v| match v {
                Value::Number(n) => Some(*n),
                Value::String(s) => s.parse::<f32>().ok(),
                _ => None,
            })
            .unwrap_or(1000.0);
        filters.push(FilterSpec {
            kind: kind.to_lowercase(),
            cutoff,
        });
        filter_states.push(FilterState {
            prev_l: 0.0,
            prev_r: 0.0,
            prev_in_l: 0.0,
            prev_in_r: 0.0,
            prev_out_l: 0.0,
            prev_out_r: 0.0,
        });
    }

    // LFO parsing (from synth or note) - simplified: prefer note params over synth params
    let mut lfo_rate = 0.0f32;
    let mut lfo_depth = 0.0f32;
    let mut lfo_target: Option<String> = None;
    if let Some(Value::Map(m)) = synth_params.get("lfo") {
        if let Some(Value::Number(r)) = m.get("rate") {
            lfo_rate = *r;
        }
        if let Some(Value::Number(d)) = m.get("depth") {
            lfo_depth = *d;
        }
        if let Some(Value::String(t)) = m.get("target") {
            lfo_target = Some(t.clone());
        }
    }
    if let Some(Value::Map(m)) = note_params.get("lfo") {
        if let Some(Value::Number(r)) = m.get("rate") {
            lfo_rate = *r;
        }
        if let Some(Value::Number(d)) = m.get("depth") {
            lfo_depth = *d;
        }
        if let Some(Value::String(t)) = m.get("target") {
            lfo_target = Some(t.clone());
        }
    }

    let voices = engine
        .extract_f32(note_params, "voices")
        .or(engine.extract_f32(synth_params, "voices"))
        .unwrap_or(1.0)
        .max(1.0)
        .round() as usize;
    let unison_detune = engine
        .extract_f32(note_params, "unison_detune")
        .or(engine.extract_f32(synth_params, "unison_detune"))
        .unwrap_or(0.0);

    let (volume_env, pan_env, pitch_env) = (
        helpers::env_map_to_hash(&_volume_env),
        helpers::env_map_to_hash(&_pan_env),
        helpers::env_map_to_hash(&_pitch_env),
    );

    NoteSetup {
        sample_rate,
        channels,
        total_samples,
        start_sample,
        attack_samples,
        decay_samples,
        release_samples,
        sustain_level,
        pluck_click,
        pluck_click_samples,
        drive,
        filters,
        filter_states,
        lfo_rate,
        lfo_depth,
        lfo_target,
        voices,
        unison_detune,
        volume_env,
        pan_env,
        pitch_env,
    }
}
