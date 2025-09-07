use devalang_types::Value;
use std::collections::HashMap;

pub fn apply_defaults(synth_params: &mut HashMap<String, Value>) {
    use devalang_types::Value as DV;
    // sub bass: strong sustain, lowpass emphasis
    synth_params
        .entry("attack".to_string())
        .or_insert(DV::Number(10.0));
    synth_params
        .entry("decay".to_string())
        .or_insert(DV::Number(80.0));
    // sustain stored as 0.0-1.0 for consistency
    synth_params
        .entry("sustain".to_string())
        .or_insert(DV::Number(1.0));
    synth_params
        .entry("release".to_string())
        .or_insert(DV::Number(300.0));
    synth_params
        .entry("detune".to_string())
        .or_insert(DV::Number(0.0));
    synth_params
        .entry("lowpass".to_string())
        .or_insert(DV::Number(150.0));
    // octave stacking and drive for extra body
    synth_params
        .entry("octaves".to_string())
        .or_insert(DV::Number(1.0));
    synth_params
        .entry("drive".to_string())
        .or_insert(DV::Number(0.0));
    // encourage a lowpass default if none provided
    synth_params
        .entry("lowpass".to_string())
        .or_insert(DV::Number(120.0));
}

// prepare_note for sub: shift note one or two octaves down and set stronger amp
pub fn prepare_note(
    note_name: &str,
    _index: usize,
    _total: usize,
    _start_time_ms: f32,
    _duration_ms: f32,
    amp: f32,
    _synth_params: &HashMap<String, Value>,
    note_params: &HashMap<String, Value>,
    _automation: &Option<HashMap<String, Value>>,
) -> (f32, f32, f32, HashMap<String, Value>) {
    let mut params_out = note_params.clone();
    params_out
        .entry("attack".to_string())
        .or_insert(Value::Number(5.0));
    params_out
        .entry("release".to_string())
        .or_insert(Value::Number(200.0));
    params_out
        .entry("sustain".to_string())
        .or_insert(Value::Number(1.0));

    // compute deep-frequency one octave down by default
    let freq = note_to_freq(note_name) * 0.5;
    // allow stacking additional octaves for extra body
    if let Some(Value::Number(o)) = _synth_params.get("octaves") {
        let oct = (*o as i32).max(1);
        if oct >= 2 {
            // add lower harmonic by halving frequency again (engine may layer if octaves param is observed)
            // we still return the primary frequency but advertise octaves in params
            params_out.insert("octaves".to_string(), Value::Number(*o));
        }
    }

    // boost amplitude a little for sub
    let amp_out = (amp * 1.5).min(1.0);

    // advertise drive if present so engine/effects can apply soft clipping
    if let Some(Value::Number(d)) = _synth_params.get("drive") {
        params_out.insert("drive".to_string(), Value::Number(*d));
    }

    (0.0, freq, amp_out, params_out)
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
