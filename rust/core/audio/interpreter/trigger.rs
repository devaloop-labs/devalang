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
        if let Some(trigger_val) = variable_table.get(entity) {
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
