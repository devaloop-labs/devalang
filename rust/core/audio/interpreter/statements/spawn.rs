use devalang_types::{Duration, Value};

use crate::core::{
    audio::{engine::AudioEngine, interpreter::driver::execute_audio_block},
    parser::statement::{Statement, StatementKind},
    store::global::GlobalStore,
};
use devalang_types::store::{FunctionTable, VariableTable};

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

        // Pattern case: allow spawning a pattern similar to call
        if let Some(pattern_stmt) = find_pattern(name, variable_table, global_store) {
            // Support Value::String(pattern) or Value::Map({ pattern: "..", swing: .., humanize: .. })
            let mut pat_opt: Option<String> = None;
            let mut opts_map: Option<std::collections::HashMap<String, Value>> = None;

            match &pattern_stmt.value {
                Value::String(s) => pat_opt = Some(s.clone()),
                Value::Map(m) => {
                    if let Some(Value::String(s)) = m.get("pattern") {
                        pat_opt = Some(s.clone());
                    }
                    opts_map = Some(m.clone());
                }
                _ => {}
            }

            if let Some(pat) = pat_opt {
                let mut target_entity = name.clone();
                if let StatementKind::Pattern { name: _n, target } = &pattern_stmt.kind {
                    if let Some(t) = target {
                        target_entity = t.clone();
                    }
                }

                let final_variable_table = if let Some(parent) = &variable_table.parent {
                    devalang_types::VariableTable {
                        variables: parent.variables.clone(),
                        parent: None,
                    }
                } else {
                    variable_table.clone()
                };

                let pattern_str: String = pat.chars().filter(|c| !c.is_whitespace()).collect();
                if pattern_str.is_empty() {
                    return (max_end_time, cursor_time);
                }

                let step_count = pattern_str.len() as f32;
                let total_bar = 4.0 * base_duration;
                let step_duration = total_bar / step_count;

                // extract optional swing/humanize from pattern_stmt.value
                let mut swing: f32 = 0.0;
                let mut humanize: f32 = 0.0;
                if let Value::Map(m) = &pattern_stmt.value {
                    if let Some(Value::Number(s)) = m.get("swing") {
                        swing = *s;
                    }
                    if let Some(Value::Number(h)) = m.get("humanize") {
                        humanize = *h;
                    }
                }

                let mut updated_max = max_end_time;

                for (i, ch) in pattern_str.chars().enumerate() {
                    if ch == '-' {
                        continue;
                    }

                    // Apply swing: shift every other step by +/- swing*step_duration
                    let mut event_time = cursor_time + (i as f32) * step_duration;
                    if swing.abs() > 0.0001 {
                        // swing applies to off-beats: shift odd steps forward, even steps back
                        if i % 2 == 1 {
                            event_time += swing * step_duration;
                        } else {
                            event_time -= swing * step_duration;
                        }
                    }

                    // Apply humanize: small random jitter in [-humanize*step_duration/2, +...]
                    if humanize.abs() > 0.0001 {
                        let jitter_range = humanize * step_duration;
                        // lightweight RNG using a simple hash to avoid adding rand dependency
                        let seed = (audio_engine.module_name.len() + i) as u64
                            + (event_time.to_bits() as u64);
                        let mut x = seed.wrapping_mul(0x9E3779B97F4A7C15).rotate_left(13);
                        x ^= x >> 7;
                        let r = (x as i64 % 1000) as f32 / 1000.0; // [0,1)
                        let jitter = (r * 2.0 - 1.0) * jitter_range / 2.0;
                        event_time += jitter;
                    }

                    let mut trigger_val = Value::String(target_entity.clone());
                    if let Some(val) = variable_table.variables.get(&target_entity) {
                        match val {
                            Value::Identifier(id) => {
                                if let Some(parent) = &variable_table.parent {
                                    if let Some(v) = parent.get(id) {
                                        trigger_val = v.clone();
                                    }
                                } else if let Some(v) = variable_table.get(id) {
                                    trigger_val = v.clone();
                                }
                            }
                            Value::Map(map) => {
                                if let Some(Value::String(src)) = map.get("entity") {
                                    trigger_val = Value::String(src.clone());
                                } else if let Some(Value::Identifier(src)) = map.get("entity") {
                                    trigger_val = Value::Identifier(src.clone());
                                }
                            }
                            Value::Sample(sample_src) => {
                                trigger_val = Value::Sample(sample_src.clone());
                            }
                            _ => {}
                        }
                    }

                    let (src, sample_length) = crate::core::audio::loader::trigger::load_trigger(
                        &trigger_val,
                        &Duration::Number(step_duration),
                        &None,
                        base_duration,
                        final_variable_table.clone(),
                    );

                    let play_length = step_duration.min(sample_length);

                    let trigger_src = match trigger_val.get("entity") {
                        Some(Value::String(s)) => s.clone(),
                        Some(Value::Identifier(id)) => id.clone(),
                        Some(Value::Statement(stmt)) => {
                            if let StatementKind::Trigger { entity, .. } = &stmt.kind {
                                entity.clone()
                            } else {
                                src.clone()
                            }
                        }
                        _ => src.clone(),
                    };

                    audio_engine.insert_sample(
                        &trigger_src,
                        event_time,
                        play_length,
                        None,
                        &final_variable_table,
                    );

                    let end_time = event_time + play_length;
                    if end_time > updated_max {
                        updated_max = end_time;
                    }
                }

                audio_engine.merge_with(local_engine);
                return (updated_max.max(max_end_time), cursor_time);
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

fn find_pattern(
    name: &str,
    variable_table: &VariableTable,
    global_store: &GlobalStore,
) -> Option<Statement> {
    use crate::core::parser::statement::Statement;
    use crate::core::parser::statement::StatementKind;

    if let Some(Value::Statement(stmt_box)) = variable_table.get(name) {
        if let StatementKind::Pattern { .. } = stmt_box.kind {
            return Some(*stmt_box.clone());
        }
    }

    if let Some(val) = global_store.variables.variables.get(name) {
        match val {
            Value::Statement(stmt_box) => {
                if let StatementKind::Pattern { .. } = stmt_box.kind {
                    return Some(*stmt_box.clone());
                }
            }
            Value::Map(map) => {
                if let Some(Value::String(_pat)) = map.get("pattern") {
                    // Rebuild a Pattern statement from stored map if possible
                    let stmt = Statement {
                        kind: StatementKind::Pattern {
                            name: name.to_string(),
                            target: map.get("target").and_then(|v| match v {
                                Value::String(s) => Some(s.clone()),
                                _ => None,
                            }),
                        },
                        value: Value::String(
                            map.get("pattern")
                                .and_then(|v| match v {
                                    Value::String(s) => Some(s.clone()),
                                    _ => None,
                                })
                                .unwrap_or_default(),
                        ),
                        indent: 0,
                        line: 0,
                        column: 0,
                    };
                    return Some(stmt);
                }
            }
            _ => {}
        }
    }

    None
}
