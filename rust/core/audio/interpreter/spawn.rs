use crate::core::{
    audio::{ engine::AudioEngine, interpreter::driver::execute_audio_block },
    parser::statement::Statement,
    shared::value::Value,
    store::variable::VariableTable,
};

pub fn interprete_spawn_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    base_bpm: f32,
    base_duration: f32,
    cursor_time: f32,
    max_end_time: f32,
) -> Option<(f32, f32, AudioEngine)> {
    let resolved_block_opt = match &stmt.value {
        Value::String(ident) | Value::Identifier(ident) => {
            match variable_table.get(ident) {
                Some(Value::Map(map)) => map.get("body").cloned(),
                Some(other) => {
                    eprintln!("❌ Spawn target '{}' is not a map, got {:?}", ident, other);
                    None
                }
                None => {
                    eprintln!("❌ Spawn target '{}' not found in variable table", ident);
                    None
                }
            }
        }
        Value::Map(map) => map.get("body").cloned(),
        other => {
            eprintln!("❌ Invalid spawn statement value: expected string, identifier or map, got {:?}", other);
            None
        }
    };

    let Some(Value::Block(block)) = resolved_block_opt else {
        eprintln!("❌ No valid block found to spawn");
        return None;
    };

    let mut local_max = cursor_time;
    let mut updated_engine = audio_engine.clone();

    for inner_stmt in block {
        let (inner_engine, _, inner_end_time) = execute_audio_block(
            updated_engine.clone(),
            variable_table.clone(),
            vec![inner_stmt.clone()],
            base_bpm,
            base_duration,
            max_end_time,
            cursor_time,
        );

        updated_engine = inner_engine;
        if inner_end_time > local_max {
            local_max = inner_end_time;
        }
    }

    let new_max_end_time = local_max.max(max_end_time);
    Some((local_max, new_max_end_time, updated_engine))
}
