use crate::core::{
    audio::{
        engine::AudioEngine,
        evaluator::evaluate_condition_string,
        interpreter::driver::execute_audio_block,
    },
    parser::statement::Statement,
    shared::value::Value,
    store::{ function::FunctionTable, global::GlobalStore, variable::VariableTable },
};

pub fn interprete_condition_statement(
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
    let mut cur_time = cursor_time;
    let mut max_time = max_end_time;

    let mut current = stmt.value.clone();

    loop {
        let Value::Map(map) = current else {
            break;
        };

        let should_execute = match map.get("condition") {
            Some(Value::Boolean(b)) => *b,
            Some(Value::String(expr)) => evaluate_condition_string(expr, &variable_table.clone()),
            Some(_) => false,
            None => true,
        };

        if should_execute {
            if let Some(Value::Block(block)) = map.get("body") {
                let (new_max, cursor_time) = execute_audio_block(
                    audio_engine,
                    global_store,
                    variable_table.clone(),
                    functions_table.clone(),
                    block.clone(),
                    base_bpm,
                    base_duration,
                    max_time,
                    cur_time
                );
                return (new_max, cursor_time);
            } else {
                break;
            }
        }

        // Advance to the next condition
        match map.get("next") {
            Some(Value::Map(next_map)) => {
                current = Value::Map(next_map.clone());
            }
            _ => {
                break;
            }
        }
    }

    (max_end_time, cursor_time)
}
