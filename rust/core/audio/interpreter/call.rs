use crate::core::{
    audio::{ engine::AudioEngine, interpreter::driver::execute_audio_block },
    parser::statement::{ Statement, StatementKind },
    store::{ function::{ FunctionTable }, variable::VariableTable },
};

pub fn interprete_call_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    functions: &FunctionTable,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32
) -> (f32, f32) {
    match &stmt.kind {
        StatementKind::Call { name, args } => {
            if let Some(func) = functions.functions.get(name) {
                if func.parameters.len() != args.len() {
                    eprintln!(
                        "❌ Function '{}' expects {} args, got {}",
                        name,
                        func.parameters.len(),
                        args.len()
                    );
                    return (max_end_time, cursor_time);
                }

                let mut local_vars = VariableTable::with_parent(variable_table.clone());

                for (param, arg) in func.parameters.iter().zip(args) {
                    local_vars.set(param.clone(), arg.clone());
                }

                let (new_max, new_cursor) = execute_audio_block(
                    audio_engine,
                    local_vars,
                    functions.clone(),
                    func.body.clone(),
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );

                return (new_max, new_cursor);
            } else {
                eprintln!("❌ Function '{}' not found", name);
            }
        }

        _ => {
            eprintln!("❌ interprete_call_statement expected Call, got {:?}", stmt.kind);
        }
    }

    (max_end_time, cursor_time)
}
