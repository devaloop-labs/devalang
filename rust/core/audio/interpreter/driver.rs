use rayon::prelude::*;

use crate::{
    core::{
        audio::{
            engine::AudioEngine,
            interpreter::{
                arrow_call::interprete_call_arrow_statement, call::interprete_call_statement,
                function::interprete_function_statement, let_::interprete_let_statement,
                load::interprete_load_statement, loop_::interprete_loop_statement,
                sleep::interprete_sleep_statement, spawn::interprete_spawn_statement,
                tempo::interprete_tempo_statement, trigger::interprete_trigger_statement,
            },
        },
        parser::statement::{Statement, StatementKind},
        shared::value::Value,
        store::{function::FunctionTable, global::GlobalStore, variable::VariableTable},
    },
    utils::logger::{LogLevel, Logger},
};

pub fn run_audio_program(
    statements: &Vec<Statement>,
    audio_engine: &mut AudioEngine,
    _entry: String,
    _output: String,
    _module_variables: VariableTable,
    _module_functions: FunctionTable,
    global_store: &mut GlobalStore,
) -> (f32, f32) {
    let base_bpm = 120.0;
    let base_duration = 60.0 / base_bpm;

    let (max_end_time, cursor_time) = execute_audio_block(
        audio_engine,
        global_store,
        global_store.variables.clone(),
        global_store.functions.clone(),
        &statements,
        base_bpm,
        base_duration,
        0.0,
        0.0,
    );

    (max_end_time, cursor_time)
}

pub fn execute_audio_block(
    audio_engine: &mut AudioEngine,
    global_store: &GlobalStore,
    mut variable_table: VariableTable,
    mut functions_table: FunctionTable,
    statements: &[Statement],
    mut base_bpm: f32,
    mut base_duration: f32,
    mut max_end_time: f32,
    mut cursor_time: f32,
) -> (f32, f32) {
    // Track nested depth of execute_audio_block to avoid scheduling periodic events multiple times
    let current_depth = match variable_table.get("__depth") {
        Some(Value::Number(n)) => *n,
        _ => 0.0,
    };
    variable_table.set("__depth".to_string(), Value::Number(current_depth + 1.0));
    let (spawns, others): (Vec<_>, Vec<_>) = statements
        .iter()
        .partition(|stmt| matches!(stmt.kind, StatementKind::Spawn { .. }));

    // Execute sequential statements first
    for stmt in others {
        match &stmt.kind {
            StatementKind::Load { .. } => {
                if let Some(new_table) = interprete_load_statement(&stmt, &mut variable_table) {
                    variable_table = new_table;
                }
            }
            StatementKind::On { .. } => {
                // already registered in global store during parsing; nothing to do at runtime
            }
            StatementKind::Emit { event, payload: _ } => {
                if let Some(handlers) = global_store.get_event_handlers(event) {
                    for h in handlers {
                        if let StatementKind::On {
                            event: _,
                            args,
                            body,
                        } = &h.kind
                        {
                            // Create a derived variable table with event context
                            let mut vt = variable_table.clone();
                            let mut ctx = std::collections::HashMap::new();
                            ctx.insert("name".to_string(), Value::String(event.clone()));
                            if let Some(arg_list) = args.clone() {
                                ctx.insert("args".to_string(), Value::Array(arg_list));
                            }
                            // Attach payload if any on the Emit statement value
                            ctx.insert("payload".to_string(), stmt.value.clone());
                            vt.set("event".to_string(), Value::Map(ctx));
                            // Mark we're inside an event handler to avoid re-scheduling periodic events recursively
                            vt.set("__in_event".to_string(), Value::Boolean(true));

                            let (_max, _cursor) = execute_audio_block(
                                audio_engine,
                                global_store,
                                vt,
                                functions_table.clone(),
                                body,
                                base_bpm,
                                base_duration,
                                max_end_time,
                                cursor_time,
                            );
                        }
                    }
                }
            }
            StatementKind::Let { .. } => {
                if let Some(new_table) = interprete_let_statement(&stmt, &mut variable_table) {
                    variable_table = new_table;
                }
            }
            StatementKind::Function { .. } => {
                if let Some(new_functions) =
                    interprete_function_statement(&stmt, &mut functions_table)
                {
                    functions_table = new_functions;
                }
            }
            StatementKind::Tempo => {
                if let Some((new_bpm, new_duration)) = interprete_tempo_statement(&stmt) {
                    base_bpm = new_bpm;
                    base_duration = new_duration;
                }
            }
            StatementKind::Trigger { .. } => {
                if let Some((new_cursor, new_max, _)) = interprete_trigger_statement(
                    &stmt,
                    audio_engine,
                    &variable_table,
                    base_duration,
                    cursor_time,
                    max_end_time,
                ) {
                    cursor_time = new_cursor;
                    max_end_time = new_max;
                }
            }
            StatementKind::Sleep => {
                let (new_cursor, new_max) =
                    interprete_sleep_statement(&stmt, cursor_time, max_end_time);
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
                    cursor_time,
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
                    true,
                );

                cursor_time = new_cursor;

                if new_max > max_end_time {
                    max_end_time = new_max;
                }
            }
            StatementKind::Automate { .. } => {
                if let Some(new_table) =
                    crate::core::audio::interpreter::automate::interprete_automate_statement(
                        &stmt,
                        &mut variable_table,
                    )
                {
                    variable_table = new_table;
                }
            }
            StatementKind::Print => {
                // Only print in real-time mode (during playback), not during offline render.
                let is_realtime = matches!(variable_table.get("__rt"), Some(Value::Boolean(true)));
                if is_realtime {
                    let logger = Logger::new();
                    match &stmt.value {
                        Value::String(s) => {
                            let bpm = if let Some(Value::Number(n)) = variable_table.get("bpm") {
                                *n
                            } else {
                                120.0
                            };
                            let beat = if let Some(Value::Number(n)) = variable_table.get("beat") {
                                *n
                            } else {
                                0.0
                            };
                            // First try JS-like string concatenation: "str " + var + 1 + $env.*
                            if let Some(res) =
                                crate::core::audio::evaluator::evaluate_string_expression(
                                    s,
                                    &variable_table,
                                    bpm,
                                    beat,
                                )
                            {
                                logger.log_message(LogLevel::Print, &res);
                            } else if let Some(val) = variable_table.get(&s) {
                                logger.log_message(LogLevel::Print, &format!("{:?}", val));
                            } else if s.contains("$env")
                                || s.contains("$math")
                                || s.parse::<f32>().is_ok()
                            {
                                let v = crate::core::audio::evaluator::evaluate_rhs_into_value(
                                    s,
                                    &variable_table,
                                    bpm,
                                    beat,
                                );
                                match v {
                                    Value::Number(n) => {
                                        logger.log_message(LogLevel::Print, &format!("{}", n))
                                    }
                                    _ => logger.log_message(LogLevel::Print, s),
                                }
                            } else {
                                logger.log_message(LogLevel::Print, s)
                            }
                        }
                        Value::Number(n) => {
                            logger.log_message(LogLevel::Print, &format!("{}", n));
                        }
                        Value::Identifier(name) => {
                            if let Some(val) = variable_table.get(name) {
                                match val {
                                    Value::Number(n) => {
                                        logger.log_message(LogLevel::Print, &format!("{}", n))
                                    }
                                    Value::String(s) => logger.log_message(LogLevel::Print, s),
                                    Value::Boolean(b) => {
                                        logger.log_message(LogLevel::Print, &format!("{}", b))
                                    }
                                    other => {
                                        logger.log_message(LogLevel::Print, &format!("{:?}", other))
                                    }
                                }
                            } else {
                                logger.log_message(LogLevel::Print, name)
                            }
                        }
                        v => logger.log_message(LogLevel::Print, &format!("{:?}", v)),
                    }
                }
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
                0.0,
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

    // ─────────────────────────────────────────────────────────────
    // Built-in periodic events (e.g., on beat(n), on bar(n))
    // Emit handlers across the timeline up to max_end_time.
    // If no audio was scheduled (max_end_time == 0.0), skip.
    // Don't schedule periodic events if we're already inside an event handler
    let in_event = matches!(variable_table.get("__in_event"), Some(Value::Boolean(true)));
    let depth_is_root = matches!(variable_table.get("__depth"), Some(Value::Number(n)) if (*n - 1.0).abs() < f32::EPSILON);
    if max_end_time > 0.0 && !in_event && depth_is_root {
        if !global_store.events.is_empty() {
            // Beat-based handlers (support "beat" and "$beat")
            for ev_key in ["beat", "$beat"] {
                if let Some(handlers) = global_store.get_event_handlers(ev_key) {
                    let mut seen: std::collections::HashSet<(usize, usize, usize)> =
                        std::collections::HashSet::new();
                    // Default every 1 beat if args missing
                    for h in handlers {
                        let key = (h.line, h.column, h.indent);
                        if !seen.insert(key) {
                            continue;
                        }
                        if let StatementKind::On { event, args, body } = &h.kind {
                            let every: f32 = args
                                .as_ref()
                                .and_then(|v| v.get(0))
                                .and_then(|x| match x {
                                    Value::Number(n) => Some(*n),
                                    Value::Identifier(s) => {
                                        // Try to resolve from variables first, fallback to parsing the literal
                                        match variable_table.get(s) {
                                            Some(Value::Number(n)) => Some(*n),
                                            _ => s.parse::<f32>().ok(),
                                        }
                                    }
                                    _ => None,
                                })
                                .unwrap_or(1.0)
                                .max(0.0001);
                            let step = base_duration * every;
                            // Start from first full bar boundary after t=0
                            let mut t = step;
                            while t <= max_end_time {
                                // Prepare event context
                                let mut vt = variable_table.clone();
                                let mut ctx = std::collections::HashMap::new();
                                ctx.insert("name".to_string(), Value::String(event.clone()));
                                if let Some(a) = args.clone() {
                                    ctx.insert("args".to_string(), Value::Array(a));
                                }
                                vt.set("event".to_string(), Value::Map(ctx));
                                vt.set("beat".to_string(), Value::Number(t / base_duration));
                                // Prevent nested scheduling
                                vt.set("__in_event".to_string(), Value::Boolean(true));

                                let (_m, _c) = execute_audio_block(
                                    audio_engine,
                                    global_store,
                                    vt,
                                    functions_table.clone(),
                                    body,
                                    base_bpm,
                                    base_duration,
                                    max_end_time,
                                    t,
                                );

                                t += step;
                            }
                        }
                    }
                }
            }

            // Bar-based handlers (default 4/4 time => 4 beats per bar); support "bar" and "$bar"
            for ev_key in ["bar", "$bar"] {
                if let Some(handlers) = global_store.get_event_handlers(ev_key) {
                    let mut seen: std::collections::HashSet<(usize, usize, usize)> =
                        std::collections::HashSet::new();
                    for h in handlers {
                        let key = (h.line, h.column, h.indent);
                        if !seen.insert(key) {
                            continue;
                        }
                        if let StatementKind::On { event, args, body } = &h.kind {
                            let bar_beats = 4.0f32; // TODO: time signature support
                            let first_only = args.as_ref().and_then(|v| v.get(0)).is_none();

                            let every_bar: f32 = if first_only {
                                1.0
                            } else {
                                args.as_ref()
                                    .and_then(|v| v.get(0))
                                    .and_then(|x| match x {
                                        Value::Number(n) => Some(*n),
                                        Value::Identifier(s) => match variable_table.get(s) {
                                            Some(Value::Number(n)) => Some(*n),
                                            _ => s.parse::<f32>().ok(),
                                        },
                                        _ => None,
                                    })
                                    .unwrap_or(1.0)
                                    .max(0.0001)
                            };

                            let step = base_duration * bar_beats * every_bar;

                            if first_only {
                                let t = step; // first full bar after t=0
                                if t <= max_end_time {
                                    let mut vt = variable_table.clone();
                                    let mut ctx = std::collections::HashMap::new();
                                    ctx.insert("name".to_string(), Value::String(event.clone()));
                                    if let Some(a) = args.clone() {
                                        ctx.insert("args".to_string(), Value::Array(a));
                                    }
                                    vt.set("event".to_string(), Value::Map(ctx));
                                    vt.set("beat".to_string(), Value::Number(t / base_duration));
                                    // Prevent nested scheduling
                                    vt.set("__in_event".to_string(), Value::Boolean(true));

                                    let (_m, _c) = execute_audio_block(
                                        audio_engine,
                                        global_store,
                                        vt,
                                        functions_table.clone(),
                                        body,
                                        base_bpm,
                                        base_duration,
                                        max_end_time,
                                        t,
                                    );
                                }
                            } else {
                                let mut t = step; // start from first full bar after t=0
                                while t <= max_end_time {
                                    let mut vt = variable_table.clone();
                                    let mut ctx = std::collections::HashMap::new();
                                    ctx.insert("name".to_string(), Value::String(event.clone()));
                                    if let Some(a) = args.clone() {
                                        ctx.insert("args".to_string(), Value::Array(a));
                                    }
                                    vt.set("event".to_string(), Value::Map(ctx));
                                    vt.set("beat".to_string(), Value::Number(t / base_duration));
                                    // Prevent nested scheduling
                                    vt.set("__in_event".to_string(), Value::Boolean(true));

                                    let (_m, _c) = execute_audio_block(
                                        audio_engine,
                                        global_store,
                                        vt,
                                        functions_table.clone(),
                                        body,
                                        base_bpm,
                                        base_duration,
                                        max_end_time,
                                        t,
                                    );

                                    t += step;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (max_end_time.max(cursor_time), cursor_time)
}
