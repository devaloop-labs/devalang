use crate::core::{
    audio::{
        engine::AudioEngine,
        evaluator::evaluate_condition_string,
        interpreter::driver::execute_audio_block,
    },
    parser::statement::Statement,
    shared::value::Value,
    store::variable::VariableTable,
};

pub fn interprete_condition_statement(
    stmt: &Statement,
    audio_engine: AudioEngine,
    variable_table: VariableTable,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32
) -> (AudioEngine, f32, f32) {
    let mut engine = audio_engine.clone();
    let mut vars = variable_table.clone();
    let mut cur_time = cursor_time;
    let mut max_time = max_end_time;

    let mut current = stmt.value.clone();

    loop {
        let Value::Map(map) = current else {
            break;
        };

        let should_execute = match map.get("condition") {
            Some(Value::Boolean(b)) => *b,
            Some(Value::String(expr)) => evaluate_condition_string(expr, &vars),
            Some(_) => false,
            None => true,
        };

        if should_execute {
            if let Some(Value::Block(block)) = map.get("body") {
                let (new_engine, _, new_max) = execute_audio_block(
                    engine,
                    vars,
                    block.clone(),
                    base_bpm,
                    base_duration,
                    max_time,
                    cur_time
                );
                return (new_engine, new_max, new_max);
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

    (audio_engine, max_end_time, cursor_time)
}
