use crate::{
    core::{
        audio::{ engine::AudioEngine, loader::load_trigger },
        parser::statement::{ Statement, StatementKind },
        shared::{ duration::Duration, value::Value },
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
                    // Étape 1 : Résolution de duration
                    let duration_secs = match duration {
                        Duration::Number(n) => *n,

                        Duration::Identifier(id) => {
                            if id == "auto" {
                                1.0 // Valeur par défaut pour "auto"
                            } else {
                                match variable_table.get(id) {
                                    Some(Value::Number(n)) => *n,
                                    Some(Value::Identifier(other)) => {
                                        if other == "auto" {
                                            1.0 // Valeur par défaut pour "auto"
                                        } else {
                                            eprintln!("❌ Invalid duration identifier '{}': expected number or string, got identifier", id);
                                            continue;
                                        }
                                    }
                                    Some(other) => {
                                        eprintln!(
                                            "❌ Invalid duration reference '{}': expected number or string, got {:?}",
                                            id,
                                            other
                                        );
                                        continue;
                                    }
                                    None => {
                                        eprintln!("❌ Duration identifier '{}' not found in variable table", id);
                                        continue;
                                    }
                                }
                            }
                        }

                        Duration::Auto => {
                            // Si "auto", tu choisis une valeur par défaut ou duration du sample ?
                            1.0 // ← par exemple 1.0 beat
                        }
                    };

                    // Durée réelle en secondes selon tempo
                    let duration_final = duration_secs * base_duration;

                    // Chargement de l'audio
                    let (src, _) = load_trigger(
                        trigger_val,
                        duration,
                        base_duration,
                        variable_table.clone()
                    );

                    audio_engine.insert(&src, cursor_time, duration_final, None);

                    cursor_time += duration_final;
                    if cursor_time > max_end_time {
                        max_end_time = cursor_time;
                    }
                } else {
                    eprintln!("❌ Unknown trigger entity: {}", entity);
                }
            }

            StatementKind::Spawn => {
                if let Value::String(identifier) = &stmt.value {
                    match variable_table.get(identifier) {
                        Some(Value::Map(map)) => {
                            if let Some(Value::Block(block)) = map.get("body") {
                                let mut local_max = cursor_time;

                                for inner_stmt in block {
                                    let (inner_engine, _, inner_end_time) =
                                        execute_audio_statements(
                                            audio_engine.clone(),
                                            variable_table.clone(),
                                            vec![inner_stmt.clone()],
                                            base_bpm,
                                            base_duration,
                                            max_end_time,
                                            cursor_time // <- important: same cursor time
                                        );
                                    audio_engine = inner_engine;
                                    if inner_end_time > local_max {
                                        local_max = inner_end_time;
                                    }
                                }

                                // Update cursor once all done
                                if local_max > max_end_time {
                                    max_end_time = local_max;
                                }
                                cursor_time = local_max;
                            }
                        }
                        _ => eprintln!("❌ Cannot spawn '{}'", identifier),
                    }
                }
            }

            StatementKind::Sleep => {
                let duration_secs = match &stmt.value {
                    Value::Number(ms) => *ms / 1000.0,

                    Value::String(s) if s.ends_with("ms") => {
                        let ms = s.trim_end_matches("ms").parse::<f32>();
                        match ms {
                            Ok(ms) => ms / 1000.0,
                            Err(_) => {
                                eprintln!("❌ Invalid sleep value (ms): {}", s);
                                continue;
                            }
                        }
                    }

                    Value::String(s) if s.ends_with("s") => {
                        let s_ = s.trim_end_matches("s").parse::<f32>();
                        match s_ {
                            Ok(secs) => secs,
                            Err(_) => {
                                eprintln!("❌ Invalid sleep value (s): {}", s);
                                continue;
                            }
                        }
                    }

                    other => {
                        eprintln!("❌ Invalid sleep value: {:?}", other);
                        continue;
                    }
                };

                cursor_time += duration_secs;

                if cursor_time > max_end_time {
                    max_end_time = cursor_time;
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

            StatementKind::Call => {
                if let Value::String(identifier) = &stmt.value {
                    match variable_table.get(identifier) {
                        Some(Value::Map(map)) => {
                            match map.get("body") {
                                Some(Value::Block(block)) => {
                                    let (called_engine, _, called_end_time) =
                                        execute_audio_statements(
                                            audio_engine.clone(),
                                            variable_table.clone(),
                                            block.clone(),
                                            base_bpm,
                                            base_duration,
                                            max_end_time,
                                            cursor_time
                                        );

                                    audio_engine = called_engine;
                                    cursor_time = called_end_time;
                                    if called_end_time > max_end_time {
                                        max_end_time = called_end_time;
                                    }
                                }
                                Some(other) => {
                                    eprintln!(
                                        "❌ Cannot call '{}': expected 'body' to be a block, got {:?}",
                                        identifier,
                                        other
                                    );
                                }
                                None => {
                                    eprintln!("❌ Cannot call '{}': missing 'body' in group map", identifier);
                                }
                            }
                        }
                        Some(other) => {
                            eprintln!(
                                "❌ Cannot call '{}': expected a Map with 'body', got {:?}",
                                identifier,
                                other
                            );
                        }
                        None => {
                            eprintln!("❌ Cannot call '{}': not found in variable table", identifier);
                        }
                    }
                } else {
                    eprintln!(
                        "❌ Invalid call statement: expected a string identifier, got {:?}",
                        stmt.value
                    );
                }
            }

            StatementKind::Bank => {}

            StatementKind::Import { names, source } => {}

            StatementKind::Export { names, source } => {}

            StatementKind::Group => {}

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
