use devalang_types::Value;
use std::collections::HashMap;

pub fn apply_defaults(synth_params: &mut HashMap<String, Value>) {
    use devalang_types::Value as DV;
    // pluck: very short attack, short decay, very low sustain, short release
    synth_params
        .entry("attack".to_string())
        .or_insert(DV::Number(2.0));
    synth_params
        .entry("decay".to_string())
        .or_insert(DV::Number(80.0));
    synth_params
        .entry("sustain".to_string())
        .or_insert(DV::Number(0.0));
    synth_params
        .entry("release".to_string())
        .or_insert(DV::Number(120.0));
    synth_params
        .entry("glide".to_string())
        .or_insert(DV::Boolean(false));
    synth_params
        .entry("slide".to_string())
        .or_insert(DV::Boolean(false));
    // small transient helper for plucks
    synth_params
        .entry("pluck_click".to_string())
        .or_insert(DV::Number(0.08));
    synth_params
        .entry("pluck_click_ms".to_string())
        .or_insert(DV::Number(8.0));
}

// Prepare per-note modifications for pluck: short attack/release, optional amp curve per-index
pub fn prepare_note(
    _note_name: &str,
    index: usize,
    _total: usize,
    _start_time_ms: f32,
    _duration_ms: f32,
    amp: f32,
    _synth_params: &HashMap<String, Value>,
    note_params: &HashMap<String, Value>,
    _automation: &Option<HashMap<String, Value>>,
) -> (f32, f32, f32, HashMap<String, Value>) {
    // pluck slightly reduces amp for later notes by a tiny fraction
    let decay = 0.02 * (index as f32);
    let amp_out = (amp * (1.0 - decay)).max(0.0);
    let mut params_out = note_params.clone();
    // ensure short attack & short release to sound plucky
    params_out
        .entry("attack".to_string())
        .or_insert(Value::Number(1.0));
    params_out
        .entry("release".to_string())
        .or_insert(Value::Number(100.0));
    params_out
        .entry("sustain".to_string())
        .or_insert(Value::Number(0.0));
    // add a small "click" transient and a light resonant filter to emphasize the pluck
    use devalang_types::Value as DV;
    let freq = note_to_freq(_note_name);
    let mut filt: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    filt.insert("type".to_string(), Value::String("bandpass".to_string()));
    // center filter a bit above the note to give a bright pluck transient
    // use more explicit filter parameter names requested by user
    filt.insert(
        "cutoff".to_string(),
        Value::Number((freq * 2.0).min(10000.0)),
    );
    filt.insert("resonance".to_string(), Value::Number(6.0));
    // only insert filters if not already present
    if !params_out.contains_key("filters") {
        params_out.insert("filters".to_string(), DV::Array(vec![DV::Map(filt)]));
    }
    // frequency unchanged here; caller computes freq
    // stronger pitch drop for a plucky character
    let freq_end = freq * 0.98;
    (0.0, freq_end, amp_out, params_out)
}

fn note_to_freq(note: &str) -> f32 {
    let notes = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];

    if note.len() < 2 || note.len() > 3 {
        return 440.0;
    }

    let (name, octave_str) = note.split_at(note.len() - 1);
    let semitone = notes.iter().position(|&n| n == name).unwrap_or(9) as i32;
    let octave = octave_str.parse::<i32>().unwrap_or(4);
    let midi_note = (octave + 1) * 12 + semitone;

    440.0 * (2.0_f32).powf(((midi_note as f32) - 69.0) / 12.0)
}
