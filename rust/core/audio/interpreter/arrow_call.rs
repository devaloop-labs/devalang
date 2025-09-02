use crate::core::{
    audio::engine::AudioEngine,
    parser::statement::{Statement, StatementKind},
    plugin::runner::WasmPluginRunner,
    store::{global::GlobalStore, variable::VariableTable},
};
use devalang_types::Value;
use devalang_utils::logger::{Logger, LogLevel};

use std::collections::HashMap;

pub fn interprete_call_arrow_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    global_store: &GlobalStore,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: &mut f32,
    mut cursor_time: Option<&mut f32>,
    update_cursor: bool,
) -> (f32, f32) {
    let cursor_copy = cursor_time.as_ref().map(|c| **c).unwrap_or(0.0);

    if let StatementKind::ArrowCall {
        target,
        method,
        args,
    } = &stmt.kind
    {
        let Some(Value::Statement(synth_stmt)) = variable_table.get(target) else {
            let logger = Logger::new();
            logger.log_message(LogLevel::Error, &format!("Synth '{}' not found in variable table", target));
            return (*max_end_time, cursor_copy);
        };

        let Value::Map(synth_map) = &synth_stmt.value else {
            let logger = Logger::new();
            logger.log_message(LogLevel::Error, &format!("Invalid synth statement for '{}', expected a map.", target));
            return (*max_end_time, cursor_copy);
        };

        let Some(Value::String(entity)) = synth_map.get("entity") else {
            let logger = Logger::new();
            logger.log_message(LogLevel::Error, &format!("Missing 'entity' key in synth '{}'.", target));
            return (*max_end_time, cursor_copy);
        };

        if entity != "synth" {
            let logger = Logger::new();
            logger.log_message(LogLevel::Error, &format!("'{}' is not a synth, entity is '{}'.", target, entity));
            return (*max_end_time, cursor_copy);
        }

        let Some(Value::Map(value_map)) = synth_map.get("value") else {
            let logger = Logger::new();
            logger.log_message(LogLevel::Error, &format!("Missing 'value' map in synth '{}'.", target));
            return (*max_end_time, cursor_copy);
        };

        let waveform_str = match value_map.get("waveform") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Identifier(s)) => s.clone(),
            _ => {
                let logger = Logger::new();
                logger.log_message(LogLevel::Error, &format!("Missing or invalid 'waveform' in synth '{}'.", target));
                return (*max_end_time, cursor_copy);
            }
        };
        let Some(Value::Map(params)) = value_map.get("parameters") else {
            println!("❌ Missing or invalid 'parameters' in synth '{}'.", target);
            return (*max_end_time, cursor_copy);
        };

        // Synth parameters
        let synth_params = params.clone();
        let amp = extract_f32(&synth_params, "amp", base_bpm).unwrap_or(1.0);

        if method == "note" {
            let filtered_args: Vec<_> = args
                .iter()
                .filter(|arg| !matches!(arg, Value::Unknown))
                .collect();

            let Some(Value::Identifier(note_name)) = filtered_args.first().map(|v| (*v).clone())
            else {
                println!(
                    "❌ Invalid or missing argument for 'note' method on '{}'.",
                    target
                );
                return (*max_end_time, cursor_copy);
            };

            let mut note_params = HashMap::new();
            if let Some(arg1) = filtered_args.get(1) {
                if let Value::Map(map) = (*arg1).clone() {
                    for (key, value) in map {
                        note_params.insert(key, value);
                    }
                }
            }

            // Note parameters and calculations
            let amp_note = extract_f32(&note_params, "amp", base_bpm).unwrap_or(amp);
            let duration_ms =
                extract_f32(&note_params, "duration", base_bpm).unwrap_or(base_duration * 1000.0);

            let duration_secs = duration_ms / 1000.0;
            let final_freq = note_to_freq(&note_name);
            let start_time = cursor_copy;
            let end_time = start_time + duration_secs;

            // Fetch automation map if present:
            // - Global (per-synth): key "<target>__automation" => map with key "params"
            // - Per-note: note parameter "automate" => map
            let auto_key = format!("{}__automation", target);
            let synth_automation = match variable_table.get(&auto_key) {
                Some(Value::Map(map)) => match map.get("params") {
                    Some(Value::Map(p)) => Some(p.clone()),
                    _ => None,
                },
                _ => None,
            };

            let note_automation = match note_params.get("automate") {
                Some(Value::Map(m)) => Some(m.clone()),
                _ => None,
            };

            // Merge: per-note overrides synth automation per key (volume/pan/pitch)
            let automation = match (synth_automation, note_automation) {
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

            // If waveform references a plugin alias (e.g., alias.synth), use the WASM plugin runner
            if waveform_str.contains('.') && waveform_str.ends_with(".synth") {
                let alias = waveform_str.split('.').next().unwrap_or("");
                if let Some(Value::String(uri)) = variable_table.get(alias) {
                    if let Some(id) = uri.strip_prefix("devalang://plugin/") {
                        let mut parts = id.split('.');
                        let author = parts.next().unwrap_or("");
                        let name = parts.next().unwrap_or("");
                        let key = format!("{}:{}", author, name);
                        if let Some((_info, wasm_bytes)) = global_store.plugins.get(&key) {
                            // Prepare buffer (stereo f32)
                            let sample_rate = 44100.0_f32;
                            let total_samples = ((duration_ms / 1000.0) * sample_rate) as usize;
                            let channels = 2usize;
                            let start_index = ((start_time * sample_rate) as usize) * channels;
                            let required_len = start_index + total_samples * channels;
                            if audio_engine.buffer.len() < required_len {
                                audio_engine.buffer.resize(required_len, 0);
                            }
                            let mut fbuf = vec![0.0f32; total_samples * channels];
                            let runner = WasmPluginRunner::new();
                            let mut params_num: std::collections::HashMap<String, f32> =
                                std::collections::HashMap::new();
                            let mut params_str: std::collections::HashMap<String, String> =
                                std::collections::HashMap::new();
                            for (k, v) in synth_params.iter() {
                                match v {
                                    Value::Number(n) => {
                                        params_num.insert(k.clone(), *n);
                                    }
                                    Value::String(s) => {
                                        params_str.insert(k.clone(), s.clone());
                                    }
                                    Value::Identifier(s) => {
                                        params_str.insert(k.clone(), s.clone());
                                    }
                                    _ => {}
                                }
                            }
                            let _ = runner.render_note_with_params_in_place(
                                wasm_bytes,
                                &mut fbuf,
                                None,
                                final_freq,
                                amp_note,
                                duration_ms as i32,
                                44100,
                                2,
                                &params_num,
                                Some(&params_str),
                            );
                            for (i, sample) in
                                fbuf.iter().enumerate().take(total_samples * channels)
                            {
                                let s = (sample.clamp(-1.0, 1.0) * (i16::MAX as f32)) as i16;
                                let idx = start_index + i;
                                audio_engine.buffer[idx] =
                                    audio_engine.buffer[idx].saturating_add(s);
                            }
                        } else {
                            let logger = Logger::new();
                            logger.log_message(LogLevel::Warning, &format!("Plugin bytes not found for key '{}' (alias '{}').", key, alias));
                        }
                    } else {
                        let logger = Logger::new();
                        logger.log_message(LogLevel::Warning, &format!("Invalid plugin URI in alias '{}': {}", alias, uri));
                    }
                } else {
                    let logger = Logger::new();
                    logger.log_message(LogLevel::Warning, &format!("Plugin alias '{}' not found in variable table.", alias));
                }
            } else {
                audio_engine.insert_note(
                    waveform_str.clone(),
                    final_freq,
                    amp_note,
                    start_time * 1000.0,
                    duration_ms,
                    synth_params,
                    note_params,
                    automation,
                );
            }

            *max_end_time = (*max_end_time).max(end_time);

            if update_cursor {
                if let Some(c) = cursor_time.as_mut() {
                    **c = end_time;
                }
            }

            return (*max_end_time, end_time);
        } else {
            let logger = Logger::new();
            logger.log_message(LogLevel::Error, &format!("Unknown method '{}' on synth '{}'.", method, target));
        }
    }

    (*max_end_time, cursor_copy)
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
