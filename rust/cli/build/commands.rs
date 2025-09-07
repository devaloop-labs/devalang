use crate::config::driver::ProjectConfig;
use devalang_utils::logger::{LogLevel, Logger};
use devalang_utils::path::find_entry_file;
use devalang_utils::watcher::watch_directory;

#[cfg(feature = "cli")]
pub fn handle_build_command(
    config: Option<ProjectConfig>,
    entry: Option<String>,
    output: Option<String>,
    output_format: Vec<crate::cli::parser::OutputFormat>,
    audio_format: crate::cli::parser::AudioFormat,
    sample_rate: u32,
    watch: bool,
    debug: bool,
    compress: bool,
) -> Result<(), String> {
    // determine fetched values from config or CLI
    let fetched_entry = if entry.is_none() {
        config
            .as_ref()
            .and_then(|c| c.defaults.entry.clone())
            .unwrap_or_default()
    } else {
        entry.clone().unwrap_or_default()
    };

    let fetched_output = if output.is_none() {
        config
            .as_ref()
            .and_then(|c| c.defaults.output.clone())
            .unwrap_or_default()
    } else {
        output.clone().unwrap_or_default()
    };

    let fetched_watch = if watch {
        watch
    } else {
        config
            .as_ref()
            .and_then(|c| c.defaults.watch)
            .unwrap_or(false)
    };

    let logger = Logger::new();

    // Determine final audio_format: prefer CLI, else config default if present
    let mut final_audio_format = audio_format;
    if let Some(cfg) = config.as_ref() {
        if let Some(af) = cfg.defaults.audio_format.as_ref() {
            // Only override if CLI provided an empty/placeholder — clap provides a default, so we only override when CLI wasn't explicit (rare).
            // We'll accept values: "wav16", "wav24", "wav32" (case-insensitive)
            let af_low = af.to_lowercase();
            final_audio_format = match af_low.as_str() {
                "wav24" => crate::cli::parser::AudioFormat::Wav24,
                "wav32" => crate::cli::parser::AudioFormat::Wav32,
                _ => crate::cli::parser::AudioFormat::Wav16,
            };
        }
    }

    // Determine final output_format: prefer CLI vector, else config default list
    let mut final_output_format: Vec<crate::cli::parser::OutputFormat> = output_format.clone();
    if final_output_format.is_empty() {
        if let Some(cfg) = config.as_ref() {
            if let Some(ofs) = cfg.defaults.output_format.as_ref() {
                for s in ofs {
                    match s.to_lowercase().as_str() {
                        "mid" | "midi" => {
                            final_output_format.push(crate::cli::parser::OutputFormat::Mid);
                        }
                        _ => final_output_format.push(crate::cli::parser::OutputFormat::Wav),
                    }
                }
            }
        }
    }

    if fetched_entry.is_empty() {
        logger.log_message(
            LogLevel::Error,
            "Entry path is not specified. Please provide a valid entry path.",
        );
        return Err("missing entry path".to_string());
    }
    if fetched_output.is_empty() {
        logger.log_message(
            LogLevel::Error,
            "Output directory is not specified. Please provide a valid output directory.",
        );
        return Err("missing output directory".to_string());
    }

    let entry_file = match find_entry_file(&fetched_entry) {
        Some(p) => p,
        None => {
            logger.log_message(
                LogLevel::Error,
                &format!("❌ index.deva not found in directory: {}", fetched_entry),
            );
            return Err("index.deva not found".to_string());
        }
    };

    // SECTION Begin build
    if fetched_watch {
        let _ = crate::cli::build::process::process_build(
            entry_file.clone(),
            fetched_output.clone(),
            final_output_format.clone(),
            final_audio_format,
            sample_rate,
            debug,
            compress,
        );

        logger.log_message(
            LogLevel::Watcher,
            &format!("Watching for changes in '{}'...", fetched_entry),
        );

        watch_directory(entry_file.clone(), move || {
            logger.log_message(LogLevel::Watcher, "Detected changes, re-building...");

            let _ = crate::cli::build::process::process_build(
                entry_file.clone(),
                fetched_output.clone(),
                final_output_format.clone(),
                final_audio_format,
                sample_rate,
                debug,
                compress,
            );
        })
        .unwrap();
    } else {
        let res = crate::cli::build::process::process_build(
            entry_file,
            fetched_output,
            final_output_format,
            final_audio_format,
            sample_rate,
            debug,
            compress,
        );
        if let Err(e) = res {
            return Err(e);
        }
    }

    Ok(())
}
