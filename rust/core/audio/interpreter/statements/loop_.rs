use devalang_types::Value;

use crate::core::{
    audio::{engine::AudioEngine, interpreter::driver::execute_audio_block},
    parser::statement::Statement,
    store::global::GlobalStore,
};
use devalang_types::store::{FunctionTable, VariableTable};

pub fn interprete_loop_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    global_store: &GlobalStore,
    variable_table: &VariableTable,
    functions_table: &FunctionTable,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32,
) -> (f32, f32) {
    if let Value::Map(loop_value) = &stmt.value {
        // Foreach form: { foreach: Identifier(name), array: Array([...]), body: Block }
        if let (
            Some(Value::Identifier(var_name)),
            Some(Value::Array(items)),
            Some(Value::Block(loop_body)),
        ) = (
            loop_value.get("foreach"),
            loop_value.get("array"),
            loop_value.get("body"),
        ) {
            let engine = audio_engine;
            let mut cur_time = cursor_time;
            let mut max_time = max_end_time;

            for item in items {
                let mut scoped_vars = variable_table.clone();
                scoped_vars.set(var_name.clone(), item.clone());

                let (block_end_time, new_cursor) = execute_audio_block(
                    engine,
                    global_store,
                    scoped_vars,
                    functions_table.clone(),
                    loop_body,
                    base_bpm,
                    base_duration,
                    max_time,
                    cur_time,
                );

                cur_time = new_cursor.max(block_end_time);
                max_time = max_time.max(cur_time);
            }

            return (max_time, cur_time);
        }

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
                eprintln!(
                    "❌ Loop body must be a block, found: {:?}",
                    loop_value.get("body")
                );
                return (max_end_time, cursor_time);
            }
        };

        let engine = audio_engine;
        let mut cur_time = cursor_time;
        let mut max_time = max_end_time;

        for _ in 0..loop_count {
            let (block_end_time, new_cursor) = execute_audio_block(
                engine,
                global_store,
                variable_table.clone(),
                functions_table.clone(),
                &loop_body,
                base_bpm,
                base_duration,
                max_time,
                cur_time,
            );

            cur_time = new_cursor.max(block_end_time);
            max_time = max_time.max(cur_time);
        }

        return (max_time, cur_time);
    }

    eprintln!("❌ Loop statement value is not a map");
    (max_end_time, cursor_time)
}
