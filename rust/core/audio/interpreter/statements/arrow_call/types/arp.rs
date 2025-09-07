use devalang_types::Value;
use std::collections::HashMap;

pub fn apply_defaults(synth_params: &mut HashMap<String, Value>) {
    use devalang_types::Value as DV;
    synth_params
        .entry("attack".to_string())
        .or_insert(DV::Number(1.0));
    synth_params
        .entry("decay".to_string())
        .or_insert(DV::Number(50.0));
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
    // arp defaults
    synth_params
        .entry("gate".to_string())
        .or_insert(DV::Number(80.0));
    synth_params
        .entry("pattern".to_string())
        .or_insert(DV::String("up".to_string()));
    // per-step velocity shaping
    synth_params
        .entry("velocity_scale".to_string())
        .or_insert(DV::Number(0.25));
}

// Helper to compute an arp step and schedule notes; this is small and returns the computed step
pub fn compute_arp_step(
    duration_ms: f32,
    note_count: usize,
    synth_params: &HashMap<String, Value>,
) -> f32 {
    // New API: synth_params may contain "rate" (beats/duration string or number),
    // "pattern" (up, down, updown), and "step" (ms or number)
    use devalang_types::Duration as D;

    // If rate is provided as number (notes across duration), use it
    if let Some(Value::Number(rate)) = synth_params.get("rate") {
        if *rate > 0.0 {
            return duration_ms / *rate;
        }
    }

    // If rate provided as Duration/Beat/String parse as musical fraction -> seconds
    if let Some(v) = synth_params.get("rate") {
        match v {
            Value::Duration(d) => match d {
                D::Number(sec) => {
                    if *sec > 0.0 {
                        return sec * 1000.0;
                    }
                }
                D::Beat(_) | D::Identifier(_) => {
                    // need bpm; fallback 120
                    let bpm = synth_params
                        .get("bpm")
                        .and_then(|bv| {
                            if let Value::Number(n) = bv {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(120.0);
                    if let Some(sec) =
                        crate::core::audio::engine::driver::duration_to_seconds(d, bpm)
                    {
                        if sec > 0.0 {
                            return sec * 1000.0;
                        }
                    }
                }
                _ => {}
            },
            Value::String(s) | Value::Identifier(s) => {
                let bpm = synth_params
                    .get("bpm")
                    .and_then(|bv| {
                        if let Value::Number(n) = bv {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(120.0);
                if let Some(sec) =
                    crate::core::audio::engine::driver::parse_fraction_to_seconds(s, bpm)
                {
                    if sec > 0.0 {
                        return sec * 1000.0;
                    }
                }
                if let Ok(n) = s.parse::<f32>() {
                    if n > 0.0 {
                        return duration_ms / n;
                    }
                }
            }
            _ => {}
        }
    }

    // fallback to explicit step param
    let default_step = (duration_ms / (note_count as f32)) * 0.5;
    match synth_params.get("step") {
        Some(Value::Number(n)) => *n,
        Some(Value::String(s)) => s.parse::<f32>().unwrap_or(default_step),
        Some(Value::Identifier(s)) => s.parse::<f32>().unwrap_or(default_step),
        _ => default_step,
    }
}

// Ensure arp defaults include an arp_step if provided as beats / number string
pub fn ensure_arp_step(synth_params: &mut HashMap<String, Value>, default_step: f32) {
    use devalang_types::Value as DV;
    if !synth_params.contains_key("arp_step") {
        synth_params.insert("arp_step".to_string(), DV::Number(default_step));
    }
}

// Prepare per-note scheduling & params for arp
pub fn prepare_note(
    note_name: &str,
    index: usize,
    total: usize,
    start_time_ms: f32,
    duration_ms: f32,
    amp: f32,
    synth_params: &HashMap<String, Value>,
    note_params: &HashMap<String, Value>,
    _automation: &Option<HashMap<String, Value>>,
) -> (f32, f32, f32, HashMap<String, Value>) {
    // compute a slightly syncopated step: half-step + small swing
    let mut step = compute_arp_step(duration_ms, total, synth_params);
    // introduce simple swing based on index
    let swing = match synth_params.get("swing") {
        Some(Value::Number(n)) => *n,
        _ => 0.05,
    };
    if index % 2 == 1 {
        step *= 1.0 + swing;
    }
    let start = start_time_ms + (index as f32) * step;
    // For now, arp doesn't alter amp or params beyond start offset
    let amp_out = amp;
    let mut params_out = note_params.clone();
    // optionally expose arp-specific params into note params
    if let Some(Value::Number(n)) = synth_params.get("gate") {
        // normalize gate to 0.0-1.0 if >1 (percent)
        let gate_val = if *n > 1.0 { (*n) / 100.0 } else { *n };
        params_out
            .entry("gate".to_string())
            .or_insert(Value::Number(gate_val));
    }
    // simple velocity curve across arp steps (so later steps can be quieter/louder)
    if let Some(Value::Number(vscale)) = synth_params.get("velocity_scale") {
        let mult = 1.0 - ((index as f32) / (total as f32)) * (*vscale);
        params_out.insert("velocity".to_string(), Value::Number(mult.max(0.0)));
    }
    // propagate pattern if present (engine or higher layer may use it)
    if let Some(Value::String(p)) = synth_params.get("pattern") {
        params_out.insert("pattern".to_string(), Value::String(p.clone()));
    }
    (start, note_to_freq(note_name), amp_out, params_out)
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
