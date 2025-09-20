use crate::core::audio::engine::AudioEngine;
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};
use std::collections::HashMap;

pub fn interprete_chord_method(
    args: &Vec<devalang_types::Value>,
    target: &str,
    audio_engine: &mut AudioEngine,
    variable_table: &devalang_types::VariableTable,
    _global_store: &crate::core::store::global::GlobalStore,
    waveform_str: &str,
    synth_params: &HashMap<String, devalang_types::Value>,
    amp: f32,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: &mut f32,
    mut cursor_time: Option<&mut f32>,
    cursor_copy: f32,
    update_cursor: bool,
) -> (f32, f32) {
    // Filter unknown args
    let filtered_args: Vec<_> = args
        .iter()
        .filter(|arg| !matches!(arg, Value::Unknown))
        .collect();

    // Expect at least one note identifier, up to 4 notes, optionally last arg is a map of params
    if filtered_args.is_empty() {
        let logger = Logger::new();
        logger.log_message(
            LogLevel::Error,
            &format!("Invalid or missing arguments for 'chord' on '{}'.", target),
        );
        return (*max_end_time, cursor_copy);
    }

    // Collect note names (first N args that are Identifier or String)
    let mut note_names: Vec<String> = Vec::new();
    let mut note_params: HashMap<String, Value> = HashMap::new();

    for (_i, arg) in filtered_args.iter().enumerate().take(5) {
        match (*arg).clone() {
            Value::Identifier(s) | Value::String(s) => {
                if note_names.len() < 4 {
                    note_names.push(s);
                }
            }
            Value::Map(m) => {
                // treat as chord-level params (duration, glide, etc.)
                for (k, v) in m {
                    note_params.insert(k, v);
                }
            }
            _ => {}
        }
    }

    if note_names.is_empty() {
        let logger = Logger::new();
        logger.log_message(
            LogLevel::Error,
            &format!("No valid notes found for 'chord' on '{}'.", target),
        );
        return (*max_end_time, cursor_copy);
    }

    // duration & amp for chord
    let amp_note = extract_f32(&note_params, "amp", base_bpm).unwrap_or(amp);
    let duration_ms =
        extract_f32(&note_params, "duration", base_bpm).unwrap_or(base_duration * 1000.0);
    let duration_secs = duration_ms / 1000.0;

    let start_time = cursor_copy;
    let end_time = start_time + duration_secs;

    // Expand shorthand chord notation like C#min, Dmaj, Amin7 into individual note names
    note_names = expand_chord_shorthands(note_names);

    // Prepare automation merge similar to note method
    let auto_key = format!("{}__automation", target);
    let synth_automation = match variable_table.get(&auto_key) {
        Some(Value::Map(map)) => match map.get("params") {
            Some(Value::Map(p)) => Some(p.clone()),
            _ => None,
        },
        _ => None,
    };

    let chord_automation = match note_params.get("automate") {
        Some(Value::Map(m)) => Some(m.clone()),
        _ => None,
    };

    let automation = match (synth_automation, chord_automation) {
        (Some(mut a), Some(n)) => {
            for (k, v) in n {
                a.insert(k, v);
            }
            Some(a)
        }
        (None, Some(n)) => Some(n),
        (Some(a), None) => Some(a),
        _ => None,
    };

    // For each note, let the selected type compute per-note scheduling and parameters
    for (i, note_name) in note_names.iter().enumerate() {
        // default: start at chord start
        let start_ms_default = start_time * 1000.0;
        let mut note_amp = amp_note;
        let mut note_params_clone = note_params.clone();
        let mut note_start_ms = start_ms_default;
        let mut note_freq = note_to_freq(note_name);

        if let Some(tval) = synth_params.get("type") {
            let tname = match tval {
                Value::String(s) => s.as_str(),
                Value::Identifier(s) => s.as_str(),
                _ => "",
            };
            match tname {
                "arp" => {
                    let (s_ms, _f, amp_out, params_out) =
                        crate::core::audio::interpreter::statements::arrow_call::types::arp::prepare_note(
                            note_name,
                            i,
                            note_names.len(),
                            start_ms_default,
                            duration_ms,
                            amp_note,
                            &synth_params,
                            &note_params_clone,
                            &automation,
                        );
                    note_start_ms = s_ms;
                    note_freq = _f;
                    note_amp = amp_out;
                    note_params_clone = params_out;
                }
                "pluck" => {
                    let (s_ms, _f, amp_out, params_out) =
                        crate::core::audio::interpreter::statements::arrow_call::types::pluck::prepare_note(
                            note_name,
                            i,
                            note_names.len(),
                            start_ms_default,
                            duration_ms,
                            amp_note,
                            &synth_params,
                            &note_params_clone,
                            &automation,
                        );
                    // pluck.prepare_note returns start offset (s_ms) relative to default; if zero use default
                    if s_ms > 0.0 {
                        note_start_ms = s_ms
                    }
                    note_amp = amp_out;
                    note_params_clone = params_out;
                }
                "pad" => {
                    let (s_ms, _f, amp_out, params_out) =
                        crate::core::audio::interpreter::statements::arrow_call::types::pad::prepare_note(
                            note_name,
                            i,
                            note_names.len(),
                            start_ms_default,
                            duration_ms,
                            amp_note,
                            &synth_params,
                            &note_params_clone,
                            &automation,
                        );
                    if s_ms > 0.0 {
                        note_start_ms = s_ms
                    }
                    note_amp = amp_out;
                    note_params_clone = params_out;
                }
                _ => {}
            }
        }

        let ranges = audio_engine.insert_note(
            Some(target.to_string()),
            waveform_str.to_string(),
            note_freq,
            note_amp,
            note_start_ms,
            duration_ms,
            synth_params.clone(),
            note_params_clone.clone(),
            automation.clone(),
        );

        // Apply simple per-note effects if provided under note_params_clone.effects (map)
        if let Some(Value::Map(eff_map)) = note_params_clone.get("effects") {
            for (_start, _len) in ranges.iter() {
                crate::core::audio::interpreter::statements::arrow_call::methods::effects::apply_effect_chain(
                    "echo",
                    &vec![Value::Map(eff_map.clone())],
                    target,
                    audio_engine,
                    variable_table,
                );
            }
        }

        let note_end = (note_start_ms / 1000.0) + (duration_ms / 1000.0);
        *max_end_time = (*max_end_time).max(note_end);
    }

    *max_end_time = (*max_end_time).max(end_time);

    if update_cursor {
        if let Some(c) = cursor_time.as_mut() {
            **c = end_time;
        }
    }

    (*max_end_time, end_time)
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

fn extract_f32(map: &HashMap<String, Value>, key: &str, base_bpm: f32) -> Option<f32> {
    map.get(key).and_then(|v| match v {
        Value::Number(n) => Some(*n),
        Value::Beat(beat_str) => {
            let parts: Vec<&str> = beat_str.split('/').collect();
            if parts.len() == 2 {
                let numerator = parts[0].parse::<f32>().ok()?;
                let denominator = parts[1].parse::<f32>().ok()?;

                Some((numerator / denominator) * ((60.0 / base_bpm) * 1000.0))
            } else {
                None
            }
        }
        _ => None,
    })
}

// Expand shorthand chords into constituent note names (strings with octave if applicable)
fn expand_chord_shorthands(names: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in names.into_iter() {
        if let Some(parts) = parse_chord_shorthand(&name) {
            // parts contains (root_semitone, octave, chord_type)
            let (root_semitone, octave, chord_type) = parts;
            let root_midi = (octave + 1) * 12 + root_semitone as i32;
            match chord_type.as_str() {
                "min" | "m" => {
                    let third = root_midi + 3;
                    let fifth = root_midi + 7;
                    out.push(midi_to_note(root_midi));
                    out.push(midi_to_note(third));
                    out.push(midi_to_note(fifth));
                }
                "7" => {
                    let third = root_midi + 4;
                    let fifth = root_midi + 7;
                    let seventh = root_midi + 10;
                    out.push(midi_to_note(root_midi));
                    out.push(midi_to_note(third));
                    out.push(midi_to_note(fifth));
                    out.push(midi_to_note(seventh));
                }
                _ => {
                    // default to major triad
                    let third = root_midi + 4;
                    let fifth = root_midi + 7;
                    out.push(midi_to_note(root_midi));
                    out.push(midi_to_note(third));
                    out.push(midi_to_note(fifth));
                }
            }
        } else {
            out.push(name);
        }
    }
    out
}

fn parse_chord_shorthand(s: &str) -> Option<(u8, i32, String)> {
    // examples: C#min, Amin, Dmaj7, Ebm
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let mut chars = s.chars().peekable();
    let first = chars.next()?;
    let letter = first.to_ascii_uppercase();
    if !matches!(letter, 'A'..='G') {
        return None;
    }
    let mut name = String::new();
    name.push(letter);
    // accidental
    if let Some(&c) = chars.peek() {
        if c == '#' || c == 'b' {
            name.push(c);
            chars.next();
        }
    }

    let rest: String = chars.collect();
    // extract trailing digits as octave
    let mut octave: i32 = 4; // default
    let mut type_part = rest.as_str();
    // if ends with digit(s)
    if !rest.is_empty() {
        let mut digits = String::new();
        for ch in rest.chars().rev() {
            if ch.is_ascii_digit() {
                digits.insert(0, ch);
            } else {
                break;
            }
        }
        if !digits.is_empty() {
            if let Ok(o) = digits.parse::<i32>() {
                octave = o;
                // remove digits from end
                let split_at = rest.len() - digits.len();
                type_part = &rest[..split_at];
            }
        }
    }

    let chord_type = type_part.to_ascii_lowercase();
    // compute semitone index
    let semitone = match name.as_str() {
        "C" => 0,
        "C#" => 1,
        "DB" => 1,
        "D" => 2,
        "D#" => 3,
        "EB" => 3,
        "E" => 4,
        "F" => 5,
        "F#" => 6,
        "GB" => 6,
        "G" => 7,
        "G#" => 8,
        "AB" => 8,
        "A" => 9,
        "A#" => 10,
        "BB" => 10,
        "B" => 11,
        _ => return None,
    };

    // normalize chord_type: if empty => major
    let chord_type = if chord_type.is_empty() {
        "maj".to_string()
    } else {
        chord_type
    };

    Some((semitone as u8, octave, chord_type))
}

fn midi_to_note(m: i32) -> String {
    let semitone = (m % 12 + 12) % 12;
    let octave = (m / 12) - 1;
    let name = match semitone {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => "C",
    };
    format!("{}{}", name, octave)
}
