use crate::core::{
    audio::{ engine::AudioEngine, interpreter::driver::execute_audio_block },
    parser::statement::{ Statement, StatementKind },
    shared::{ duration::Duration, value::Value },
    store::variable::VariableTable,
};

pub fn interprete_loop_statement(
    stmt: &Statement,
    audio_engine: AudioEngine,
    variable_table: VariableTable,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32
) -> (AudioEngine, f32, f32) {
    if let Value::Map(loop_value) = &stmt.value {
        let loop_count = match loop_value.get("iterator") {
            Some(Value::Number(n)) => *n as usize,
            _ => {
                eprintln!("❌ Loop iterator must be a number");
                return (audio_engine, max_end_time, cursor_time);
            }
        };

        let loop_body = match loop_value.get("body") {
            Some(Value::Block(body)) => body.clone(),
            _ => {
                eprintln!("❌ Loop body must be a block");
                return (audio_engine, max_end_time, cursor_time);
            }
        };

        let mut engine = audio_engine;
        let mut cur_time = cursor_time;
        let mut max_time = max_end_time;

        for _ in 0..loop_count {
            let (eng, _, end_time) = execute_audio_block(
                engine.clone(),
                variable_table.clone(),
                loop_body.clone(),
                base_bpm,
                base_duration,
                max_time,
                cur_time
            );

            engine = eng;
            cur_time = end_time;
            max_time = max_time.max(end_time);
        }

        (engine, max_time, cur_time)
    } else {
        eprintln!("❌ Loop statement value is not a map");
        (audio_engine, max_end_time, cursor_time)
    }
}
