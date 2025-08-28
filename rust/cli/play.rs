use crate::{
    config::driver::ProjectConfig,
    core::{
        builder::Builder,
        debugger::{
            lexer::write_lexer_log_file,
            module::{write_module_function_log_file, write_module_variable_log_file},
            preprocessor::write_preprocessor_log_file,
            store::{write_function_log_file, write_variables_log_file},
        },
        preprocessor::loader::ModuleLoader,
        store::global::GlobalStore,
        utils::path::{find_entry_file, normalize_path},
    },
    utils::{
        logger::{LogLevel, Logger},
        spinner::with_spinner,
        watcher::watch_directory,
    },
};

use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    path::Path,
    sync::{Arc, mpsc::channel},
    thread,
    time::Duration,
};

#[cfg(feature = "cli")]
pub fn handle_play_command(
    config: Option<ProjectConfig>,
    entry: Option<String>,
    output: Option<String>,
    watch: bool,
    repeat: bool,
    debug: bool,
) -> Result<(), String> {
    use crate::core::audio::player::AudioPlayer;

    let logger = Logger::new();

    let entry_path = entry
        .or_else(|| config.as_ref().and_then(|c| c.defaults.entry.clone()))
        .unwrap_or_else(|| "".to_string());

    let output_path = output
        .or_else(|| config.as_ref().and_then(|c| c.defaults.output.clone()))
        .unwrap_or_else(|| "".to_string());

    let fetched_repeat = if repeat {
        true
    } else {
        config
            .as_ref()
            .and_then(|c| c.defaults.repeat)
            .unwrap_or(false)
    };

    if entry_path.is_empty() || output_path.is_empty() {
        logger.log_message(LogLevel::Error, "Entry or output path not specified.");
        return Err("missing entry or output".to_string());
    }

    let entry_file = match find_entry_file(&entry_path) {
        Some(p) => p,
        None => {
            logger.log_message(LogLevel::Error, "index.deva not found");
            return Err("index.deva not found".to_string());
        }
    };

    let audio_file = format!("{}/audio/index.wav", normalize_path(&output_path));
    // Persist basic stats at start of play (reflects current output dir)
    if let Err(e) = super::super::config::stats::save_to_file(&compute_basic_stats(&output_path)) {
        eprintln!("[stats] failed to save: {}", e);
    }
    let mut audio_player = AudioPlayer::new();

    // Helper: compute WAV duration in seconds
    fn wav_duration_seconds(path: &str) -> Option<f32> {
        if let Ok(reader) = hound::WavReader::open(path) {
            let spec = reader.spec();
            let len = reader.len(); // total samples (per channel)
            if spec.sample_rate == 0 {
                return None;
            }
            // len is total samples across channels; duration(seconds) = frames / sample_rate
            // frames = len / channels
            let channels = spec.channels.max(1) as u32;
            let frames = (len as u32) / channels;
            let dur = (frames as f32) / (spec.sample_rate as f32);
            Some(dur)
        } else {
            None
        }
    }

    // Real-time executor following tempo; executes one statement per beat; spawns run in parallel.
    struct RtContext {
        bpm: f32,
        entry_stmts: Vec<crate::core::parser::statement::Statement>,
        variables: crate::core::store::variable::VariableTable,
        functions: crate::core::store::function::FunctionTable,
        global_store: crate::core::store::global::GlobalStore,
    }

    struct RtRunner {
        stop: Arc<AtomicBool>,
        handle: std::thread::JoinHandle<()>,
    }
    fn start_realtime_runner(ctx: RtContext, total_secs: f32) -> RtRunner {
        use crate::core::audio::engine::AudioEngine;
        use crate::core::audio::interpreter::{
            arrow_call::interprete_call_arrow_statement, call::interprete_call_statement,
            driver::execute_audio_block, function::interprete_function_statement,
            let_::interprete_let_statement, loop_::interprete_loop_statement,
            sleep::interprete_sleep_statement, spawn::interprete_spawn_statement,
            tempo::interprete_tempo_statement,
        };
        use crate::core::parser::statement::Statement as AstStatement;
        use crate::core::parser::statement::StatementKind;
        use crate::core::shared::value::Value;
        use crate::utils::logger::{LogLevel, Logger};

        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        // State to pace a loop across beats
        struct CurrentLoopState {
            var_name: Option<String>,
            items: Vec<Value>,
            body: Vec<AstStatement>,
            idx: usize,
        }

        let handle = std::thread::spawn(move || {
            let logger = Logger::new();
            let mut bpm = if ctx.bpm > 0.0 { ctx.bpm } else { 120.0 };
            let mut beat_secs = 60.0f32 / bpm;
            let mut elapsed = 0.0f32;

            let mut variables = ctx.variables.clone();
            // mark realtime mode so prints/events only log during playback
            variables.set("__rt".to_string(), Value::Boolean(true));
            let mut functions = ctx.functions.clone();
            let global_store = ctx.global_store.clone();
            let mut audio_engine = AudioEngine::new("rt".to_string()); // dummy engine

            let mut i: usize = 0;
            let mut current_loop: Option<CurrentLoopState> = None;
            let mut beat_index: u64 = 0;
            while elapsed + 1e-3 < total_secs && i < ctx.entry_stmts.len() {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                // Wait until next beat
                std::thread::sleep(Duration::from_secs_f32(beat_secs));
                elapsed += beat_secs;
                beat_index += 1;
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                // If in the middle of a loop or if the next statement is a Loop, skip periodic events this beat to avoid interleaving
                let next_is_loop = current_loop.is_some()
                    || ctx
                        .entry_stmts
                        .get(i)
                        .map(|s| matches!(s.kind, StatementKind::Loop))
                        .unwrap_or(false);
                // Fire real-time periodic events (beat/bar)
                if !next_is_loop {
                    if let Some(handlers) = ctx.global_store.get_event_handlers("beat") {
                        for h in handlers {
                            if let StatementKind::On { event, args, body } = &h.kind {
                                let every: f32 = args
                                    .as_ref()
                                    .and_then(|v| v.get(0))
                                    .and_then(|x| match x {
                                        Value::Number(n) => Some(*n),
                                        Value::Identifier(s) => match variables.get(s) {
                                            Some(Value::Number(n)) => Some(*n),
                                            _ => s.parse::<f32>().ok(),
                                        },
                                        _ => None,
                                    })
                                    .unwrap_or(1.0)
                                    .max(0.0001);
                                let period = every.round().max(1.0) as u64;
                                if beat_index % period == 0 {
                                    let mut vt = variables.clone();
                                    let mut ctx_map = std::collections::HashMap::new();
                                    ctx_map
                                        .insert("name".to_string(), Value::String(event.clone()));
                                    if let Some(a) = args.clone() {
                                        ctx_map.insert("args".to_string(), Value::Array(a));
                                    }
                                    vt.set("event".to_string(), Value::Map(ctx_map));
                                    vt.set("beat".to_string(), Value::Number(beat_index as f32));
                                    vt.set("__in_event".to_string(), Value::Boolean(true));
                                    vt.set("__rt".to_string(), Value::Boolean(true));
                                    let _ = execute_audio_block(
                                        &mut audio_engine,
                                        &ctx.global_store,
                                        vt,
                                        functions.clone(),
                                        body,
                                        bpm,
                                        60.0 / bpm,
                                        0.0,
                                        0.0,
                                    );
                                }
                            }
                        }
                    }
                }
                if !next_is_loop {
                    if let Some(handlers) = ctx.global_store.get_event_handlers("$beat") {
                        for h in handlers {
                            if let StatementKind::On { event, args, body } = &h.kind {
                                let every: f32 = args
                                    .as_ref()
                                    .and_then(|v| v.get(0))
                                    .and_then(|x| match x {
                                        Value::Number(n) => Some(*n),
                                        Value::Identifier(s) => match variables.get(s) {
                                            Some(Value::Number(n)) => Some(*n),
                                            _ => s.parse::<f32>().ok(),
                                        },
                                        _ => None,
                                    })
                                    .unwrap_or(1.0)
                                    .max(0.0001);
                                let period = every.round().max(1.0) as u64;
                                if beat_index % period == 0 {
                                    let mut vt = variables.clone();
                                    let mut ctx_map = std::collections::HashMap::new();
                                    ctx_map
                                        .insert("name".to_string(), Value::String(event.clone()));
                                    if let Some(a) = args.clone() {
                                        ctx_map.insert("args".to_string(), Value::Array(a));
                                    }
                                    vt.set("event".to_string(), Value::Map(ctx_map));
                                    vt.set("beat".to_string(), Value::Number(beat_index as f32));
                                    vt.set("__in_event".to_string(), Value::Boolean(true));
                                    vt.set("__rt".to_string(), Value::Boolean(true));
                                    let _ = execute_audio_block(
                                        &mut audio_engine,
                                        &ctx.global_store,
                                        vt,
                                        functions.clone(),
                                        body,
                                        bpm,
                                        60.0 / bpm,
                                        0.0,
                                        0.0,
                                    );
                                }
                            }
                        }
                    }
                }
                // Bars (4/4)
                let bar_beats = 4u64;
                if !next_is_loop {
                    if let Some(handlers) = ctx.global_store.get_event_handlers("bar") {
                        for h in handlers {
                            if let StatementKind::On { event, args, body } = &h.kind {
                                let first_only = args.as_ref().and_then(|v| v.get(0)).is_none();
                                let every_bar: f32 = if first_only {
                                    1.0
                                } else {
                                    args.as_ref()
                                        .and_then(|v| v.get(0))
                                        .and_then(|x| match x {
                                            Value::Number(n) => Some(*n),
                                            Value::Identifier(s) => match variables.get(s) {
                                                Some(Value::Number(n)) => Some(*n),
                                                _ => s.parse::<f32>().ok(),
                                            },
                                            _ => None,
                                        })
                                        .unwrap_or(1.0)
                                        .max(0.0001)
                                };
                                let step_beats =
                                    ((bar_beats as f32) * every_bar).round().max(1.0) as u64;
                                let should_fire = if first_only {
                                    beat_index == step_beats
                                } else {
                                    beat_index % step_beats == 0
                                };
                                if should_fire {
                                    let mut vt = variables.clone();
                                    let mut ctx_map = std::collections::HashMap::new();
                                    ctx_map
                                        .insert("name".to_string(), Value::String(event.clone()));
                                    if let Some(a) = args.clone() {
                                        ctx_map.insert("args".to_string(), Value::Array(a));
                                    }
                                    vt.set("event".to_string(), Value::Map(ctx_map));
                                    vt.set("beat".to_string(), Value::Number(beat_index as f32));
                                    vt.set("__in_event".to_string(), Value::Boolean(true));
                                    vt.set("__rt".to_string(), Value::Boolean(true));
                                    let _ = execute_audio_block(
                                        &mut audio_engine,
                                        &ctx.global_store,
                                        vt,
                                        functions.clone(),
                                        body,
                                        bpm,
                                        60.0 / bpm,
                                        0.0,
                                        0.0,
                                    );
                                }
                            }
                        }
                    }
                }
                if !next_is_loop {
                    if let Some(handlers) = ctx.global_store.get_event_handlers("$bar") {
                        for h in handlers {
                            if let StatementKind::On { event, args, body } = &h.kind {
                                let first_only = args.as_ref().and_then(|v| v.get(0)).is_none();
                                let every_bar: f32 = if first_only {
                                    1.0
                                } else {
                                    args.as_ref()
                                        .and_then(|v| v.get(0))
                                        .and_then(|x| match x {
                                            Value::Number(n) => Some(*n),
                                            Value::Identifier(s) => match variables.get(s) {
                                                Some(Value::Number(n)) => Some(*n),
                                                _ => s.parse::<f32>().ok(),
                                            },
                                            _ => None,
                                        })
                                        .unwrap_or(1.0)
                                        .max(0.0001)
                                };
                                let step_beats =
                                    ((bar_beats as f32) * every_bar).round().max(1.0) as u64;
                                let should_fire = if first_only {
                                    beat_index == step_beats
                                } else {
                                    beat_index % step_beats == 0
                                };
                                if should_fire {
                                    let mut vt = variables.clone();
                                    let mut ctx_map = std::collections::HashMap::new();
                                    ctx_map
                                        .insert("name".to_string(), Value::String(event.clone()));
                                    if let Some(a) = args.clone() {
                                        ctx_map.insert("args".to_string(), Value::Array(a));
                                    }
                                    vt.set("event".to_string(), Value::Map(ctx_map));
                                    vt.set("beat".to_string(), Value::Number(beat_index as f32));
                                    vt.set("__in_event".to_string(), Value::Boolean(true));
                                    vt.set("__rt".to_string(), Value::Boolean(true));
                                    let _ = execute_audio_block(
                                        &mut audio_engine,
                                        &ctx.global_store,
                                        vt,
                                        functions.clone(),
                                        body,
                                        bpm,
                                        60.0 / bpm,
                                        0.0,
                                        0.0,
                                    );
                                }
                            }
                        }
                    }
                }

                // If we are in the middle of a loop, execute one iteration per beat
                if let Some(state) = &mut current_loop {
                    if state.idx < state.items.len() {
                        let mut vt = variables.clone();
                        if let Some(name) = &state.var_name {
                            vt.set(name.clone(), state.items[state.idx].clone());
                        }
                        let _ = execute_audio_block(
                            &mut audio_engine,
                            &global_store,
                            vt,
                            functions.clone(),
                            &state.body,
                            bpm,
                            60.0 / bpm,
                            0.0,
                            0.0,
                        );
                        state.idx += 1;
                        // If finished, clear and advance to next statement
                        if state.idx >= state.items.len() {
                            current_loop = None;
                            i += 1;
                        }
                        continue; // consume this beat
                    } else {
                        current_loop = None; // safety
                    }
                }

                let stmt = ctx.entry_stmts[i].clone();

                match &stmt.kind {
                    StatementKind::Tempo => {
                        if let Some((new_bpm, _dur)) = interprete_tempo_statement(&stmt) {
                            bpm = new_bpm;
                            beat_secs = 60.0 / bpm.max(0.0001);
                        }
                    }
                    StatementKind::Sleep => {
                        let (new_cursor, _max) = interprete_sleep_statement(&stmt, 0.0, 0.0);
                        if new_cursor > 0.0 {
                            std::thread::sleep(Duration::from_secs_f32(new_cursor));
                            elapsed += new_cursor;
                        }
                    }
                    StatementKind::Let { .. } => {
                        if let Some(new_vars) = interprete_let_statement(&stmt, &mut variables) {
                            variables = new_vars;
                        }
                    }
                    StatementKind::Function { .. } => {
                        if let Some(new_funcs) =
                            interprete_function_statement(&stmt, &mut functions)
                        {
                            functions = new_funcs;
                        }
                    }
                    StatementKind::Call { .. } => {
                        let (_new_max, _cursor) = interprete_call_statement(
                            &stmt,
                            &mut audio_engine,
                            &variables,
                            &functions,
                            &global_store,
                            bpm,
                            60.0 / bpm,
                            0.0,
                            0.0,
                        );
                    }
                    StatementKind::ArrowCall { .. } => {
                        let mut max_end_time = 0.0;
                        let (_max, _cursor) = interprete_call_arrow_statement(
                            &stmt,
                            &mut audio_engine,
                            &variables,
                            bpm,
                            60.0 / bpm,
                            &mut max_end_time,
                            None,
                            true,
                        );
                    }
                    StatementKind::Loop => {
                        // Initialize pacing state for the loop and execute first iteration this beat
                        if let Value::Map(loop_map) = &stmt.value {
                            let body: Vec<AstStatement> = match loop_map.get("body") {
                                Some(Value::Block(b)) => b.clone(),
                                _ => Vec::new(),
                            };

                            if let (Some(Value::Identifier(var_name)), Some(Value::Array(items))) =
                                (loop_map.get("foreach"), loop_map.get("array"))
                            {
                                // foreach form
                                current_loop = Some(CurrentLoopState {
                                    var_name: Some(var_name.clone()),
                                    items: items.clone(),
                                    body,
                                    idx: 0,
                                });
                            } else if let Some(Value::Number(n)) = loop_map.get("iterator") {
                                // count form -> iterate 0..n-1
                                let count = (*n).max(0.0) as usize;
                                let items: Vec<Value> =
                                    (0..count).map(|i| Value::Number(i as f32)).collect();
                                current_loop = Some(CurrentLoopState {
                                    var_name: None,
                                    items,
                                    body,
                                    idx: 0,
                                });
                            } else {
                                // Fallback: execute immediately if malformed
                                let (_max, _cursor) = interprete_loop_statement(
                                    &stmt,
                                    &mut audio_engine,
                                    &global_store,
                                    &variables,
                                    &functions,
                                    bpm,
                                    60.0 / bpm,
                                    0.0,
                                    0.0,
                                );
                                i += 1;
                                continue;
                            }

                            // Execute first iteration now
                            if let Some(state) = &mut current_loop {
                                if state.idx < state.items.len() {
                                    let mut vt = variables.clone();
                                    if let Some(name) = &state.var_name {
                                        vt.set(name.clone(), state.items[state.idx].clone());
                                    }
                                    let _ = execute_audio_block(
                                        &mut audio_engine,
                                        &global_store,
                                        vt,
                                        functions.clone(),
                                        &state.body,
                                        bpm,
                                        60.0 / bpm,
                                        0.0,
                                        0.0,
                                    );
                                    state.idx += 1;
                                    if state.idx >= state.items.len() {
                                        current_loop = None;
                                        i += 1;
                                    }
                                    continue; // consume this beat
                                } else {
                                    current_loop = None; // no items, advance
                                    i += 1;
                                    continue;
                                }
                            }
                        } else {
                            // Not a map, fallback to immediate interpreter
                            let (_max, _cursor) = interprete_loop_statement(
                                &stmt,
                                &mut audio_engine,
                                &global_store,
                                &variables,
                                &functions,
                                bpm,
                                60.0 / bpm,
                                0.0,
                                0.0,
                            );
                            i += 1;
                            continue;
                        }
                    }
                    StatementKind::Spawn { .. } => {
                        // Run in parallel
                        let stmt_clone = stmt.clone();
                        let mut local_engine = AudioEngine::new("rt_spawn".to_string());
                        let vars = variables.clone();
                        let funcs = functions.clone();
                        let store = global_store.clone();
                        let local_bpm = bpm;
                        std::thread::spawn(move || {
                            let _ = interprete_spawn_statement(
                                &stmt_clone,
                                &mut local_engine,
                                &vars,
                                &funcs,
                                &store,
                                local_bpm,
                                60.0 / local_bpm,
                                0.0,
                                0.0,
                            );
                        });
                    }
                    StatementKind::Print => {
                        // Reuse print behavior from audio driver
                        match &stmt.value {
                            Value::String(s) => {
                                let env_bpm = bpm;
                                let env_beat = (elapsed / beat_secs).floor();
                                if let Some(res) =
                                    crate::core::audio::evaluator::evaluate_string_expression(
                                        s, &variables, env_bpm, env_beat,
                                    )
                                {
                                    logger.log_message(LogLevel::Print, &res);
                                } else if let Some(val) = variables.get(&s) {
                                    logger.log_message(LogLevel::Print, &format!("{:?}", val));
                                } else if s.contains("$env")
                                    || s.contains("$math")
                                    || s.parse::<f32>().is_ok()
                                {
                                    let v = crate::core::audio::evaluator::evaluate_rhs_into_value(
                                        s, &variables, env_bpm, env_beat,
                                    );
                                    match v {
                                        Value::Number(n) => {
                                            logger.log_message(LogLevel::Print, &format!("{}", n))
                                        }
                                        _ => logger.log_message(LogLevel::Print, s),
                                    }
                                } else {
                                    logger.log_message(LogLevel::Print, s);
                                }
                            }
                            Value::Number(n) => {
                                logger.log_message(LogLevel::Print, &format!("{}", n));
                            }
                            Value::Identifier(name) => {
                                if let Some(val) = variables.get(name) {
                                    match val {
                                        Value::Number(n) => {
                                            logger.log_message(LogLevel::Print, &format!("{}", n))
                                        }
                                        Value::String(s) => logger.log_message(LogLevel::Print, s),
                                        Value::Boolean(b) => {
                                            logger.log_message(LogLevel::Print, &format!("{}", b))
                                        }
                                        other => logger
                                            .log_message(LogLevel::Print, &format!("{:?}", other)),
                                    }
                                } else {
                                    logger.log_message(LogLevel::Print, name);
                                }
                            }
                            v => logger.log_message(LogLevel::Print, &format!("{:?}", v)),
                        }
                    }
                    StatementKind::On { .. } => { /* handlers already registered by preprocessor */
                    }
                    StatementKind::Emit { .. } => { /* could log or handle later */ }
                    _ => { /* ignore others in RT runner */ }
                }

                i += 1;
            }
        });
        RtRunner { stop, handle }
    }
    fn stop_realtime_runner(runner_opt: &mut Option<RtRunner>) {
        if let Some(r) = runner_opt.take() {
            r.stop.store(true, Ordering::Relaxed);
            let _ = r.handle.join();
        }
    }

    fn join_realtime_runner(runner_opt: &mut Option<RtRunner>) {
        if let Some(r) = runner_opt.take() {
            let _ = r.handle.join();
        }
    }

    if watch && fetched_repeat {
        logger.log_message(
            LogLevel::Error,
            "Watch and repeat cannot be used together. Use repeat instead.",
        );
        return Err("invalid options: watch and repeat cannot be combined".to_string());
    }

    if watch {
        let (tx, rx) = channel::<()>();

        // Thread 1 : Watcher sending changes
        let entry_clone = entry_path.clone();
        thread::spawn(move || {
            let _ = watch_directory(entry_clone, move || {
                let _ = tx.send(()); // signal a change
            });
        });

        // Main thread: build + play in a loop
        let (bpm, entry_stmts, variables, functions, global_store) =
            begin_play(&config, &entry_file, &output_path, debug)?;
        audio_player.play_file_once(&audio_file);
        // Estimate duration: base on statement count plus extra for loop iterations (1 beat per iter)
        let loop_iters: usize = entry_stmts
            .iter()
            .map(|s| match &s.kind {
                crate::core::parser::statement::StatementKind::Loop => {
                    if let crate::core::shared::value::Value::Map(m) = &s.value {
                        if let Some(crate::core::shared::value::Value::Array(items)) =
                            m.get("array")
                        {
                            items.len()
                        } else if let Some(crate::core::shared::value::Value::Number(n)) =
                            m.get("iterator")
                        {
                            (*n).max(0.0) as usize
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            })
            .sum();
        let est_beats = entry_stmts.len() as f32 + loop_iters as f32;
        let est_by_len = ((60.0 / bpm).max(0.01) * est_beats).max(1.0);
        let total_secs = wav_duration_seconds(&audio_file)
            .unwrap_or(0.0)
            .max(est_by_len);
        let mut rt_runner = Some(start_realtime_runner(
            RtContext {
                bpm,
                entry_stmts,
                variables,
                functions,
                global_store,
            },
            total_secs,
        ));

        logger.log_message(
            LogLevel::Watcher,
            "Watching for changes... Press Ctrl+C to exit.",
        );

        while let Ok(_) = rx.recv() {
            logger.log_message(LogLevel::Watcher, "Change detected, rebuilding...");

            // Stop previous real-time runner before restarting playback
            stop_realtime_runner(&mut rt_runner);

            let (bpm, entry_stmts, variables, functions, global_store) =
                match begin_play(&config, &entry_file, &output_path, debug) {
                    Ok(v) => v,
                    Err(e) => {
                        logger.log_message(LogLevel::Error, &format!("Rebuild failed: {}", e));
                        continue;
                    }
                };

            logger.log_message(LogLevel::Info, "🎵 Playback started (once mode)...");

            audio_player.play_file_once(&audio_file);
            let loop_iters: usize = entry_stmts
                .iter()
                .map(|s| match &s.kind {
                    crate::core::parser::statement::StatementKind::Loop => {
                        if let crate::core::shared::value::Value::Map(m) = &s.value {
                            if let Some(crate::core::shared::value::Value::Array(items)) =
                                m.get("array")
                            {
                                items.len()
                            } else if let Some(crate::core::shared::value::Value::Number(n)) =
                                m.get("iterator")
                            {
                                (*n).max(0.0) as usize
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }
                    _ => 0,
                })
                .sum();
            let est_beats = entry_stmts.len() as f32 + loop_iters as f32;
            let est_by_len = ((60.0 / bpm).max(0.01) * est_beats).max(1.0);
            let total_secs = wav_duration_seconds(&audio_file)
                .unwrap_or(0.0)
                .max(est_by_len);
            rt_runner = Some(start_realtime_runner(
                RtContext {
                    bpm,
                    entry_stmts,
                    variables,
                    functions,
                    global_store,
                },
                total_secs,
            ));
        }
    } else if fetched_repeat {
        // Initial build to start from a clean slate
        let (bpm, entry_stmts, variables, functions, global_store) =
            begin_play(&config, &entry_file, &output_path, debug)?;

        logger.log_message(LogLevel::Info, "🎵 Playback started (repeat mode)...");

        let mut last_snapshot = snapshot_files(&entry_path);
        let mut audio_player = AudioPlayer::new();
        audio_player.play_file_once(&audio_file);

        let loop_iters: usize = entry_stmts
            .iter()
            .map(|s| match &s.kind {
                crate::core::parser::statement::StatementKind::Loop => {
                    if let crate::core::shared::value::Value::Map(m) = &s.value {
                        if let Some(crate::core::shared::value::Value::Array(items)) =
                            m.get("array")
                        {
                            items.len()
                        } else if let Some(crate::core::shared::value::Value::Number(n)) =
                            m.get("iterator")
                        {
                            (*n).max(0.0) as usize
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                }
                _ => 0,
            })
            .sum();
        let est_beats = entry_stmts.len() as f32 + loop_iters as f32;
        let est_by_len = ((60.0 / bpm).max(0.01) * est_beats).max(1.0);
        let total_secs = wav_duration_seconds(&audio_file)
            .unwrap_or(0.0)
            .max(est_by_len);
        let mut rt_runner = Some(start_realtime_runner(
            RtContext {
                bpm,
                entry_stmts: entry_stmts.clone(),
                variables: variables.clone(),
                functions: functions.clone(),
                global_store: global_store.clone(),
            },
            total_secs,
        ));

        loop {
            let current_snapshot = snapshot_files(&entry_path);
            let has_changed = files_changed(&last_snapshot, &current_snapshot);

            if has_changed {
                logger.log_message(
                    LogLevel::Info,
                    "Change detected, rebuilding in background...",
                );
                let entry_file = entry_file.clone();
                let output_path = output_path.clone();
                let config_clone = config.clone();

                // Rebuild in a separate thread
                std::thread::spawn(move || {
                    if let Err(e) = begin_play(&config_clone, &entry_file, &output_path, debug) {
                        eprintln!("Rebuild failed in background: {}", e);
                    }
                });

                last_snapshot = current_snapshot;
            }

            // Wait for the audio to finish
            audio_player.wait_until_end();
            // Stop the current real-time runner
            stop_realtime_runner(&mut rt_runner);

            // Then replay the audio (rebuilt or not)
            audio_player.play_file_once(&audio_file);
            let loop_iters: usize = entry_stmts
                .iter()
                .map(|s| match &s.kind {
                    crate::core::parser::statement::StatementKind::Loop => {
                        if let crate::core::shared::value::Value::Map(m) = &s.value {
                            if let Some(crate::core::shared::value::Value::Array(items)) =
                                m.get("array")
                            {
                                items.len()
                            } else if let Some(crate::core::shared::value::Value::Number(n)) =
                                m.get("iterator")
                            {
                                (*n).max(0.0) as usize
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }
                    _ => 0,
                })
                .sum();
            let est_beats = entry_stmts.len() as f32 + loop_iters as f32;
            let est_by_len = ((60.0 / bpm).max(0.01) * est_beats).max(1.0);
            let total_secs = wav_duration_seconds(&audio_file)
                .unwrap_or(0.0)
                .max(est_by_len);
            rt_runner = Some(start_realtime_runner(
                RtContext {
                    bpm,
                    entry_stmts: entry_stmts.clone(),
                    variables: variables.clone(),
                    functions: functions.clone(),
                    global_store: global_store.clone(),
                },
                total_secs,
            ));
        }
    } else {
        // Single execution
        let (bpm, entry_stmts, variables, functions, global_store) =
            begin_play(&config, &entry_file, &output_path, debug)?;

        logger.log_message(LogLevel::Info, "🎵 Playback started (once mode)...");

        audio_player.play_file_once(&audio_file);

        let est_by_len = ((60.0 / bpm).max(0.01) * (entry_stmts.len() as f32)).max(1.0);
        let total_secs = wav_duration_seconds(&audio_file)
            .unwrap_or(0.0)
            .max(est_by_len);
        let mut rt_runner = Some(start_realtime_runner(
            RtContext {
                bpm,
                entry_stmts,
                variables,
                functions,
                global_store,
            },
            total_secs,
        ));

        audio_player.wait_until_end();
        // Let the runner finish naturally to execute all remaining statements (e.g., loop prints)
        join_realtime_runner(&mut rt_runner);
    }
    // ... existing implementation continues unchanged until the end of the file ...
    // The original function didn’t return a Result; ensure Ok(()) on normal paths and Err(...) on failures.
    Ok(())
}

fn compute_basic_stats(output_dir: &str) -> crate::config::stats::ProjectStats {
    use crate::config::stats::ProjectStats;
    use std::{fs, path::Path};
    let mut stats = ProjectStats::new();
    let out = Path::new(output_dir);
    let mut nb_files = 0usize;
    let mut nb_lines = 0usize;
    if out.exists() {
        if let Ok(entries) = fs::read_dir(out) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_file() {
                        nb_files += 1;
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            nb_lines += content.lines().count();
                        }
                    }
                }
            }
        }
    }
    stats.counts.nb_files = nb_files;
    stats.counts.nb_lines = nb_lines;
    stats
}

fn begin_play(
    _config: &Option<ProjectConfig>,
    entry_file: &str,
    output: &str,
    debug: bool,
) -> Result<
    (
        f32,
        Vec<crate::core::parser::statement::Statement>,
        crate::core::store::variable::VariableTable,
        crate::core::store::function::FunctionTable,
        crate::core::store::global::GlobalStore,
    ),
    String,
> {
    let spinner = with_spinner("Building...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let normalized_entry = normalize_path(entry_file);
    let normalized_output_dir = normalize_path(&output);

    let duration = std::time::Instant::now();
    let mut global_store = GlobalStore::new();
    let loader = ModuleLoader::new(&normalized_entry, &normalized_output_dir);
    let (modules_tokens, modules_statements) = loader.load_all_modules(&mut global_store);

    // Try to detect initial BPM from statements (fallback to 120.0)
    let mut detected_bpm: f32 = 120.0;
    let mut entry_statements: Vec<crate::core::parser::statement::Statement> = Vec::new();
    // Prefer the entry module if present
    if let Some(entry_stmts) = modules_statements.get(&normalized_entry) {
        entry_statements = entry_stmts.clone();
        for stmt in entry_stmts {
            if let crate::core::parser::statement::StatementKind::Tempo = &stmt.kind {
                if let crate::core::shared::value::Value::Number(n) = &stmt.value {
                    detected_bpm = *n;
                    break;
                }
            }
        }
    }
    // If still default, scan other modules for a tempo directive
    if (detected_bpm - 120.0).abs() < f32::EPSILON {
        'outer: for (_name, stmts) in modules_statements.iter() {
            for stmt in stmts {
                if let crate::core::parser::statement::StatementKind::Tempo = &stmt.kind {
                    if let crate::core::shared::value::Value::Number(n) = &stmt.value {
                        detected_bpm = *n;
                        break 'outer;
                    }
                }
            }
        }
    }

    // SECTION Write logs
    if debug {
        for (module_path, module) in global_store.modules.clone() {
            write_module_variable_log_file(
                &normalized_output_dir,
                &module_path,
                &module.variable_table,
            );
            write_module_function_log_file(
                &normalized_output_dir,
                &module_path,
                &module.function_table,
            );
        }

        write_lexer_log_file(
            &normalized_output_dir,
            "lexer_tokens.log",
            modules_tokens.clone(),
        );
        write_preprocessor_log_file(
            &normalized_output_dir,
            "resolved_statements.log",
            modules_statements.clone(),
        );
        write_variables_log_file(
            &normalized_output_dir,
            "global_variables.log",
            global_store.variables.clone(),
        );
        write_function_log_file(
            &normalized_output_dir,
            "global_functions.log",
            global_store.functions.clone(),
        );
    }

    // SECTION Detect errors before building (like build.rs)
    let all_errors = crate::utils::error::collect_all_errors_with_modules(&modules_statements);
    let (warnings, criticals) = crate::utils::error::partition_errors(all_errors);
    crate::utils::error::log_errors_with_stack("Play", &warnings, &criticals);
    if !criticals.is_empty() {
        spinner.finish_and_clear();
        return Err(format!(
            "play failed with {} critical error(s): {}",
            criticals.len(),
            criticals[0].message
        ));
    }

    // SECTION Building AST and Audio
    let builder = Builder::new();
    builder.build_ast(&modules_statements, &output, false);
    builder.build_audio(&modules_statements, &output, &mut global_store);

    // SECTION Logging
    let logger = Logger::new();
    let success_message = format!(
        "Build completed successfully in {:.2?}. Output files written to: '{}'",
        duration.elapsed(),
        normalized_output_dir
    );

    // Compute and persist rich stats (align with build/check)
    let stats = crate::config::stats::compute_from(
        &modules_statements,
        &global_store,
        &_config,
        Some(&normalized_output_dir),
    );
    crate::config::stats::set_memory_stats(stats.clone());
    if let Err(e) = crate::config::stats::save_to_file(&stats) {
        eprintln!("[stats] failed to save: {}", e);
    }

    spinner.finish_and_clear();
    logger.log_message(LogLevel::Success, &success_message);

    Ok((
        detected_bpm,
        entry_statements,
        global_store.variables.clone(),
        global_store.functions.clone(),
        global_store,
    ))
}

fn snapshot_files<P: AsRef<Path>>(dir: P) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        map.insert(entry.path().display().to_string(), duration.as_secs());
                    }
                }
            }
        }
    }
    map
}

fn files_changed(old: &HashMap<String, u64>, new: &HashMap<String, u64>) -> bool {
    old != new
}
