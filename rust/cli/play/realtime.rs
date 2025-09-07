use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use devalang_types::Value;

pub struct RtRunner {
    pub stop: Arc<AtomicBool>,
    pub handle: std::thread::JoinHandle<()>,
}

pub struct RtContext {
    pub bpm: f32,
    pub entry_stmts: Vec<crate::core::parser::statement::Statement>,
    pub variables: devalang_types::store::VariableTable,
    pub functions: devalang_types::store::FunctionTable,
    pub global_store: crate::core::store::global::GlobalStore,
}

pub fn start_realtime_runner(ctx: RtContext, total_secs: f32) -> RtRunner {
    use crate::core::audio::engine::AudioEngine;
    use crate::core::audio::interpreter::driver::execute_audio_block;
    use crate::core::parser::statement::StatementKind;
    use devalang_utils::logger::Logger;

    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    let handle = std::thread::spawn(move || {
        let _logger = Logger::new();
        let bpm = if ctx.bpm > 0.0 { ctx.bpm } else { 120.0 };
        let beat_secs = 60.0f32 / bpm;
        let mut elapsed = 0.0f32;

        let mut variables = ctx.variables.clone();
        variables.set("__rt".to_string(), Value::Boolean(true));
        let functions = ctx.functions.clone();
        let _global_store = ctx.global_store.clone();
        let mut audio_engine = AudioEngine::new("rt".to_string());

        let i: usize = 0;
        let mut _current_loop: Option<()> = None; // simplified state
        let mut _beat_index: u64 = 0;
        while elapsed + 1e-3 < total_secs && i < ctx.entry_stmts.len() {
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }

            std::thread::sleep(Duration::from_secs_f32(beat_secs));
            elapsed += beat_secs;
            _beat_index += 1;
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }

            // Only fire periodic handlers when not in a loop - simplified
            if let Some(handlers) = ctx.global_store.get_event_handlers("beat") {
                for h in handlers {
                    if let StatementKind::On { body, .. } = &h.kind {
                        let _ = execute_audio_block(
                            &mut audio_engine,
                            &ctx.global_store,
                            variables.clone(),
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
    });

    RtRunner { stop, handle }
}

pub fn stop_realtime_runner(runner_opt: &mut Option<RtRunner>) {
    if let Some(r) = runner_opt.take() {
        r.stop.store(true, Ordering::Relaxed);
        let _ = r.handle.join();
    }
}

pub fn join_realtime_runner(runner_opt: &mut Option<RtRunner>) {
    if let Some(r) = runner_opt.take() {
        let _ = r.handle.join();
    }
}
