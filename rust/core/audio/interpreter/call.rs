use crate::core::{
    audio::{ engine::AudioEngine, interpreter::{ driver::execute_audio_block } },
    parser::statement::{ Statement, StatementKind },
    shared::{ duration::Duration, value::Value },
    store::variable::VariableTable,
};

pub fn interprete_call_statement(
    stmt: &Statement,
    audio_engine: AudioEngine,
    variable_table: VariableTable,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32,
    all_statements: &Vec<Statement>
) -> (AudioEngine, f32, f32, f32) {
    match &stmt.value {
        Value::String(identifier) | Value::Identifier(identifier) => {
            if
                let Some(group_stmt) = all_statements
                    .iter()
                    .find(|s| {
                        matches!(s.kind, StatementKind::Group) &&
                            s.value.get("identifier") == Some(&Value::String(identifier.clone()))
                    })
            {
                if let Some(Value::Block(block)) = group_stmt.value.get("body") {
                    let (eng, _, end_time) = execute_audio_block(
                        audio_engine,
                        variable_table,
                        block.clone(),
                        base_bpm,
                        base_duration,
                        max_end_time,
                        cursor_time
                    );
                    return (eng, max_end_time.max(end_time), end_time, cursor_time);
                } else {
                    eprintln!("❌ Group '{}' found but no valid body block", identifier);
                }
            } else {
                eprintln!("❌ Group '{}' not found in statements", identifier);
            }
        }

        Value::Map(map) => {
            if let Some(Value::Block(block)) = map.get("body") {
                let (eng, _, end_time) = execute_audio_block(
                    audio_engine,
                    variable_table,
                    block.clone(),
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );
                return (eng, max_end_time.max(end_time), end_time, cursor_time);
            } else {
                eprintln!("❌ Call map has no 'body' block");
            }
        }

        other => {
            eprintln!("❌ Invalid call statement: expected identifier or map, found {:?}", other);
        }
    }

    (audio_engine, base_bpm, max_end_time, cursor_time)
}
