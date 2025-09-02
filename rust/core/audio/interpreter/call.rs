use devalang_types::{Duration, Value};

use crate::core::{
    audio::{engine::AudioEngine, interpreter::driver::execute_audio_block},
    parser::statement::{Statement, StatementKind},
    store::{function::FunctionTable, global::GlobalStore, variable::VariableTable},
};

pub fn interprete_call_statement(
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
    if let StatementKind::Call { name, args } = &stmt.kind {
        // Classic function call case
        if let Some(func) = functions.functions.get(name) {
            // function found
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

            return execute_audio_block(
                audio_engine,
                global_store,
                local_vars,
                functions.clone(),
                &func.body,
                base_bpm,
                base_duration,
                max_end_time,
                cursor_time,
            );
        }

        // Group case
        if let Some(group_stmt) = find_group(name, variable_table, global_store) {
            // group found
            if let Value::Map(map) = &group_stmt.value {
                if let Some(Value::Block(body)) = map.get("body") {
                    return execute_audio_block(
                        audio_engine,
                        global_store,
                        variable_table.clone(),
                        functions.clone(),
                        body,
                        base_bpm,
                        base_duration,
                        max_end_time,
                        cursor_time,
                    );
                }
            }
        }

        // Pattern case
        if let Some(pattern_stmt) = find_pattern(name, variable_table, global_store) {
            // Extract pattern string from statement value
            if let Value::String(pat) = &pattern_stmt.value {
                // Determine target entity (explicit or inferred)
                let mut target_entity = name.clone();
                if let StatementKind::Pattern { name: _n, target } = &pattern_stmt.kind {
                    if let Some(t) = target {
                        target_entity = t.clone();
                    }
                }

                // Build a variable table snapshot for resolution like triggers do
                // Preserve the full parent chain so lookups behave the same as runtime
                fn clone_with_parents(orig: &crate::core::store::variable::VariableTable) -> crate::core::store::variable::VariableTable {
                    crate::core::store::variable::VariableTable {
                        variables: orig.variables.clone(),
                        parent: orig.parent.as_ref().map(|p| Box::new(clone_with_parents(p))),
                    }
                }

                let final_variable_table = clone_with_parents(variable_table);

                // Normalize pattern: remove spaces and line breaks
                let pattern_str: String = pat.chars().filter(|c| !c.is_whitespace()).collect();
                if pattern_str.is_empty() {
                    return (max_end_time, cursor_time);
                }

                let step_count = pattern_str.len() as f32;
                // Assume pattern spans one bar (4 beats)
                let total_bar = 4.0 * base_duration;
                let step_duration = total_bar / step_count; // seconds per step

                let mut updated_max = max_end_time;

                for (i, ch) in pattern_str.chars().enumerate() {
                    if ch == '-' {
                        continue; // rest
                    }

                    // Schedule a trigger at cursor_time + offset
                    let event_time = cursor_time + (i as f32) * step_duration;

                    // Resolve trigger value similarly to interprete_trigger_statement
                    let mut trigger_val = Value::String(target_entity.clone());
                    if let Some(val) = variable_table.variables.get(&target_entity) {
                        match val {
                            Value::Identifier(id) => {
                                // resolve from parent if available
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
                            _ => {
                                // leave as string
                            }
                        }
                    }

                    // Use loader to get sample path and sample length
                    let (src, sample_length) =
                        crate::core::audio::loader::trigger::load_trigger(&trigger_val, &Duration::Number(step_duration), &None, base_duration, final_variable_table.clone());

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

                    audio_engine.insert_sample(&trigger_src, event_time, play_length, None, &final_variable_table);

                    let end_time = event_time + play_length;
                    if end_time > updated_max {
                        updated_max = end_time;
                    }
                }

                return (updated_max, cursor_time);
            }
        }

        // Function or group not found; keep as debug-free fail path
    }

    (max_end_time, cursor_time)
}

fn find_group(
    name: &str,
    variable_table: &VariableTable,
    global_store: &GlobalStore,
) -> Option<Statement> {
    use crate::core::parser::statement::Statement;
    use crate::core::parser::statement::StatementKind;

    if let Some(Value::Statement(stmt_box)) = variable_table.get(name) {
        if let StatementKind::Group = stmt_box.kind {
            return Some(*stmt_box.clone());
        }
    }

    if let Some(val) = global_store.variables.variables.get(name) {
        match val {
            Value::Statement(stmt_box) => {
                if let StatementKind::Group = stmt_box.kind {
                    return Some(*stmt_box.clone());
                }
            }
            Value::Map(map) => {
                // Try to rebuild a Group statement from the stored map
                if let (Some(Value::String(_id)), Some(Value::Block(_body))) =
                    (map.get("identifier"), map.get("body"))
                {
                    let stmt = Statement {
                        kind: StatementKind::Group,
                        value: Value::Map(map.clone()),
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
                        value: Value::String(map.get("pattern").and_then(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            _ => None,
                        }).unwrap_or_default()),
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
