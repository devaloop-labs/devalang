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
    max_end_time: f32
) -> Option<(f32, f32, AudioEngine)> {
    if let Value::String(identifier) = &stmt.value {
        match variable_table.get(identifier) {
            Some(Value::Map(map)) => {
                if let Some(Value::Block(block)) = map.get("body") {
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
                            cursor_time
                        );

                        updated_engine = inner_engine;

                        if inner_end_time > local_max {
                            local_max = inner_end_time;
                        }
                    }

                    let new_max_end_time = local_max.max(max_end_time);

                    return Some((local_max, new_max_end_time, updated_engine));
                } else {
                    eprintln!("❌ Cannot spawn '{}': no valid body block", identifier);
                }
            }
            Some(other) => {
                eprintln!(
                    "❌ Cannot spawn '{}': expected map with block, got {:?}",
                    identifier,
                    other
                );
            }
            None => {
                eprintln!("❌ Cannot spawn '{}': not found in variable table", identifier);
            }
        }
    } else {
        eprintln!("❌ Invalid spawn statement: expected a string identifier, got {:?}", stmt.value);
    }

    None
}
