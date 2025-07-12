use crate::core::{
    audio::{
        engine::AudioEngine,
        interpreter::{
            call::interprete_call_statement,
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
    store::variable::VariableTable,
};

pub fn run_audio_program(
    statements: &Vec<Statement>,
    audio_engine: AudioEngine,
    entry: String,
    output: String
) -> (AudioEngine, f32, f32) {
    let mut base_bpm = 120.0;
    let mut base_duration = 60.0 / base_bpm;

    let variable_table = audio_engine.variables.clone();

    let (updated_audio_engine, base_bpm, max_end_time) = execute_audio_block(
        audio_engine.clone(),
        variable_table.clone(),
        statements.clone(),
        base_bpm.clone(),
        base_duration.clone(),
        0.0,
        0.0
    );

    (updated_audio_engine, base_bpm, max_end_time)
}

pub fn execute_audio_block(
    mut audio_engine: AudioEngine,
    mut variable_table: VariableTable,
    mut statements: Vec<Statement>,
    mut base_bpm: f32,
    mut base_duration: f32,
    mut max_end_time: f32,
    mut cursor_time: f32
) -> (AudioEngine, f32, f32) {
    for stmt in statements {
        match &stmt.kind {
            StatementKind::Load { .. } => {
                if
                    let Some(new_variable_table) = interprete_load_statement(
                        &stmt,
                        &mut variable_table
                    )
                {
                    variable_table = new_variable_table;
                } else {
                    eprintln!("❌ Failed to interpret load statement: {:?}", stmt);
                }
            }

            StatementKind::Let { .. } => {
                if
                    let Some(new_variable_table) = interprete_let_statement(
                        &stmt,
                        &mut variable_table
                    )
                {
                    variable_table = new_variable_table;
                } else {
                    eprintln!("❌ Failed to interpret let statement: {:?}", stmt);
                }
            }

            StatementKind::Tempo => {
                if let Some((new_bpm, new_duration)) = interprete_tempo_statement(&stmt) {
                    base_bpm = new_bpm;
                    base_duration = new_duration;
                } else {
                    eprintln!("❌ Failed to interpret tempo statement: {:?}", stmt);
                }
            }

            StatementKind::Trigger { .. } => {
                if
                    let Some((new_cursor_time, new_max_end_time, updated_engine)) =
                        interprete_trigger_statement(
                            &stmt,
                            &mut audio_engine,
                            &variable_table,
                            base_duration,
                            cursor_time,
                            max_end_time
                        )
                {
                    cursor_time = new_cursor_time;
                    max_end_time = new_max_end_time;
                    audio_engine = updated_engine;
                } else {
                    eprintln!("❌ Failed to interpret trigger statement: {:?}", stmt);
                }
            }

            StatementKind::Spawn => {
                if
                    let Some((new_cursor_time, new_max_end_time, updated_engine)) =
                        interprete_spawn_statement(
                            &stmt,
                            &mut audio_engine,
                            &variable_table,
                            base_bpm,
                            base_duration,
                            cursor_time,
                            max_end_time
                        )
                {
                    cursor_time = new_cursor_time;
                    max_end_time = new_max_end_time;
                    audio_engine = updated_engine;
                } else {
                    eprintln!("❌ Failed to interpret spawn statement: {:?}", stmt);
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
                let (loop_engine, new_max, new_cursor) = interprete_loop_statement(
                    &stmt,
                    audio_engine.clone(),
                    variable_table.clone(),
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );
                audio_engine = loop_engine;
                cursor_time = new_cursor;
                max_end_time = new_max;
            }

            StatementKind::Call => {
                let (call_engine, new_max, new_cursor) = interprete_call_statement(
                    &stmt,
                    audio_engine.clone(),
                    variable_table.clone(),
                    base_bpm,
                    base_duration,
                    max_end_time,
                    cursor_time
                );
                audio_engine = call_engine;
                cursor_time = new_cursor;
                max_end_time = new_max;
            }

            | StatementKind::Bank
            | StatementKind::Import { .. }
            | StatementKind::Export { .. }
            | StatementKind::Group
            | StatementKind::Unknown => {
                // NOTE: Ignoring unsupported statement kinds for now.
            }

            _ => {
                eprintln!("Unsupported audio statement kind: {:?}", stmt);
            }
        }
    }

    audio_engine.set_variables(variable_table);

    (audio_engine, base_bpm, max_end_time)
}
