use crate::core::{
    audio::{ engine::AudioEngine, loader::trigger::load_trigger },
    parser::statement::{ Statement, StatementKind },
    shared::{ duration::Duration, value::Value },
    store::variable::VariableTable,
};

pub fn interprete_trigger_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    base_duration: f32,
    cursor_time: f32,
    max_end_time: f32
) -> Option<(f32, f32, AudioEngine)> {
    if let StatementKind::Trigger { entity, duration } = &stmt.kind {
        if let Some(trigger_val) = resolve_namespaced_variable(entity, variable_table) {
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
                                    id,
                                    other
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
                    // Assuming beat_str is in the format "numerator/denominator"
                    let parts: Vec<&str> = beat_str.split('/').collect();

                    if parts.len() != 2 {
                        eprintln!("❌ Invalid beat duration format: {}", beat_str);
                        return None;
                    }

                    let numerator: f32 = parts[0].parse().unwrap_or(1.0);
                    let denominator: f32 = parts[1].parse().unwrap_or(1.0);
                    numerator / denominator
                }

                Duration::Auto => 1.0,
            };

            let duration_final = duration_secs * base_duration;

            let (src, _) = load_trigger(
                trigger_val,
                duration,
                base_duration,
                variable_table.clone()
            );

            let mut updated_engine = audio_engine.clone();
            updated_engine.insert_sample(&src, cursor_time, duration_final, None);

            let new_cursor_time = cursor_time + duration_final;
            let new_max_end_time = new_cursor_time.max(max_end_time);

            return Some((new_cursor_time, new_max_end_time, updated_engine));
        } else {
            eprintln!("❌ Unknown trigger entity: {}", entity);
        }
    }

    None
}

fn resolve_namespaced_variable<'a>(path: &str, variables: &'a VariableTable) -> Option<&'a Value> {
    let mut current: Option<&Value> = None;

    for (i, part) in path.split('.').enumerate() {
        if i == 0 {
            current = variables.get(part);
        } else {
            current = match current {
                Some(Value::Map(map)) => map.get(part),
                _ => {
                    return None;
                }
            };
        }
    }

    current
}
