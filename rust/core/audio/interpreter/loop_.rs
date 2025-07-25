use crate::core::{
    audio::{ engine::AudioEngine, interpreter::driver::execute_audio_block },
    parser::statement::{ Statement, StatementKind },
    shared::{ duration::Duration, value::Value },
    store::{function::FunctionTable, global::GlobalStore, variable::VariableTable},
};

pub fn interprete_loop_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    global_store: &GlobalStore,
    variable_table: &VariableTable,
    functions_table: &FunctionTable,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32
) -> (f32, f32) {
    if let Value::Map(loop_value) = &stmt.value {
        let loop_count = match loop_value.get("iterator") {
            Some(Value::Number(n)) => *n as usize,
            Some(Value::Identifier(ident)) => {
                if let Some(Value::Number(n)) = variable_table.get(ident) {
                    *n as usize
                } else {
                    eprintln!("❌ Loop iterator must be a number, found: {:?}", ident);
                    return (max_end_time, cursor_time);
                }
            }
            _ => {
                eprintln!(
                    "❌ Loop iterator must be a number, found: {:?}",
                    loop_value.get("iterator")
                );
                return (max_end_time, cursor_time);
            }
        };

        let loop_body = match loop_value.get("body") {
            Some(Value::Block(body)) => body.clone(),
            _ => {
                eprintln!("❌ Loop body must be a block, found: {:?}", loop_value.get("body"));
                return (max_end_time, cursor_time);
            }
        };

        let mut engine = audio_engine;
        let mut cur_time = cursor_time;
        let mut max_time = max_end_time;

        for i in 0..loop_count {
            let (block_end_time, cursor_time) = execute_audio_block(
                &mut engine,
                global_store,
                variable_table.clone(),
                functions_table.clone(),
                loop_body.clone(),
                base_bpm,
                base_duration,
                max_time,
                cur_time
            );

            cur_time = block_end_time;
            max_time = max_time.max(cur_time);
        }

        return (max_time, cur_time);
    }

    eprintln!("❌ Loop statement value is not a map");
    (max_end_time, cursor_time)
}
