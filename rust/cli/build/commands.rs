use crate::{config::driver::ProjectConfig, core::utils::path::find_entry_file};
use devalang_utils::logger::{LogLevel, Logger};
use devalang_utils::watcher::watch_directory;

#[cfg(feature = "cli")]
pub fn handle_build_command(
    config: Option<ProjectConfig>,
    entry: Option<String>,
    output: Option<String>,
    watch: bool,
    debug: bool,
    compress: bool,
) -> Result<(), String> {
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
                debug,
                compress,
            );
        })
        .unwrap();
    } else {
        let res =
            crate::cli::build::process::process_build(entry_file, fetched_output, debug, compress);
        if let Err(e) = res {
            return Err(e);
        }
    }

    Ok(())
}
