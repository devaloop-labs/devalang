use std::collections::HashMap;

use crate::core::{
    audio::{engine::AudioEngine, loader::trigger::load_trigger},
    parser::statement::{Statement, StatementKind},
    shared::{duration::Duration, value::Value},
    store::variable::VariableTable,
};
use crate::utils::logger::Logger;

pub fn interprete_trigger_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    base_duration: f32,
    cursor_time: f32,
    max_end_time: f32,
) -> Option<(f32, f32, AudioEngine)> {
    if let StatementKind::Trigger {
        entity,
        duration,
        effects,
    } = &stmt.kind
    {
        let mut trigger_val = Value::String(entity.clone());
        let mut trigger_src = String::new();

        match variable_table.variables.get(entity) {
            Some(Value::Identifier(id)) => {
                // Get real value from global variable table
                if let Some(global_table) = &variable_table.parent {
                    if let Some(val) = global_table.get(id) {
                        trigger_val = val.clone();
                    } else {
                        eprintln!(
                            "❌ Trigger entity '{}' not found in global variable table",
                            id
                        );
                        return None;
                    }
                } else if let Some(val) = variable_table.get(id) {
                    trigger_val = val.clone();
                } else {
                    eprintln!("❌ Trigger entity '{}' not found in variable table", id);
                    return None;
                }
            }
            Some(Value::Sample(sample_src)) => {
                // If the entity is a sample, we use its path directly
                trigger_src = sample_src.clone();
            }
            Some(Value::Map(map)) => {
                // If the entity is a map, we assume it contains an "entity" key
                if let Some(Value::String(src)) = map.get("entity") {
                    trigger_val = Value::String(src.clone());
                } else if let Some(Value::Identifier(src)) = map.get("entity") {
                    trigger_val = Value::Identifier(src.clone());
                } else {
                    eprintln!(
                        "❌ Trigger map must contain an 'entity' key with a string or identifier value."
                    );
                    return None;
                }
            }
            _ => {
                trigger_val = Value::String(entity.clone());
            }
        }

        // If trigger could not be resolved to a known mapping or explicit path, abort early
        if let Value::String(s) = &trigger_val {
            let is_protocol = s.starts_with("devalang://");
            let is_var = variable_table.get(s).is_some()
                || variable_table
                    .parent
                    .as_ref()
                    .and_then(|p| p.get(s))
                    .is_some();
            let looks_like_path = s.contains('/')
                || s.ends_with(".wav")
                || s.ends_with(".mp3")
                || s.ends_with(".ogg");
            if !is_protocol && !is_var && !looks_like_path {
                let logger = Logger::new();
                logger.log_error_with_stacktrace(
                    &format!("unknown trigger: {}", s),
                    &format!("{}:{}:{}", audio_engine.module_name, stmt.line, stmt.column),
                );
                return None;
            }
        }

        let duration_secs = match duration {
            Duration::Number(n) => *n,

            Duration::Identifier(id) => {
                if id == "auto" {
                    1.0
                } else {
                    match variable_table.get(id) {
                        Some(Value::Number(n)) => *n,
                        Some(Value::Identifier(other)) if other == "auto" => 1.0,
                        Some(other) => {
                            eprintln!(
                                "❌ Invalid duration reference '{}': expected number, got {:?}",
                                id, other
                            );
                            return None;
                        }
                        None => {
                            eprintln!("❌ Duration identifier '{}' not found", id);
                            return None;
                        }
                    }
                }
            }

            Duration::Beat(beat_str) => {
                let parts: Vec<&str> = beat_str.split('/').collect();
                if parts.len() != 2 {
                    eprintln!("❌ Invalid beat duration format: {}", beat_str);
                    return None;
                }

                let numerator: f32 = parts[0].parse().unwrap_or(1.0);
                let denominator: f32 = parts[1].parse().unwrap_or(4.0);

                let beats = (numerator / denominator) * 4.0;

                beats * base_duration
            }

            Duration::Auto => base_duration,
        };

        let final_variable_table = if let Some(parent) = &variable_table.parent {
            VariableTable {
                variables: parent.variables.clone(),
                parent: None,
            }
        } else {
            variable_table.clone()
        };

        let (src, sample_length) = load_trigger(
            &trigger_val,
            duration,
            effects,
            base_duration,
            final_variable_table.clone(),
        );

        if trigger_src.is_empty() {
            trigger_src = src;
        }

        let effects = extract_effects(stmt.value.clone());
        let one_shot = effects
            .as_ref()
            .and_then(|map| map.get("one_shot"))
            .and_then(|v| match v {
                Value::Identifier(id) if id == "true" => Some(true),
                Value::String(s) if s == "true" => Some(true),
                _ => None,
            })
            .unwrap_or(false);

        let play_length = if one_shot {
            sample_length // play entire sample
        } else {
            duration_secs.min(sample_length)
        };

        let trigger_src = match trigger_val.get("entity") {
            Some(Value::String(src)) => src.clone(),
            Some(Value::Identifier(id)) => id.clone(),
            Some(Value::Statement(stmt)) => {
                if let StatementKind::Trigger { entity, .. } = &stmt.kind {
                    entity.clone()
                } else {
                    eprintln!("❌ Invalid trigger statement in map: expected 'Trigger' kind");
                    return None;
                }
            }
            _ => trigger_src,
        };

        if let Some(effects_map) = effects {
            audio_engine.insert_sample(
                &trigger_src,
                cursor_time,
                play_length,
                Some(effects_map),
                &final_variable_table,
            );
        } else {
            audio_engine.insert_sample(
                &trigger_src,
                cursor_time,
                play_length,
                None,
                &final_variable_table,
            );
        }

        let new_cursor_time = cursor_time + duration_secs; // advance by beat duration
        let new_max_end_time = (cursor_time + play_length).max(max_end_time);

        let updated_engine = audio_engine.clone();

        return Some((new_cursor_time, new_max_end_time, updated_engine));
    }

    None
}

fn extract_effects(value: Value) -> Option<HashMap<String, Value>> {
    if let Value::Map(map) = value {
        let mut effects = HashMap::new();

        for (key, val) in map {
            if key == "effects" {
                if let Value::Map(effect_map) = val {
                    for (effect_key, effect_val) in effect_map {
                        effects.insert(effect_key, effect_val);
                    }
                } else {
                    return None; // effects must be a map
                }
            } else {
                return Some(effects);
            }
        }

        Some(effects)
    } else {
        None
    }
}
