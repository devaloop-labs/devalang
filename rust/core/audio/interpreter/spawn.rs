use crate::core::{
    audio::{ engine::AudioEngine, interpreter::driver::execute_audio_block },
    parser::statement::Statement,
    shared::value::Value,
    store::{ function::FunctionTable, variable::VariableTable },
};

pub fn interprete_spawn_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &mut VariableTable,
    functions_table: &mut FunctionTable,
    base_bpm: f32,
    base_duration: f32,
    cursor_time: f32,
    max_end_time: f32
) -> Option<(f32, f32)> {
    match &stmt.value {
        Value::String(identifier) | Value::Identifier(identifier) => {
            handle_spawn_identifier(
                identifier,
                audio_engine,
                variable_table,
                functions_table,
                base_bpm,
                base_duration,
                cursor_time,
                max_end_time
            )
        }

        Value::Map(map) => {
            if let Some(Value::Block(block)) = map.get("body") {
                let (end_time, cursor_time) = execute_audio_block(
                    audio_engine,
                    variable_table.clone(),
                    functions_table.clone(),
                    block.clone(),
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );
                return Some((max_end_time.max(end_time), cursor_time));
            } else {
                eprintln!("❌ Spawn map has no 'body' block");
            }
            None
        }

        _ => {
            eprintln!("❌ Invalid spawn statement: expected identifier, found {:?}", stmt.value);
            None
        }
    }
}

fn handle_spawn_identifier(
    identifier: &str,
    audio_engine: &mut AudioEngine,
    variable_table: &mut VariableTable,
    functions_table: &mut FunctionTable,
    base_bpm: f32,
    base_duration: f32,
    cursor_time: f32,
    max_end_time: f32
) -> Option<(f32, f32)> {
    if let Some(Value::Map(map)) = variable_table.get(identifier) {
        if let Some(Value::Block(block)) = map.get("body") {
            let (end_time, cursor_time) = execute_audio_block(
                audio_engine,
                variable_table.clone(),
                functions_table.clone(),
                block.clone(),
                base_bpm,
                base_duration,
                max_end_time,
                cursor_time
            );
            return Some((max_end_time.max(end_time), cursor_time));
        } else {
            eprintln!("❌ Spawn group '{}' has no 'body' block", identifier);
        }
    } else {
        eprintln!("❌ Spawn group '{}' not found or not a map", identifier);
    }

    None
}
