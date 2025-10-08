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

pub fn collect_events(interpreter: &mut AudioInterpreter, statements: &[Statement]) -> Result<()> {
    // (content copied from top-level collector.rs)
    let (spawns, others): (Vec<_>, Vec<_>) = statements
        .iter()
        .partition(|stmt| matches!(stmt.kind, StatementKind::Spawn { .. }));

    for stmt in &others {
        interpreter.current_statement_location = Some((stmt.line, stmt.column));
        interpreter.special_vars.update_time(interpreter.cursor_time);

        match &stmt.kind {
            StatementKind::Let { name, value } => {
                if let Some(val) = value {
                    super::handler::handle_let(interpreter, name, val)?;
                }
            }
            StatementKind::ArrowCall { target, method, args } => {
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

                let context = crate::engine::audio::interpreter::statements::arrow_call::execute_arrow_call(
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
                                0,
                                "RuntimeError".to_string(),
                                "error".to_string(),
                            );
                        }
                    }
                    e
                })?;

                super::extractor::extract_audio_event(interpreter, target, &context)?;
                interpreter.cursor_time += context.duration;
            }
            StatementKind::Tempo => {
                if let Value::Number(bpm_value) = &stmt.value { interpreter.set_bpm(*bpm_value); }
            }
            StatementKind::Sleep => {
                if let Value::Number(duration) = &stmt.value { interpreter.cursor_time += duration / 1000.0; }
            }
            StatementKind::Group { name, body } => { interpreter.groups.insert(name.clone(), body.clone()); }
            StatementKind::Call { name, args: _ } => { super::handler::handle_call(interpreter, name)?; }
            StatementKind::Spawn { .. } => { unreachable!("Spawn statements should be handled in parallel section"); }
            StatementKind::Loop { count, body } => { interpreter.execute_loop(count, body)?; }
            StatementKind::For { variable, iterable, body } => { interpreter.execute_for(variable, iterable, body)?; }
            StatementKind::Print => { super::handler::execute_print(interpreter, &stmt.value)?; }
            StatementKind::If { condition, body, else_body } => { super::handler::execute_if(interpreter, condition, body, else_body)?; }
            StatementKind::On { event, args, body } => {
                let once = args.as_ref().and_then(|a| a.first()).and_then(|v| { if let Value::String(s) = v { Some(s == "once") } else { None } }).unwrap_or(false);
                // Normalize event name (strip trailing ':' if present)
                let event_name = event.trim_end_matches(':').trim().to_string();
                let handler = EventHandler { event_name: event_name.clone(), body: body.clone(), once };
                interpreter.event_registry.register_handler(handler);
                println!("📡 Event handler registered: {} (once={})", event_name, once);
            }
            StatementKind::Emit { event, payload } => {
                let data = if let Some(Value::Map(map)) = payload { map.clone() } else { HashMap::new() };
                interpreter.event_registry.emit(event.clone(), data, interpreter.cursor_time);
                super::handler::execute_event_handlers(interpreter, event)?;
                println!("📤 Event emitted: {}", event);
            }
            StatementKind::Assign { target, property } => { super::handler::handle_assign(interpreter, target, property, &stmt.value)?; }
            StatementKind::Load { source, alias } => { super::handler::handle_load(interpreter, source, alias)?; }
            #[cfg(feature = "cli")]
            StatementKind::UsePlugin { author, name, alias } => { super::handler::handle_use_plugin(interpreter, author, name, alias)?; }
            #[cfg(not(feature = "cli"))]
            StatementKind::UsePlugin { author, name, .. } => { eprintln!("⚠️  Plugin loading not supported in WASM mode: {}.{}", author, name); }
            StatementKind::Pattern { name, target } => {
                let pattern_stmt = Statement { kind: StatementKind::Pattern { name: name.clone(), target: target.clone() }, value: stmt.value.clone(), indent: stmt.indent, line: stmt.line, column: stmt.column };
                interpreter.variables.insert(name.clone(), Value::Statement(Box::new(pattern_stmt)));
                println!("📝 Pattern defined: {}", name);
            }
            StatementKind::Bank { name, alias } => { super::handler::handle_bank(interpreter, name, alias)?; }
            StatementKind::Bind { source, target } => { super::handler::handle_bind(interpreter, source, target, &stmt.value)?; }
            StatementKind::Import { names, source } => {
                let path = std::path::Path::new(&source);
                if path.exists() {
                    match crate::language::preprocessor::loader::load_module_exports(path) {
                        Ok(exports) => {
                            for name in names {
                                if let Some(val) = exports.variables.get(name) {
                                    interpreter.variables.insert(name.clone(), val.clone());
                                    println!("🔗 Imported variable '{}' from {}", name, source);
                                } else if let Some(group_body) = exports.groups.get(name) {
                                    interpreter.groups.insert(name.clone(), group_body.clone());
                                    println!("🔗 Imported group '{}' from {}", name, source);
                                } else if let Some(pattern_stmt) = exports.patterns.get(name) {
                                    interpreter.variables.insert(name.clone(), Value::Statement(Box::new(pattern_stmt.clone())));
                                    println!("🔗 Imported pattern '{}' from {}", name, source);
                                } else {
                                    println!("⚠️  Import: '{}' not found in {}", name, source);
                                }
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Failed to load module {}: {}", source, e);
                        }
                    }
                } else {
                    println!("⚠️  Import path not found: {}", source);
                }
            }
            StatementKind::Trigger { entity, duration: _, effects: _ } => { super::handler::handle_trigger(interpreter, entity)?; }
            _ => {}
        }
    }

    if !spawns.is_empty() {
        println!("🚀 Executing {} spawn(s) in parallel", spawns.len());
        let current_time = interpreter.cursor_time;
        let current_bpm = interpreter.bpm;
        let groups_snapshot = interpreter.groups.clone();
        let variables_snapshot = interpreter.variables.clone();
        let special_vars_snapshot = interpreter.special_vars.clone();

        let spawn_results: Vec<Result<AudioEventList>> = spawns.par_iter().map(|stmt| {
            if let StatementKind::Spawn { name, args: _ } = &stmt.kind {
                let resolved_name = if name.starts_with('.') { &name[1..] } else { name };
                if resolved_name.contains('.') {
                    let parts: Vec<&str> = resolved_name.split('.').collect();
                    if parts.len() == 2 {
                        let (var_name, property) = (parts[0], parts[1]);
                        if let Some(Value::Map(map)) = variables_snapshot.get(var_name) {
                            if let Some(Value::String(sample_uri)) = map.get(property) {
                                println!("🎵 Spawn nested sample: {}.{} -> {}", var_name, property, sample_uri);
                                let mut event_list = AudioEventList::new();
                                event_list.add_sample_event(sample_uri.trim_matches('"').trim_matches('\''), current_time, 1.0);
                                return Ok(event_list);
                            }
                        }
                    }
                }
                if let Some(sample_value) = variables_snapshot.get(resolved_name) {
                    if let Value::String(sample_uri) = sample_value {
                        println!("🎵 Spawn sample: {} -> {}", resolved_name, sample_uri);
                        let mut event_list = AudioEventList::new();
                        event_list.add_sample_event(sample_uri.trim_matches('"').trim_matches('\''), current_time, 1.0);
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
                    current_statement_location: None,
                    suppress_beat_emit: interpreter.suppress_beat_emit,
                };

                // Inherit synth definitions from parent so spawned groups can snapshot synths/plugins
                local_interpreter.events.synths = interpreter.events.synths.clone();

                    if let Some(body) = groups_snapshot.get(resolved_name) {
                        println!("🎬 Spawn group: {} (parallel)", resolved_name);
                        collect_events(&mut local_interpreter, body)?;
                        Ok(local_interpreter.events)
                    } else {
                    println!("⚠️  Warning: Spawn target '{}' not found (neither sample nor group)", resolved_name);
                    Ok(AudioEventList::new())
                }
            } else { Ok(AudioEventList::new()) }
        }).collect();

        for result in spawn_results {
            match result { Ok(spawn_events) => { interpreter.events.merge(spawn_events); }, Err(e) => { println!("⚠️  Error in spawn execution: {}", e); } }
        }

        println!("✅ Parallel spawn execution completed");
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
