use crate::core::{
    audio::{
        engine::AudioEngine,
        interpreter::{
            arrow_call::interprete_call_arrow_statement,
            call::interprete_call_statement,
            condition::interprete_condition_statement,
            function::interprete_function_statement,
            let_::interprete_let_statement,
            load::interprete_load_statement,
            loop_::interprete_loop_statement,
            sleep::interprete_sleep_statement,
            spawn::interprete_spawn_statement,
            tempo::interprete_tempo_statement,
            trigger::interprete_trigger_statement,
        },
    },
    parser::statement::{ self, Statement, StatementKind },
    shared::value::Value,
    store::{ function::FunctionTable, global::GlobalStore, variable::VariableTable },
};

pub fn run_audio_program(
    statements: &Vec<Statement>,
    audio_engine: &mut AudioEngine,
    entry: String,
    output: String,
    mut module_variables: VariableTable,
    mut module_functions: FunctionTable,
    global_store: &mut GlobalStore
) -> (f32, f32) {
    let mut base_bpm = 120.0;
    let mut base_duration = 60.0 / base_bpm;

    // Fill the variable table with global variables
    module_variables.variables.extend(global_store.variables.variables.clone());

    // Fill the functions table with global functions
    module_functions.functions.extend(global_store.functions.functions.clone());

    let (max_end_time, cursor_time) = execute_audio_block(
        audio_engine,
        module_variables,
        module_functions,
        statements.clone(),
        base_bpm,
        base_duration,
        0.0,
        0.0
    );

    (max_end_time, cursor_time)
}

pub fn execute_audio_block(
    audio_engine: &mut AudioEngine,
    mut variable_table: VariableTable,
    mut functions_table: FunctionTable,
    statements: Vec<Statement>,
    mut base_bpm: f32,
    mut base_duration: f32,
    mut max_end_time: f32,
    mut cursor_time: f32
) -> (f32, f32) {
    for stmt in &statements {
        match &stmt.kind {
            StatementKind::Load { .. } => {
                if let Some(new_table) = interprete_load_statement(&stmt, &mut variable_table) {
                    variable_table.variables.extend(new_table.variables);
                }
            }

            StatementKind::Let { .. } => {
                if let Some(new_table) = interprete_let_statement(&stmt, &mut variable_table) {
                    variable_table.variables.extend(new_table.variables);
                }
            }

            StatementKind::Function { name, parameters, body } => {
                if let Some(new_table) = interprete_function_statement(&stmt, &mut functions_table) {
                    functions_table.functions.extend(new_table.functions);
                }
            }

            StatementKind::Tempo => {
                if let Some((new_bpm, new_duration)) = interprete_tempo_statement(&stmt) {
                    base_bpm = new_bpm;
                    base_duration = new_duration;
                }
            }

            StatementKind::Trigger { .. } => {
                if
                    let Some((new_cursor, new_max, _)) = interprete_trigger_statement(
                        &stmt,
                        audio_engine,
                        &variable_table,
                        base_duration,
                        cursor_time,
                        max_end_time
                    )
                {
                    cursor_time = new_cursor;
                    max_end_time = new_max;
                }
            }

            StatementKind::Sleep => {
                let (new_cursor, new_max) = interprete_sleep_statement(
                    &stmt,
                    cursor_time,
                    max_end_time
                );
                cursor_time = new_cursor;
                max_end_time = new_max;
            }

            StatementKind::Loop => {
                let (new_max, new_cursor) = interprete_loop_statement(
                    &stmt,
                    audio_engine,
                    &variable_table,
                    &functions_table,
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );
                cursor_time = new_cursor;
                max_end_time = new_max;
            }

            StatementKind::Call { .. } => {
                let (new_max, new_cursor) = interprete_call_statement(
                    &stmt,
                    audio_engine,
                    &variable_table,
                    &functions_table,
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );
                cursor_time = new_cursor;
                max_end_time = new_max;
            }

            StatementKind::ArrowCall { .. } => {
                let (new_max, new_cursor) = interprete_call_arrow_statement(
                    &stmt,
                    audio_engine,
                    &variable_table,
                    base_bpm,
                    base_duration,
                    &mut max_end_time,
                    Some(&mut cursor_time),
                    true
                );
                cursor_time = new_cursor;
                max_end_time = new_max;
            }

            _ => {}
        }
    }

    (max_end_time.max(cursor_time), cursor_time)
}
