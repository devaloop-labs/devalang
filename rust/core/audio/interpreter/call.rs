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
    cursor_time: f32
) -> (AudioEngine, f32, f32) {
    match &stmt.value {
        Value::String(identifier) | Value::Identifier(identifier) => {
            if let Some(Value::Map(map)) = variable_table.clone().get(identifier) {
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

                    return (eng, max_end_time.max(end_time), end_time);
                } else {
                    eprintln!("❌ Group '{}' has no 'body' block", identifier);
                }
            } else {
                eprintln!("❌ Group '{}' not found or not a map", identifier);
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

                return (eng, max_end_time.max(end_time), end_time);
            } else {
                eprintln!("❌ Call map has no 'body' block");
            }
        }

        other => {
            eprintln!("❌ Invalid call statement: expected identifier or map, found {:?}", other);
        }
    }

    (audio_engine, max_end_time, cursor_time)
}
