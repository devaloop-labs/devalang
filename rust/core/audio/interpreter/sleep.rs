use crate::core::{
    audio::{ engine::AudioEngine, loader::trigger::load_trigger },
    parser::statement::{ Statement, StatementKind },
    shared::{ duration::Duration, value::Value },
    store::variable::VariableTable,
};

pub fn interprete_sleep_statement(
    stmt: &Statement,
    cursor_time: f32,
    max_end_time: f32,
) -> (f32, f32) {
    let duration_secs = match &stmt.value {
        Value::Number(ms) => *ms / 1000.0,
        Value::String(s) if s.ends_with("ms") => {
            s.trim_end_matches("ms").parse::<f32>().map(|ms| ms / 1000.0).unwrap_or_else(|_| {
                eprintln!("❌ Invalid sleep value (ms): {}", s);
                0.0
            })
        }
        Value::String(s) if s.ends_with("s") => {
            s.trim_end_matches("s").parse::<f32>().unwrap_or_else(|_| {
                eprintln!("❌ Invalid sleep value (s): {}", s);
                0.0
            })
        }
        other => {
            eprintln!("❌ Invalid sleep value: {:?}", other);
            0.0
        }
    };

    let new_cursor = cursor_time + duration_secs;
    let new_max = max_end_time.max(new_cursor);
    (new_cursor, new_max)
}
