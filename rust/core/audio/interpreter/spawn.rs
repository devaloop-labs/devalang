use devalang_types::Value;

use crate::core::{
    audio::{engine::AudioEngine, interpreter::driver::execute_audio_block},
    parser::statement::{Statement, StatementKind},
    store::{function::FunctionTable, global::GlobalStore, variable::VariableTable},
};

pub fn interprete_spawn_statement(
    stmt: &Statement,
    audio_engine: &mut AudioEngine,
    variable_table: &VariableTable,
    functions: &FunctionTable,
    global_store: &GlobalStore,
    base_bpm: f32,
    base_duration: f32,
    max_end_time: f32,
    cursor_time: f32,
) -> (f32, f32) {
    if let StatementKind::Spawn { name, args } = &stmt.kind {
        let mut local_engine = AudioEngine::new(audio_engine.module_name.clone());

        // Function case
        if let Some(func) = functions.functions.get(name) {
            if func.parameters.len() != args.len() {
                return (max_end_time, cursor_time);
            }

            let mut local_vars = VariableTable::with_parent(variable_table.clone());
            for (param, arg) in func.parameters.iter().zip(args) {
                local_vars.set(param.clone(), arg.clone());
            }

            let (spawn_max, _) = execute_audio_block(
                &mut local_engine,
                global_store,
                local_vars,
                functions.clone(),
                &func.body,
                base_bpm,
                base_duration,
                0.0,
                0.0,
            );

            audio_engine.merge_with(local_engine);
            return (spawn_max.max(max_end_time), cursor_time);
        }

        // Group case
        if let Some(group_stmt) = find_group(name, variable_table, global_store) {
            if let Value::Map(map) = &group_stmt.value {
                if let Some(Value::Block(body)) = map.get("body") {
                    let (spawn_max, _) = execute_audio_block(
                        &mut local_engine,
                        global_store,
                        variable_table.clone(),
                        functions.clone(),
                        body,
                        base_bpm,
                        base_duration,
                        0.0,
                        0.0,
                    );
                    audio_engine.merge_with(local_engine);
                    return (spawn_max.max(max_end_time), cursor_time);
                }
            }
        }

        // Function or group not found
    }

    (max_end_time, cursor_time)
}

fn find_group<'a>(
    name: &str,
    variable_table: &'a VariableTable,
    global_store: &'a GlobalStore,
) -> Option<&'a Statement> {
    if let Some(Value::Statement(stmt_box)) = variable_table.get(name) {
        if let StatementKind::Group = stmt_box.kind {
            return Some(stmt_box);
        }
    }
    if let Some(Value::Statement(stmt_box)) = global_store.variables.variables.get(name) {
        if let StatementKind::Group = stmt_box.kind {
            return Some(stmt_box);
        }
    }
    None
}
