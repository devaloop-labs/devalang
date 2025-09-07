use crate::core::{
    audio::{
        engine::AudioEngine,
        interpreter::statements::arrow_call::methods::{
            chord::interprete_chord_method, note::interprete_note_method,
        },
    },
    parser::statement::{Statement, StatementKind},
    store::global::GlobalStore,
};
use devalang_types::{Value, VariableTable};
use devalang_utils::logger::{LogLevel, Logger};

use std::collections::HashMap;

pub fn interprete_arrow_call_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    global_store: &GlobalStore,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: &mut f32,
    cursor_time: Option<&mut f32>,
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
            logger.log_message(
                LogLevel::Error,
                &format!("Synth '{}' not found in variable table", target),
            );
            return (*max_end_time, cursor_copy);
        };

        let Value::Map(synth_map) = &synth_stmt.value else {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Error,
                &format!("Invalid synth statement for '{}', expected a map.", target),
            );
            return (*max_end_time, cursor_copy);
        };

        let Some(Value::String(entity)) = synth_map.get("entity") else {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Error,
                &format!("Missing 'entity' key in synth '{}'.", target),
            );
            return (*max_end_time, cursor_copy);
        };

        if entity != "synth" {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Error,
                &format!("'{}' is not a synth, entity is '{}'.", target, entity),
            );
            return (*max_end_time, cursor_copy);
        }

        let Some(Value::Map(value_map)) = synth_map.get("value") else {
            let logger = Logger::new();
            logger.log_message(
                LogLevel::Error,
                &format!("Missing 'value' map in synth '{}'.", target),
            );
            return (*max_end_time, cursor_copy);
        };

        let waveform_str = match value_map.get("waveform") {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Identifier(s)) => s.clone(),
            _ => {
                let logger = Logger::new();
                logger.log_message(
                    LogLevel::Error,
                    &format!("Missing or invalid 'waveform' in synth '{}'.", target),
                );
                return (*max_end_time, cursor_copy);
            }
        };
        let Some(Value::Map(params)) = value_map.get("parameters") else {
            println!("❌ Missing or invalid 'parameters' in synth '{}'.", target);
            return (*max_end_time, cursor_copy);
        };

        // Synth parameters (mutable so we can apply type presets)
        let mut synth_params = params.clone();

        // Apply type defaults using the modular types module
        crate::core::audio::interpreter::statements::arrow_call::types::apply_type(
            &mut synth_params,
        );

        let amp = extract_f32(&synth_params, "amp", base_bpm).unwrap_or(1.0);

        match method.as_str() {
            "note" => {
                return interprete_note_method(
                    args,
                    target,
                    audio_engine,
                    variable_table,
                    global_store,
                    &waveform_str,
                    &synth_params,
                    amp,
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time,
                    cursor_copy,
                    update_cursor,
                );
            }

            "chord" => {
                return interprete_chord_method(
                    args,
                    target,
                    audio_engine,
                    variable_table,
                    global_store,
                    &waveform_str,
                    &synth_params,
                    amp,
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time,
                    cursor_copy,
                    update_cursor,
                );
            }

            _ => {
                let logger = Logger::new();
                logger.log_message(
                    LogLevel::Error,
                    &format!("Unknown method '{}' on synth '{}'.", method, target),
                );
            }
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
