use crate::{
    audio::{ engine::AudioEngine, loader::load_trigger },
    core::{
        parser::statement::{ Statement, StatementKind },
        shared::value::Value,
        store::variable::VariableTable,
    },
};

pub fn interprete_statements(
    statements: &Vec<Statement>,
    audio_engine: AudioEngine,
    entry: String,
    output: String
) -> (AudioEngine, f32, f32) {
    let mut base_bpm = 120.0;
    let mut base_duration = 60.0 / base_bpm;

    let variable_table = audio_engine.variables.clone();

    let (updated_audio_engine, base_bpm, max_end_time) = execute_audio_statements(
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

pub fn execute_audio_statements(
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
            StatementKind::Load { source, alias } => {
                variable_table.set(alias.to_string(), Value::String(source.clone()));
            }

            StatementKind::Let { name } => {
                variable_table.set(name.to_string(), stmt.value.clone());
            }

            StatementKind::Tempo => {
                if let Value::Number(bpm_) = &stmt.value {
                    base_bpm = *bpm_ as f32;
                    base_duration = 60.0 / base_bpm;
                } else {
                    eprintln!("❌ Invalid tempo value: {:?}", stmt.value);
                }
            }

            StatementKind::Trigger { entity, duration } => {
                if let Some(trigger_val) = variable_table.get(entity) {
                    let (src, duration_secs) = load_trigger(
                        trigger_val,
                        duration,
                        base_duration,
                        variable_table.clone()
                    );

                    audio_engine.insert(&src, cursor_time, duration_secs, None);

                    cursor_time += duration_secs;

                    if cursor_time > max_end_time {
                        max_end_time = cursor_time;
                    }
                } else {
                    eprintln!("❌ Unknown trigger entity: {}", entity);
                }
            }

            StatementKind::Loop => {
                if let Value::Map(loop_value) = &stmt.value {
                    let iterator = loop_value.get("iterator");
                    let body = loop_value.get("body");

                    let loop_count = if let Some(Value::Number(n)) = iterator {
                        *n as usize
                    } else {
                        eprintln!("❌ Loop iterator must be a number: {:?}", iterator);
                        continue;
                    };

                    let loop_body = if let Some(Value::Block(body)) = body {
                        body.clone()
                    } else {
                        eprintln!("❌ Loop body must be a block: {:?}", body);
                        continue;
                    };

                    for _ in 0..loop_count {
                        let (loop_engine, _, loop_end_time) = execute_audio_statements(
                            audio_engine.clone(),
                            variable_table.clone(),
                            loop_body.clone(),
                            base_bpm,
                            base_duration,
                            max_end_time,
                            cursor_time
                        );

                        audio_engine = loop_engine;

                        // Update time and max_end_time after each loop iteration
                        cursor_time = loop_end_time;
                        if loop_end_time > max_end_time {
                            max_end_time = loop_end_time;
                        }
                    }
                }
            }

            StatementKind::Bank => {}

            StatementKind::Import { names, source } => {}

            StatementKind::Export { names, source } => {}

            StatementKind::Unknown => {
                // Ignore unknown statements
            }

            _ => {
                eprintln!("Unsupported statement kind: {:?}", stmt);
            }
        }
    }

    audio_engine.set_variables(variable_table);

    (audio_engine, base_bpm, max_end_time)
}
