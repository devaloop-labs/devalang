use rayon::prelude::*;
use std::sync::{ Arc, Mutex };

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
    parser::statement::{ Statement, StatementKind },
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

    let (max_end_time, cursor_time) = execute_audio_block(
        audio_engine,
        global_store,
        global_store.variables.clone(),
        global_store.functions.clone(),
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
    global_store: &GlobalStore,
    variable_table: VariableTable,
    functions_table: FunctionTable,
    statements: Vec<Statement>,
    mut base_bpm: f32,
    mut base_duration: f32,
    mut max_end_time: f32,
    mut cursor_time: f32
) -> (f32, f32) {
    let (spawns, others): (Vec<_>, Vec<_>) = statements
        .into_iter()
        .partition(|stmt| matches!(stmt.kind, StatementKind::Spawn { .. }));

    // Execute sequential statements first
    for stmt in others {
        match &stmt.kind {
            StatementKind::Load { .. } => {
                if
                    let Some(new_table) = interprete_load_statement(
                        &stmt,
                        &mut variable_table.clone()
                    )
                {
                    // Extend the variable_table if necessary
                }
            }
            StatementKind::Let { .. } => {
                interprete_let_statement(&stmt, &mut variable_table.clone());
            }
            StatementKind::Function { .. } => {
                interprete_function_statement(&stmt, &mut functions_table.clone());
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
                    global_store,
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
                let (new_max, _) = interprete_call_statement(
                    &stmt,
                    audio_engine,
                    &variable_table,
                    &functions_table,
                    global_store,
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time,
                );
                cursor_time = new_max;
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

    // Execute parallel spawns (collect results)
    let spawn_results: Vec<(AudioEngine, f32)> = spawns
        .par_iter()
        .map(|stmt| {
            let mut local_engine = AudioEngine::new(audio_engine.module_name.clone());
            let (spawn_max, _) = interprete_spawn_statement(
                stmt,
                &mut local_engine,
                &variable_table,
                &functions_table,
                global_store,
                base_bpm,
                base_duration,
                0.0,
                0.0
            );
            (local_engine, spawn_max)
        })
        .collect();

    // Finally, merge results from all spawns
    for (local_engine, spawn_max) in spawn_results {
        audio_engine.merge_with(local_engine);
        if spawn_max > max_end_time {
            max_end_time = spawn_max;
        }
    }

    (max_end_time.max(cursor_time), cursor_time)
}
