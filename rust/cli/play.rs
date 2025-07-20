use crate::{
    core::{
        builder::Builder,
        debugger::{ lexer::write_lexer_log_file, preprocessor::write_preprocessor_log_file },
        preprocessor::loader::ModuleLoader,
        store::global::GlobalStore,
        utils::path::{ find_entry_file, normalize_path },
    },
    config::driver::Config,
    utils::{ logger::{ LogLevel, Logger }, spinner::with_spinner, watcher::watch_directory },
};

use std::{ path::Path, sync::mpsc::channel, thread, time::Duration };
use std::fs;
use std::collections::HashMap;

#[cfg(feature = "cli")]
pub fn handle_play_command(
    config: Option<Config>,
    entry: Option<String>,
    output: Option<String>,
    watch: bool,
    repeat: bool
) {
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
        std::process::exit(1);
    }

    let entry_file = find_entry_file(&entry_path).unwrap_or_else(|| {
        logger.log_message(LogLevel::Error, "index.deva not found");
        std::process::exit(1);
    });

    let audio_file = format!("{}/audio/index.wav", normalize_path(&output_path));
    let mut audio_player = AudioPlayer::new();

    if watch && fetched_repeat {
        logger.log_message(
            LogLevel::Error,
            "Watch and repeat cannot be used together. Use repeat instead."
        );
        std::process::exit(1);
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
        begin_play(&config, &entry_file, &output_path);
        audio_player.play_file_once(&audio_file);

        logger.log_message(LogLevel::Watcher, "Watching for changes... Press Ctrl+C to exit.");

        while let Ok(_) = rx.recv() {
            logger.log_message(LogLevel::Watcher, "Change detected, rebuilding...");

            begin_play(&config, &entry_file, &output_path);

            logger.log_message(LogLevel::Info, "🎵 Playback started (once mode)...");

            audio_player.play_file_once(&audio_file);
        }
    } else if fetched_repeat {
        // Initial build to start from a clean slate
        begin_play(&config, &entry_file, &output_path);

        logger.log_message(LogLevel::Info, "🎵 Playback started (repeat mode)...");

        let mut last_snapshot = snapshot_files(&entry_path);
        let mut audio_player = AudioPlayer::new();
        audio_player.play_file_once(&audio_file);

        loop {
            let current_snapshot = snapshot_files(&entry_path);
            let has_changed = files_changed(&last_snapshot, &current_snapshot);

            if has_changed {
                logger.log_message(LogLevel::Info, "Change detected, rebuilding in background...");
                let entry_file = entry_file.clone();
                let output_path = output_path.clone();
                let config_clone = config.clone();

                // Rebuild in a separate thread
                std::thread::spawn(move || {
                    begin_play(&config_clone, &entry_file, &output_path);
                });

                last_snapshot = current_snapshot;
            }

            // Wait for the audio to finish without blocking the current playback
            audio_player.wait_until_end();

            // Then replay the audio (rebuilt or not)
            audio_player.play_file_once(&audio_file);
        }
    } else {
        // Single execution
        begin_play(&config, &entry_file, &output_path);
        
        logger.log_message(LogLevel::Info, "🎵 Playback started (once mode)...");

        audio_player.play_file_once(&audio_file);
        audio_player.wait_until_end();
    }
}

fn begin_play(config: &Option<Config>, entry_file: &str, output: &str) {
    let spinner = with_spinner("Building...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let normalized_entry = normalize_path(entry_file);
    let normalized_output_dir = normalize_path(&output);

    let duration = std::time::Instant::now();
    let mut global_store = GlobalStore::new();
    let loader = ModuleLoader::new(&normalized_entry, &normalized_output_dir);
    let (modules_tokens, modules_statements) = loader.load_all_modules(&mut global_store);

    // SECTION Write logs
    write_lexer_log_file(&normalized_output_dir, "lexer_tokens.log", modules_tokens.clone());
    write_preprocessor_log_file(
        &normalized_output_dir,
        "resolved_statements.log",
        modules_statements.clone()
    );

    // SECTION Building AST and Audio
    let builder = Builder::new();
    builder.build_ast(&modules_statements, &output);
    builder.build_audio(&modules_statements, &output, &mut global_store);

    // SECTION Logging
    let logger = Logger::new();
    let success_message = format!(
        "Build completed successfully in {:.2?}. Output files written to: '{}'",
        duration.elapsed(),
        normalized_output_dir
    );

    logger.log_message(LogLevel::Success, &success_message);
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
