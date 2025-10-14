// moved to driver/ as child module
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;

use crate::engine::audio::events::AudioEventList;
use crate::engine::events::EventHandler;
use crate::engine::events::EventRegistry;
use crate::engine::functions::FunctionRegistry;
use crate::language::syntax::ast::{Statement, StatementKind, Value};

use super::AudioInterpreter;

// Conditional logging macros for CLI feature
#[cfg(feature = "cli")]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(format!($($arg)*))
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_info {
    ($_logger:expr, $($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

#[cfg(feature = "cli")]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warn(format!($($arg)*))
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_warn {
    ($_logger:expr, $($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

#[cfg(feature = "cli")]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(format!($($arg)*))
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_error {
    ($_logger:expr, $($arg:tt)*) => {
        let _ = ($($arg)*);
    };
}

pub fn collect_events(interpreter: &mut AudioInterpreter, statements: &[Statement]) -> Result<()> {
    #[cfg(feature = "cli")]
    let logger = crate::tools::logger::Logger::new();
    #[cfg(not(feature = "cli"))]
    let _logger = ();
    // (content copied from top-level collector.rs)
    let (spawns, others): (Vec<_>, Vec<_>) = statements
        .iter()
        .partition(|stmt| matches!(stmt.kind, StatementKind::Spawn { .. }));

    for stmt in &others {
        interpreter.current_statement_location = Some((stmt.line, stmt.column));
        interpreter
            .special_vars
            .update_time(interpreter.cursor_time);

        match &stmt.kind {
            StatementKind::Let { name, value } => {
                if let Some(val) = value {
                    super::handler::handle_let(interpreter, name, val)?;
                }
            }
            StatementKind::ArrowCall {
                target,
                method,
                args,
            } => {
                let chain = if let Value::Map(map) = &stmt.value {
                    map.get("chain").and_then(|v| {
                        if let Value::Array(arr) = v {
                            Some(arr.as_slice())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let context =
                    crate::engine::audio::interpreter::statements::arrow_call::execute_arrow_call(
                        &interpreter.function_registry,
                        target,
                        method,
                        args,
                        chain,
                        interpreter.cursor_time,
                        interpreter.bpm,
                    )
                    .map_err(|e| {
                        #[cfg(feature = "wasm")]
                        {
                            use crate::web::registry::debug;
                            if debug::is_debug_errors_enabled() {
                                debug::push_parse_error_from_parts(
                                    format!("{}", e),
                                    stmt.line,
                                    stmt.column,
                                    "RuntimeError".to_string(),
                                );
                            }
                        }
                        e
                    })?;

                super::extractor::extract_audio_event(interpreter, target, &context)?;
                interpreter.cursor_time += context.duration;
            }
            StatementKind::Tempo => {
                if let Value::Number(bpm_value) = &stmt.value {
                    interpreter.set_bpm(*bpm_value);
                }
            }
            StatementKind::Sleep => {
                // Accept either a raw number (ms) or a Duration value (beats/fraction)
                match &stmt.value {
                    Value::Number(n) => {
                        interpreter.cursor_time += n / 1000.0;
                    }
                    Value::Duration(dv) => {
                        // convert duration value to seconds
                        let secs = match dv {
                            crate::language::syntax::ast::DurationValue::Milliseconds(ms) => {
                                Some(ms / 1000.0)
                            }
                            crate::language::syntax::ast::DurationValue::Beats(b) => {
                                Some(b * (60.0 / interpreter.bpm))
                            }
                            crate::language::syntax::ast::DurationValue::Beat(s) => {
                                // parse a fraction like "3/4" into beats
                                if let Some((num, den)) = {
                                    let mut sp = s.split('/');
                                    if let (Some(a), Some(b)) = (sp.next(), sp.next()) {
                                        if let (Ok(an), Ok(bn)) =
                                            (a.trim().parse::<f32>(), b.trim().parse::<f32>())
                                        {
                                            Some((an, bn))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } {
                                    if den.abs() > f32::EPSILON {
                                        Some((num / den) * (60.0 / interpreter.bpm))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            crate::language::syntax::ast::DurationValue::Number(n) => {
                                Some(n / 1000.0)
                            }
                            _ => None,
                        };
                        if let Some(s) = secs {
                            interpreter.cursor_time += s;
                        }
                    }
                    _ => {}
                }
            }
            StatementKind::Group { name, body } => {
                interpreter.groups.insert(name.clone(), body.clone());
            }
            StatementKind::Call { name, args: _ } => {
                // If this call contains an inline pattern (parser stores it in stmt.value as a Map
                // with `inline_pattern = true`), register it as a Pattern statement in variables
                // so that handle_call can find and execute it targeting the correct bank.trigger.
                if let Value::Map(map) = &stmt.value {
                    if let Some(Value::Boolean(inline)) = map.get("inline_pattern") {
                        if *inline {
                            let target = map.get("target").and_then(|v| {
                                if let Value::String(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            });

                            let pattern_stmt = Statement {
                                kind: StatementKind::Pattern {
                                    name: name.clone(),
                                    target: target.clone(),
                                },
                                value: stmt.value.clone(),
                                indent: stmt.indent,
                                line: stmt.line,
                                column: stmt.column,
                            };

                            interpreter
                                .variables
                                .insert(name.clone(), Value::Statement(Box::new(pattern_stmt)));
                        }
                    }
                }

                super::handler::handle_call(interpreter, name)?;
            }

            StatementKind::Spawn { .. } => {
                unreachable!("Spawn statements should be handled in parallel section");
            }
            StatementKind::Loop { count, body } => {
                interpreter.execute_loop(count, body)?;
            }
            StatementKind::For {
                variable,
                iterable,
                body,
            } => {
                interpreter.execute_for(variable, iterable, body)?;
            }
            StatementKind::Print => {
                super::handler::execute_print(interpreter, &stmt.value)?;
            }
            StatementKind::If {
                condition,
                body,
                else_body,
            } => {
                super::handler::execute_if(interpreter, condition, body, else_body)?;
            }
            StatementKind::On { event, args, body } => {
                let once = args
                    .as_ref()
                    .and_then(|a| a.first())
                    .and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s == "once")
                        } else {
                            None
                        }
                    })
                    .unwrap_or(false);
                // Normalize event name (strip trailing ':' if present)
                let event_name = event.trim_end_matches(':').trim().to_string();
                let handler = EventHandler {
                    event_name: event_name.clone(),
                    body: body.clone(),
                    once,
                };
                interpreter.event_registry.register_handler(handler);
            }
            StatementKind::Emit { event, payload } => {
                let data = if let Some(Value::Map(map)) = payload {
                    map.clone()
                } else {
                    HashMap::new()
                };
                interpreter
                    .event_registry
                    .emit(event.clone(), data, interpreter.cursor_time);
                super::handler::execute_event_handlers(interpreter, event)?;
            }
            StatementKind::Assign { target, property } => {
                super::handler::handle_assign(interpreter, target, property, &stmt.value)?;
            }
            StatementKind::Load { source, alias } => {
                super::handler::handle_load(interpreter, source, alias)?;
            }
            #[cfg(feature = "cli")]
            StatementKind::UsePlugin {
                author,
                name,
                alias,
            } => {
                super::handler::handle_use_plugin(interpreter, author, name, alias)?;
            }
            #[cfg(not(feature = "cli"))]
            StatementKind::UsePlugin { author, name, .. } => {
                eprintln!(
                    "⚠️  Plugin loading not supported in WASM mode: {}.{}",
                    author, name
                );
            }
            StatementKind::Pattern { name, target } => {
                let pattern_stmt = Statement {
                    kind: StatementKind::Pattern {
                        name: name.clone(),
                        target: target.clone(),
                    },
                    value: stmt.value.clone(),
                    indent: stmt.indent,
                    line: stmt.line,
                    column: stmt.column,
                };
                interpreter
                    .variables
                    .insert(name.clone(), Value::Statement(Box::new(pattern_stmt)));
            }
            StatementKind::Bank { name, alias } => {
                super::handler::handle_bank(interpreter, name, alias)?;
            }
            StatementKind::Bind { source, target } => {
                super::handler::handle_bind(interpreter, source, target, &stmt.value)?;
            }
            StatementKind::Import { names, source } => {
                let path = std::path::Path::new(&source);
                if path.exists() {
                    match crate::language::preprocessor::loader::load_module_exports(path) {
                        Ok(exports) => {
                            for name in names {
                                if let Some(val) = exports.variables.get(name) {
                                    interpreter.variables.insert(name.clone(), val.clone());
                                } else if let Some(group_body) = exports.groups.get(name) {
                                    interpreter.groups.insert(name.clone(), group_body.clone());
                                } else if let Some(pattern_stmt) = exports.patterns.get(name) {
                                    interpreter.variables.insert(
                                        name.clone(),
                                        Value::Statement(Box::new(pattern_stmt.clone())),
                                    );
                                } else {
                                    log_warn!(logger, "Import: '{}' not found in {}", name, source);
                                }
                            }
                        }
                        Err(e) => {
                            log_warn!(logger, "Failed to load module {}: {}", source, e);
                        }
                    }
                } else {
                    log_warn!(logger, "Import path not found: {}", source);
                }
            }
            StatementKind::Trigger {
                entity,
                duration: _,
                effects: _,
            } => {
                super::handler::handle_trigger(interpreter, entity)?;
            }
            _ => {}
        }
    }

    if !spawns.is_empty() {
        // ============================================================================
        // SPAWN EXECUTION (Parallel)
        // ============================================================================
        // Spawn accepts:
        // - Groups (defined with `group name:`)
        // - Patterns (defined with `pattern name with trigger = "x---"`)
        // - Samples (variables containing sample URIs)
        // - Bank properties (e.g., `spawn myBank.kick`)
        // ============================================================================

        // Executing spawns in parallel
        let current_time = interpreter.cursor_time;
        let current_bpm = interpreter.bpm;
        let groups_snapshot = interpreter.groups.clone();
        let variables_snapshot = interpreter.variables.clone();
        let special_vars_snapshot = interpreter.special_vars.clone();

        let spawn_results: Vec<Result<AudioEventList>> = spawns
            .par_iter()
            .map(|stmt| {
                if let StatementKind::Spawn { name, args: _ } = &stmt.kind {
                    let resolved_name = if name.starts_with('.') {
                        &name[1..]
                    } else {
                        name
                    };
                    if resolved_name.contains('.') {
                        let parts: Vec<&str> = resolved_name.split('.').collect();
                        if parts.len() == 2 {
                            let (var_name, property) = (parts[0], parts[1]);
                            if let Some(Value::Map(map)) = variables_snapshot.get(var_name) {
                                if let Some(Value::String(sample_uri)) = map.get(property) {
                                    // Spawn nested sample
                                    let mut event_list = AudioEventList::new();
                                    event_list.add_sample_event(
                                        sample_uri.trim_matches('"').trim_matches('\''),
                                        current_time,
                                        1.0,
                                    );
                                    return Ok(event_list);
                                }
                            }
                        }
                    }
                    if let Some(sample_value) = variables_snapshot.get(resolved_name) {
                        if let Value::String(sample_uri) = sample_value {
                            // Spawn sample
                            let mut event_list = AudioEventList::new();
                            event_list.add_sample_event(
                                sample_uri.trim_matches('"').trim_matches('\''),
                                current_time,
                                1.0,
                            );
                            return Ok(event_list);
                        }
                    }

                    let mut local_interpreter = AudioInterpreter {
                        sample_rate: interpreter.sample_rate,
                        bpm: current_bpm,
                        function_registry: FunctionRegistry::new(),
                        events: AudioEventList::new(),
                        variables: variables_snapshot.clone(),
                        groups: groups_snapshot.clone(),
                        banks: interpreter.banks.clone(),
                        cursor_time: current_time,
                        special_vars: special_vars_snapshot.clone(),
                        event_registry: EventRegistry::new(),
                        #[cfg(feature = "cli")]
                        midi_manager: interpreter.midi_manager.clone(),
                        current_statement_location: None,
                        suppress_beat_emit: interpreter.suppress_beat_emit,
                    };

                    // Inherit synth definitions from parent so spawned groups can snapshot synths/plugins
                    local_interpreter.events.synths = interpreter.events.synths.clone();

                    // Try to spawn a group first
                    if let Some(body) = groups_snapshot.get(resolved_name) {
                        // Spawn group (parallel)
                        log_info!(
                            logger,
                            "Spawning group '{}' with {} synths inherited",
                            resolved_name,
                            local_interpreter.events.synths.len()
                        );
                        collect_events(&mut local_interpreter, body)?;
                        log_info!(
                            logger,
                            "Group '{}' produced {} events, {} synths",
                            resolved_name,
                            local_interpreter.events.events.len(),
                            local_interpreter.events.synths.len()
                        );
                        Ok(local_interpreter.events)
                    }
                    // Try to spawn a pattern
                    else if let Some(pattern_value) = variables_snapshot.get(resolved_name) {
                        if let Value::Statement(stmt_box) = pattern_value {
                            if let StatementKind::Pattern { target, .. } = &stmt_box.kind {
                                if let Some(tgt) = target.as_ref() {
                                    let (pattern_str, options) =
                                        local_interpreter.extract_pattern_data(&stmt_box.value);
                                    if let Some(pat) = pattern_str {
                                        // Execute pattern in spawned context
                                        log_info!(
                                            logger,
                                            "Spawning pattern '{}' on target '{}'",
                                            resolved_name,
                                            tgt
                                        );
                                        local_interpreter.execute_pattern(
                                            tgt.as_str(),
                                            &pat,
                                            options,
                                        )?;
                                        log_info!(
                                            logger,
                                            "Pattern '{}' produced {} events",
                                            resolved_name,
                                            local_interpreter.events.events.len()
                                        );
                                        return Ok(local_interpreter.events);
                                    }
                                }
                            }
                        }
                        // If not a pattern, warn
                        log_warn!(
                            logger,
                            "Spawn target '{}' is not a group or pattern",
                            resolved_name
                        );
                        Ok(AudioEventList::new())
                    } else {
                        log_warn!(
                            logger,
                            "Spawn target '{}' not found (neither sample, group, nor pattern)",
                            resolved_name
                        );
                        Ok(AudioEventList::new())
                    }
                } else {
                    Ok(AudioEventList::new())
                }
            })
            .collect();

        for result in spawn_results {
            match result {
                Ok(spawn_events) => {
                    log_info!(
                        logger,
                        "Merging spawn result: {} events, {} synths",
                        spawn_events.events.len(),
                        spawn_events.synths.len()
                    );
                    interpreter.events.merge(spawn_events);
                    log_info!(
                        logger,
                        "Total after merge: {} events, {} synths",
                        interpreter.events.events.len(),
                        interpreter.events.synths.len()
                    );
                }
                Err(e) => {
                    log_error!(logger, "Error in spawn execution: {}", e);
                }
            }
        }
    }

    // Emit some built-in beat events to trigger 'on beat' handlers.
    // Use the interpreter.special_vars.total_duration when available, but limit to a small number
    // of beats to avoid excessively long renders during tests.
    {
        // Emit beat events at exact beat grid (0, beat, 2*beat, ...) up to total duration.
        // We temporarily set interpreter.cursor_time and special_vars so handlers add events at
        // the correct beat timestamps, then we restore the original cursor_time afterwards.
        if !interpreter.suppress_beat_emit {
            let beat_duration = 60.0 / interpreter.bpm;

            // Determine total duration to cover. Prefer special_vars.total_duration if set.
            let total = if interpreter.special_vars.total_duration > 0.0 {
                interpreter.special_vars.total_duration
            } else {
                // Fallback: cover at least 4 beats
                beat_duration * 4.0
            };

            let max_beats = ((total / beat_duration).ceil() as usize).min(256).max(1);

            // Save previous state
            let prev_cursor = interpreter.cursor_time;
            let prev_time = interpreter.special_vars.current_time;
            let prev_beat = interpreter.special_vars.current_beat;

            for i in 0..max_beats {
                let beat_time = i as f32 * beat_duration;
                interpreter.cursor_time = beat_time;
                interpreter.special_vars.current_time = beat_time;
                interpreter.special_vars.current_beat = beat_time / beat_duration;

                // execute handlers for 'beat' but ensure handlers won't re-emit beats by setting the guard
                let prev = interpreter.suppress_beat_emit;
                interpreter.suppress_beat_emit = true;
                interpreter.execute_event_handlers("beat")?;
                interpreter.suppress_beat_emit = prev;
            }

            // Restore previous state
            interpreter.cursor_time = prev_cursor;
            interpreter.special_vars.current_time = prev_time;
            interpreter.special_vars.current_beat = prev_beat;
        }
    }

    Ok(())
}
