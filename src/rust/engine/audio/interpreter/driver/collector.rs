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

#[cfg(feature = "cli")]
macro_rules! log_structured_error {
    ($logger:expr, $error:expr) => {
        $logger.log_structured_error(&$error)
    };
}

#[cfg(not(feature = "cli"))]
macro_rules! log_structured_error {
    ($_logger:expr, $_error:expr) => {};
}

pub fn collect_events(interpreter: &mut AudioInterpreter, statements: &[Statement]) -> Result<()> {
    #[cfg(feature = "cli")]
    let logger = crate::tools::logger::Logger::new();
    #[cfg(not(feature = "cli"))]
    let _logger = ();
    // (content copied from top-level collector.rs)
    // Drain any background worker events first so they are available during collection
    if let Some(rx) = interpreter.background_event_rx.as_mut() {
        // Drain all pending lists
        loop {
            match rx.try_recv() {
                Ok(events) => {
                    // Debug info: show how many events are being merged from background workers
                    let _cnt = events.events.len();
                    // Consider both audio events and logged print messages when computing earliest time
                    let mut times: Vec<f32> = Vec::new();
                    for e in &events.events {
                        if let crate::engine::audio::events::AudioEvent::Note {
                            start_time, ..
                        } = e
                        {
                            times.push(*start_time);
                        } else if let crate::engine::audio::events::AudioEvent::Chord {
                            start_time,
                            ..
                        } = e
                        {
                            times.push(*start_time);
                        } else if let crate::engine::audio::events::AudioEvent::Sample {
                            start_time,
                            ..
                        } = e
                        {
                            times.push(*start_time);
                        }
                    }
                    for (t, _m) in &events.logs {
                        times.push(*t);
                    }
                    let earliest = times
                        .into_iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let _ = earliest;
                    interpreter.events.merge(events);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
    }
    let (spawns, others): (Vec<_>, Vec<_>) = statements
        .iter()
        .partition(|stmt| matches!(stmt.kind, StatementKind::Spawn { .. }));

    // Iterate with index so we can inspect the remaining statements for global-automation span
    for idx in 0..others.len() {
        let stmt = &others[idx];
        interpreter.current_statement_location = Some((stmt.line, stmt.column));
        interpreter
            .special_vars
            .update_time(interpreter.cursor_time);

        match &stmt.kind {
            StatementKind::Function {
                name,
                parameters,
                body,
            } => {
                // Register function definition in variables so it can be called later
                let func_stmt = Statement {
                    kind: StatementKind::Function {
                        name: name.clone(),
                        parameters: parameters.clone(),
                        body: body.clone(),
                    },
                    value: stmt.value.clone(),
                    indent: stmt.indent,
                    line: stmt.line,
                    column: stmt.column,
                };
                interpreter
                    .variables
                    .insert(name.clone(), Value::Statement(Box::new(func_stmt)));
            }
            StatementKind::Let { name, value } => {
                if let Some(val) = value {
                    super::handler::handle_let(interpreter, name, val)?;
                }
            }
            StatementKind::Const { name, value } => {
                // Treat const like let at runtime: register the value in the interpreter variables.
                // Immutability is enforced at higher language layers; runtime simply stores the value.
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
                        interpreter,
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
                                // wasm registry expects: message, line, column, error_type
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
            StatementKind::Tempo { value, body } => {
                let prev_bpm = interpreter.bpm;
                interpreter.set_bpm(*value);

                // If this is a block, execute its body with the new tempo
                if let Some(block_body) = body {
                    collect_events(interpreter, block_body)?;
                }

                // If body is None (simple tempo declaration), keep the new BPM
                // Otherwise, restore the previous BPM after the block completes
                if body.is_some() {
                    interpreter.bpm = prev_bpm;
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
            StatementKind::Routing { body } => {
                // Process routing block - parse nodes, fx, routes, ducks, and sidechains
                for routing_stmt in body {
                    match &routing_stmt.kind {
                        StatementKind::RoutingNode { name, alias } => {
                            let config = super::RoutingNodeConfig {
                                name: name.clone(),
                                alias: alias.clone(),
                                effects: None,
                            };
                            interpreter.routing.nodes.insert(name.clone(), config);
                        }
                        StatementKind::RoutingFx { target, effects } => {
                            if let Some(node_config) = interpreter.routing.nodes.get_mut(target) {
                                node_config.effects = Some(effects.clone());
                            }
                        }
                        StatementKind::RoutingRoute {
                            source,
                            destination,
                            effects,
                        } => {
                            interpreter.routing.routes.push(super::RouteConfig {
                                source: source.clone(),
                                destination: destination.clone(),
                                effects: effects.clone(),
                            });
                        }
                        StatementKind::RoutingDuck {
                            source,
                            destination,
                            effect,
                        } => {
                            interpreter.routing.ducks.push(super::DuckConfig {
                                source: source.clone(),
                                destination: destination.clone(),
                                effect: effect.clone(),
                            });
                        }
                        StatementKind::RoutingSidechain {
                            source,
                            destination,
                            effect,
                        } => {
                            interpreter.routing.sidechains.push(super::SidechainConfig {
                                source: source.clone(),
                                destination: destination.clone(),
                                effect: effect.clone(),
                            });
                        }
                        _ => {}
                    }
                }

                // Build the audio graph from routing configuration
                interpreter.audio_graph =
                    crate::engine::audio::interpreter::AudioGraph::from_routing_setup(
                        &interpreter.routing,
                    );
            }
            StatementKind::Call { name, args } => {
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

                super::handler::handle_call(interpreter, name, args)?;
            }

            StatementKind::Automate { target } => {
                // Expect stmt.value to be a Map with keys: "mode" (optional) and "body" (raw string)
                if let Value::Map(map) = &stmt.value {
                    let mode = map
                        .get("mode")
                        .and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "global".to_string());

                    if let Some(Value::String(raw_body)) = map.get("body") {
                        // Parse templates
                        let templates =
                            crate::engine::audio::automation::parse_param_templates_from_raw(
                                raw_body,
                            );

                        if mode == "note" {
                            // For note mode, we need to estimate the duration of the block
                            // so that per-note automations can calculate their progress properly
                            let remaining: Vec<crate::language::syntax::ast::Statement> =
                                others[idx + 1..].iter().map(|s| (*s).clone()).collect();

                            // Create a local interpreter snapshot to measure duration
                            let current_bpm = interpreter.bpm;
                            let groups_snapshot = interpreter.groups.clone();
                            let variables_snapshot = interpreter.variables.clone();
                            let special_vars_snapshot = interpreter.special_vars.clone();

                            let mut local_interpreter = AudioInterpreter {
                                sample_rate: interpreter.sample_rate,
                                bpm: current_bpm,
                                function_registry: FunctionRegistry::new(),
                                events: AudioEventList::new(),
                                variables: variables_snapshot.clone(),
                                groups: groups_snapshot.clone(),
                                banks: interpreter.banks.clone(),
                                automation_registry: interpreter.automation_registry.clone(),
                                note_automation_templates: interpreter
                                    .note_automation_templates
                                    .clone(),
                                cursor_time: 0.0,
                                special_vars: special_vars_snapshot.clone(),
                                event_registry: EventRegistry::new(),
                                #[cfg(feature = "cli")]
                                midi_manager: interpreter.midi_manager.clone(),
                                current_statement_location: None,
                                suppress_beat_emit: true,
                                suppress_print: true,
                                break_flag: false,
                                // Inherit background_event_tx from parent so spawned/child
                                // interpreters reuse the same Sender when running under
                                // live playback. This prevents child interpreters from
                                // creating their own private Receiver that would be
                                // dropped while background workers keep running.
                                background_event_tx: interpreter.background_event_tx.clone(),
                                background_event_rx: None,
                                background_workers: Vec::new(),
                                realtime_print_tx: None,
                                function_call_depth: 0,
                                returning_flag: false,
                                return_value: None,
                                routing: Default::default(),
                                audio_graph: crate::engine::audio::interpreter::AudioGraph::new(),
                            };

                            // Inherit synth definitions
                            local_interpreter.events.synths = interpreter.events.synths.clone();

                            // Simulate to measure duration
                            if !remaining.is_empty() {
                                let _ = collect_events(&mut local_interpreter, &remaining);
                            }

                            let end_time =
                                interpreter.cursor_time + local_interpreter.events.total_duration();

                            // Create context with timing information
                            let context = super::NoteAutomationContext {
                                templates,
                                start_time: interpreter.cursor_time,
                                end_time,
                            };

                            // Store context under target for per-note application
                            interpreter
                                .note_automation_templates
                                .insert(target.clone(), context);
                        } else {
                            // Global mode: determine a sensible total duration for the envelope.
                            // We'll estimate it by simulating the remaining statements in this block
                            // (i.e., the statements after this automate statement in the current list).
                            let remaining: Vec<crate::language::syntax::ast::Statement> =
                                others[idx + 1..].iter().map(|s| (*s).clone()).collect();

                            // Create a local interpreter snapshot to measure the total duration
                            let current_bpm = interpreter.bpm;
                            let groups_snapshot = interpreter.groups.clone();
                            let variables_snapshot = interpreter.variables.clone();
                            let special_vars_snapshot = interpreter.special_vars.clone();

                            let mut local_interpreter = AudioInterpreter {
                                sample_rate: interpreter.sample_rate,
                                bpm: current_bpm,
                                function_registry: FunctionRegistry::new(),
                                events: AudioEventList::new(),
                                variables: variables_snapshot.clone(),
                                groups: groups_snapshot.clone(),
                                banks: interpreter.banks.clone(),
                                automation_registry: interpreter.automation_registry.clone(),
                                note_automation_templates: interpreter
                                    .note_automation_templates
                                    .clone(),
                                cursor_time: 0.0,
                                special_vars: special_vars_snapshot.clone(),
                                event_registry: EventRegistry::new(),
                                #[cfg(feature = "cli")]
                                midi_manager: interpreter.midi_manager.clone(),
                                current_statement_location: None,
                                suppress_beat_emit: true,
                                suppress_print: true,
                                break_flag: false,
                                // Keep the same background sender as the parent interpreter
                                background_event_tx: interpreter.background_event_tx.clone(),
                                background_event_rx: None,
                                background_workers: Vec::new(),
                                realtime_print_tx: None,
                                function_call_depth: 0,
                                returning_flag: false,
                                return_value: None,
                                routing: Default::default(),
                                audio_graph: crate::engine::audio::interpreter::AudioGraph::new(),
                            };

                            // Inherit synth definitions so durations reflect real events
                            local_interpreter.events.synths = interpreter.events.synths.clone();

                            // Simulate collecting events for the remaining statements to estimate duration
                            if !remaining.is_empty() {
                                let _ = collect_events(&mut local_interpreter, &remaining);
                            }

                            let total_dur = local_interpreter.events.total_duration();
                            let start_time = interpreter.cursor_time;

                            let mut envelope =
                                crate::engine::audio::automation::AutomationEnvelope::new(
                                    target.clone(),
                                );

                            // For each template, create AutomationParam segments between adjacent points
                            for tpl in templates.iter() {
                                if tpl.points.len() >= 2 {
                                    for w in tpl.points.windows(2) {
                                        let (p0, v0) = w[0];
                                        let (p1, v1) = w[1];
                                        let seg_start = start_time + p0 * total_dur;
                                        let seg_dur = (p1 - p0) * total_dur;
                                        if seg_dur <= 0.0 {
                                            continue;
                                        }
                                        envelope.add_param(
                                            crate::engine::audio::automation::AutomationParam {
                                                param_name: tpl.param_name.clone(),
                                                from_value: v0,
                                                to_value: v1,
                                                start_time: seg_start,
                                                duration: seg_dur,
                                                curve: tpl.curve,
                                            },
                                        );
                                    }
                                } else if tpl.points.len() == 1 {
                                    // Single point - treat as immediate set at that fraction
                                    let (p, v) = tpl.points[0];
                                    let seg_start = start_time + p * total_dur;
                                    envelope.add_param(
                                        crate::engine::audio::automation::AutomationParam {
                                            param_name: tpl.param_name.clone(),
                                            from_value: v,
                                            to_value: v,
                                            start_time: seg_start,
                                            duration: 0.0,
                                            curve: tpl.curve,
                                        },
                                    );
                                }
                            }

                            // Register the automation envelope
                            interpreter.automation_registry.register(envelope);
                        }
                    }
                }
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
            StatementKind::Return { value } => {
                // Only allow 'return' inside a function call context
                if interpreter.function_call_depth == 0 {
                    return Err(anyhow::anyhow!(
                        "'return' used outside of a function at {}:{}",
                        stmt.line,
                        stmt.column
                    ));
                }

                // Resolve return value (if provided) and signal function return
                if let Some(vbox) = value.as_ref() {
                    let resolved = interpreter.resolve_value(vbox)?;
                    interpreter.return_value = Some(resolved);
                } else {
                    interpreter.return_value = Some(Value::Null);
                }
                interpreter.returning_flag = true;
                // Stop further execution of the current statement block
                return Ok(());
            }
            StatementKind::Break => {
                // Signal to enclosing loop/for that a break occurred
                interpreter.break_flag = true;
                // Stop processing this block so loop logic can handle the break
                return Ok(());
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
                // Build handler: pick up once flag and keep args (intervals etc.)
                let once = args
                    .as_ref()
                    .and_then(|a| {
                        a.iter().find_map(|v| {
                            if let Value::String(s) = v {
                                Some(s == "once")
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or(false);

                // Normalize event name (strip trailing ':' if present)
                let event_name = event.trim_end_matches(':').trim().to_string();
                let handler = EventHandler {
                    event_name: event_name.clone(),
                    body: body.clone(),
                    once,
                    args: args.clone(),
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
                effects,
            } => {
                // Pass the effects associated with the trigger statement into the handler so
                // runtime scheduling can attach them to sample events.
                super::handler::handle_trigger(interpreter, entity, effects.as_ref())?;
            }
            StatementKind::Unknown => {
                // Unknown statements are parser errors - log them with structured formatting
                if let Value::String(error_msg) = &stmt.value {
                    // Parse the structured error format: "MESSAGE|||FILE:LINE|||SUGGESTION"
                    let parts: Vec<&str> = error_msg.split("|||").collect();

                    let main_msg = parts
                        .get(0)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| error_msg.clone());
                    let file_location = parts.get(1).map(|s| s.to_string());
                    let suggestion_text = parts.get(2).and_then(|s| {
                        if s.is_empty() {
                            None
                        } else {
                            Some(format!("Did you mean '{}' ?", s))
                        }
                    });

                    // Create structured error with details
                    let mut structured_err = crate::tools::logger::StructuredError::new(&main_msg)
                        .with_location(stmt.line, stmt.column)
                        .with_type("UnknownStatement");

                    // Add file location if available
                    if let Some(file_loc) = file_location {
                        structured_err = structured_err.with_file(file_loc);
                    }

                    // Add suggestion if available
                    if let Some(suggest) = suggestion_text {
                        structured_err = structured_err.with_suggestion(suggest);
                    }

                    // Log the structured error
                    log_structured_error!(logger, structured_err);

                    // Optionally push to WASM error registry
                    #[cfg(feature = "wasm")]
                    {
                        use crate::web::registry::debug;
                        if debug::is_debug_errors_enabled() {
                            debug::push_parse_error_from_parts(
                                error_msg.clone(),
                                stmt.line,
                                stmt.column,
                                "UnknownStatement".to_string(),
                            );
                        }
                    }
                }
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
                        automation_registry: interpreter.automation_registry.clone(),
                        note_automation_templates: interpreter.note_automation_templates.clone(),
                        cursor_time: current_time,
                        special_vars: special_vars_snapshot.clone(),
                        event_registry: EventRegistry::new(),
                        #[cfg(feature = "cli")]
                        midi_manager: interpreter.midi_manager.clone(),
                        current_statement_location: None,
                        suppress_beat_emit: interpreter.suppress_beat_emit,
                        suppress_print: interpreter.suppress_print,
                        break_flag: false,
                        // Ensure spawned local interpreters inherit the parent's
                        // background sender when present. This avoids creating
                        // ephemeral receivers that would be dropped and cause
                        // background workers to observe "receiver closed".
                        background_event_tx: interpreter.background_event_tx.clone(),
                        background_event_rx: None,
                        background_workers: Vec::new(),
                        realtime_print_tx: None,
                        function_call_depth: 0,
                        returning_flag: false,
                        return_value: None,
                        routing: Default::default(),
                        audio_graph: crate::engine::audio::interpreter::AudioGraph::new(),
                    };

                    // Inherit synth definitions from parent so spawned groups can snapshot synths/plugins
                    local_interpreter.events.synths = interpreter.events.synths.clone();

                    // Try to spawn a group first
                    if let Some(body) = groups_snapshot.get(resolved_name) {
                        // Spawn group (parallel)
                        collect_events(&mut local_interpreter, body)?;
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
                                        local_interpreter.execute_pattern(
                                            tgt.as_str(),
                                            &pat,
                                            options,
                                        )?;
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
                    interpreter.events.merge(spawn_events);
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
                // Update special vars consistently (updates beat and bar)
                interpreter.special_vars.update_time(beat_time);

                // execute handlers for 'beat' but ensure handlers won't re-emit beats by setting the guard
                let prev = interpreter.suppress_beat_emit;
                interpreter.suppress_beat_emit = true;
                interpreter.execute_event_handlers("beat")?;
                // Also call 'bar' handlers at bar boundaries (every 4 beats)
                if (i % 4) == 0 {
                    interpreter.execute_event_handlers("bar")?;
                }
                interpreter.suppress_beat_emit = prev;
            }

            // Restore previous state
            interpreter.cursor_time = prev_cursor;
            interpreter.special_vars.current_time = prev_time;
            interpreter.special_vars.current_beat = prev_beat;
            // restore bar too
            interpreter.special_vars.current_bar = prev_beat / 4.0;
        }
    }

    // Do not perform a blocking drain here — background 'pass' workers will deliver
    // their events asynchronously. Blocking here caused noticeable latency before
    // playback in some offline render scenarios, so we avoid waiting and let the
    // normal event merge logic handle incoming background batches.

    Ok(())
}
