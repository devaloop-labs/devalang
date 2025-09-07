use crate::core::{audio::engine::AudioEngine, plugin::runner::WasmPluginRunner};
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};
use std::collections::HashMap;

pub fn interprete_note_method(
    args: &Vec<devalang_types::Value>,
    target: &str,
    audio_engine: &mut AudioEngine,
    variable_table: &devalang_types::VariableTable,
    global_store: &crate::core::store::global::GlobalStore,
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
    let filtered_args: Vec<_> = args
        .iter()
        .filter(|arg| !matches!(arg, Value::Unknown))
        .collect();

    let Some(Value::Identifier(note_name)) = filtered_args.first().map(|v| (*v).clone()) else {
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
                    for (i, sample) in fbuf.iter().enumerate().take(total_samples * channels) {
                        let s = (sample.clamp(-1.0, 1.0) * (i16::MAX as f32)) as i16;
                        let idx = start_index + i;
                        audio_engine.buffer[idx] = audio_engine.buffer[idx].saturating_add(s);
                    }
                } else {
                    let logger = Logger::new();
                    logger.log_message(
                        LogLevel::Warning,
                        &format!(
                            "Plugin bytes not found for key '{}' (alias '{}').",
                            key, alias
                        ),
                    );
                }
            } else {
                let logger = Logger::new();
                logger.log_message(
                    LogLevel::Warning,
                    &format!("Invalid plugin URI in alias '{}': {}", alias, uri),
                );
            }
        } else {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Warning,
                &format!("Plugin alias '{}' not found in variable table.", alias),
            );
        }
    } else {
        // Allow types to adjust per-note scheduling/params
        let start_ms = start_time * 1000.0;
        let mut final_amp = amp_note;
        let mut final_note_params = note_params.clone();

        let mut handled = false;
        if let Some(tval) = synth_params.get("type") {
            let tname = match tval {
                Value::String(s) => s.as_str(),
                Value::Identifier(s) => s.as_str(),
                _ => "",
            };
            match tname {
                "arp" => {
                    // compute a step (ms) from synth params (rate/step). compute_arp_step
                    // will interpret `rate` as number of notes across the provided duration
                    let step_ms = crate::core::audio::interpreter::statements::arrow_call::types::arp::
                        compute_arp_step(duration_ms, 1, &synth_params);
                    let steps = if step_ms > 0.0 {
                        ((duration_ms / step_ms).ceil() as usize).max(1)
                    } else {
                        1usize
                    };

                    // For each arp step, call prepare_note to get per-step params and schedule it
                    for idx in 0..steps {
                        let (start_abs_ms, freq_step, amp_out, params_out) =
                            crate::core::audio::interpreter::statements::arrow_call::types::arp::prepare_note(
                                &note_name,
                                idx,
                                steps,
                                start_ms,
                                duration_ms,
                                amp_note,
                                &synth_params,
                                &final_note_params,
                                &automation,
                            );

                        // sub-note duration: default to step_ms so arp steps are audible and sequenced
                        let sub_duration_ms = if step_ms > 0.0 { step_ms } else { duration_ms };

                        audio_engine.insert_note(
                            waveform_str.to_string(),
                            freq_step,
                            amp_out,
                            start_abs_ms,
                            sub_duration_ms,
                            synth_params.clone(),
                            params_out.clone(),
                            automation.clone(),
                        );
                    }

                    // mark handled to avoid the unconditional insert below
                    handled = true;
                }
                "pluck" => {
                    let (_s, _f, amp_out, params_out) =
                        crate::core::audio::interpreter::statements::arrow_call::types::pluck::prepare_note(
                            &note_name,
                            0,
                            1,
                            start_ms,
                            duration_ms,
                            amp_note,
                            &synth_params,
                            &final_note_params,
                            &automation,
                        );
                    final_amp = amp_out;
                    final_note_params = params_out;
                }
                "pad" => {
                    let (_s, _f, amp_out, params_out) =
                        crate::core::audio::interpreter::statements::arrow_call::types::pad::prepare_note(
                            &note_name,
                            0,
                            1,
                            start_ms,
                            duration_ms,
                            amp_note,
                            &synth_params,
                            &final_note_params,
                            &automation,
                        );
                    final_amp = amp_out;
                    final_note_params = params_out;
                }
                _ => {}
            }
        }

        if !handled {
            audio_engine.insert_note(
                waveform_str.to_string(),
                final_freq,
                final_amp,
                start_ms,
                duration_ms,
                synth_params.clone(),
                final_note_params,
                automation,
            );
        }
    }

    *max_end_time = (*max_end_time).max(end_time);

    if update_cursor {
        if let Some(c) = cursor_time.as_mut() {
            **c = end_time;
        }
    }

    return (*max_end_time, end_time);
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
