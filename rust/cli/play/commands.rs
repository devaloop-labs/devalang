use crate::config::driver::ProjectConfig;
use devalang_utils::logger::{LogLevel, Logger};
use std::{sync::mpsc::channel, thread};

pub use crate::cli::play::io::wav_duration_seconds;
pub use crate::cli::play::realtime::{
    RtContext, join_realtime_runner, start_realtime_runner, stop_realtime_runner,
};

use super::process::process_play;
use super::utils::{files_changed, snapshot_files};

use crate::core::audio::player::AudioPlayer;

#[cfg(feature = "cli")]
pub fn handle_play_command(
    config: Option<ProjectConfig>,
    entry: Option<String>,
    output: Option<String>,
    watch: bool,
    repeat: bool,
    debug: bool,
) -> Result<(), String> {
    let logger = Logger::new();

    let entry_path = entry
        .or_else(|| config.as_ref().and_then(|c| c.defaults.entry.clone()))
        .unwrap_or_default();

    let output_path = output
        .or_else(|| config.as_ref().and_then(|c| c.defaults.output.clone()))
        .unwrap_or_default();

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

    let entry_file = match crate::core::utils::path::find_entry_file(&entry_path) {
        Some(p) => p,
        None => {
            logger.log_message(LogLevel::Error, "index.deva not found");
            return Err("index.deva not found".to_string());
        }
    };

    let audio_file = format!(
        "{}/audio/index.wav",
        crate::core::utils::path::normalize_path(&output_path)
    );
    let mut audio_player = AudioPlayer::new();

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
            let _ = devalang_utils::watcher::watch_directory(entry_clone, move || {
                let _ = tx.send(()); // signal a change
            });
        });

        // Main thread: build + play in a loop
        let (bpm, entry_stmts, variables, functions, global_store) =
            process_play(&config, &entry_file, &output_path, debug)?;
        audio_player.play_file_once(&audio_file);
        // Estimate duration: base on statement count plus extra for loop iterations (1 beat per iter)
        let loop_iters: usize = entry_stmts
            .iter()
            .map(|s| match &s.kind {
                crate::core::parser::statement::StatementKind::Loop => {
                    use devalang_types::Value;
                    if let Value::Map(m) = &s.value {
                        if let Some(Value::Array(items)) = m.get("array") {
                            items.len()
                        } else if let Some(Value::Number(n)) = m.get("iterator") {
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
        let est_beats = (entry_stmts.len() as f32) + (loop_iters as f32);
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

        while rx.recv().is_ok() {
            logger.log_message(LogLevel::Watcher, "Change detected, rebuilding...");

            // Stop previous real-time runner before restarting playback
            stop_realtime_runner(&mut rt_runner);

            let (bpm, entry_stmts, variables, functions, global_store) =
                match process_play(&config, &entry_file, &output_path, debug) {
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
                        use devalang_types::Value;
                        if let Value::Map(m) = &s.value {
                            if let Some(Value::Array(items)) = m.get("array") {
                                items.len()
                            } else if let Some(Value::Number(n)) = m.get("iterator") {
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
            let est_beats = (entry_stmts.len() as f32) + (loop_iters as f32);
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
    } else if fetched_repeat {
        // Initial build to start from a clean slate
        let (bpm, entry_stmts, variables, functions, global_store) =
            process_play(&config, &entry_file, &output_path, debug)?;

        logger.log_message(LogLevel::Info, "🎵 Playback started (repeat mode)...");

        let mut last_snapshot = snapshot_files(&entry_path);
        let mut audio_player = AudioPlayer::new();
        audio_player.play_file_once(&audio_file);

        let loop_iters: usize = entry_stmts
            .iter()
            .map(|s| match &s.kind {
                crate::core::parser::statement::StatementKind::Loop => {
                    use devalang_types::Value;

                    if let Value::Map(m) = &s.value {
                        if let Some(Value::Array(items)) = m.get("array") {
                            items.len()
                        } else if let Some(Value::Number(n)) = m.get("iterator") {
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
        let est_beats = (entry_stmts.len() as f32) + (loop_iters as f32);
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
                    if let Err(e) = process_play(&config_clone, &entry_file, &output_path, debug) {
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
                        use devalang_types::Value;
                        if let Value::Map(m) = &s.value {
                            if let Some(Value::Array(items)) = m.get("array") {
                                items.len()
                            } else if let Some(Value::Number(n)) = m.get("iterator") {
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
            let est_beats = (entry_stmts.len() as f32) + (loop_iters as f32);
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
            process_play(&config, &entry_file, &output_path, debug)?;

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
    Ok(())
}
