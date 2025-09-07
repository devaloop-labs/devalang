use devalang_types::Value;
use std::collections::HashMap;

pub fn apply_defaults(synth_params: &mut HashMap<String, Value>) {
    use devalang_types::Value as DV;
    // pad: long attack/release, high sustain (0.0-1.0)
    synth_params
        .entry("attack".to_string())
        .or_insert(DV::Number(600.0));
    synth_params
        .entry("decay".to_string())
        .or_insert(DV::Number(300.0));
    synth_params
        .entry("sustain".to_string())
        .or_insert(DV::Number(0.8));
    synth_params
        .entry("release".to_string())
        .or_insert(DV::Number(900.0));
    // optional unison/voices parameters
    synth_params
        .entry("voices".to_string())
        .or_insert(DV::Number(3.0));
    synth_params
        .entry("unison_detune".to_string())
        .or_insert(DV::Number(15.0));
    // slightly wider default chorus/depth for pads
    if !synth_params.contains_key("effects") {
        let mut ch: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
        ch.insert("type".to_string(), Value::String("chorus".to_string()));
        ch.insert("depth".to_string(), Value::Number(0.12));
        ch.insert("rate".to_string(), Value::Number(0.25));
        synth_params.insert("effects".to_string(), DV::Array(vec![DV::Map(ch)]));
    }
}

// Prepare per-note modifications for pad: longer release/attack, gentle detune per-voice
pub fn prepare_note(
    _note_name: &str,
    index: usize,
    total: usize,
    _start_time_ms: f32,
    _duration_ms: f32,
    amp: f32,
    synth_params: &HashMap<String, Value>,
    note_params: &HashMap<String, Value>,
    _automation: &Option<HashMap<String, Value>>,
) -> (f32, f32, f32, HashMap<String, Value>) {
    let mut params_out = note_params.clone();
    // inject pad envelope defaults if missing
    params_out
        .entry("attack".to_string())
        .or_insert(Value::Number(600.0));
    params_out
        .entry("release".to_string())
        .or_insert(Value::Number(900.0));
    params_out
        .entry("sustain".to_string())
        .or_insert(Value::Number(0.8));

    // propagate unison/voices into per-note params if synth provided
    if let Some(Value::Number(v)) = synth_params.get("voices") {
        params_out.insert("voices".to_string(), Value::Number(*v));
    }
    if let Some(Value::Number(d)) = synth_params.get("unison_detune") {
        params_out.insert("unison_detune".to_string(), Value::Number(*d));
    }

    // gentle detune based on index and synth param "detune" if present
    // gentle detune based on synth detune or unison settings
    let detune_base = match synth_params.get("detune") {
        Some(Value::Number(n)) => *n,
        _ => 0.0,
    };
    let mut detune = detune_base * ((index as f32) / (total as f32));
    // if unison used, spread voices around center
    if let Some(Value::Number(_voices)) = synth_params.get("voices") {
        if let Some(Value::Number(ud)) = synth_params.get("unison_detune") {
            // spread by a fraction based on voice index
            detune += (*ud) * (((index as f32) - (total as f32 - 1.0) / 2.0) / (total as f32));
        }
    }

    params_out.insert("detune".to_string(), Value::Number(detune));

    // compute frequency with detune in cents
    let freq = note_to_freq(_note_name) * (2.0_f32).powf(detune / 1200.0);

    // add a subtle chorus placeholder effect if none present (engine may ignore unknown effects)
    use devalang_types::Value as DV;
    if !params_out.contains_key("effects") {
        let mut ch: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
        ch.insert("type".to_string(), Value::String("chorus".to_string()));
        ch.insert("depth".to_string(), Value::Number(0.15));
        ch.insert("rate".to_string(), Value::Number(0.25));
        params_out.insert("effects".to_string(), DV::Array(vec![DV::Map(ch)]));
    }

    (0.0, freq, amp, params_out)
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
