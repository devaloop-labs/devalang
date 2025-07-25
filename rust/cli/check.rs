use crate::{
    config::driver::Config,
    core::{
        preprocessor::loader::ModuleLoader,
        store::global::GlobalStore,
        utils::path::{ find_entry_file, normalize_path },
    },
    utils::{
        collect_errors_recursively,
        logger::{ LogLevel, Logger },
        spinner::with_spinner,
        watcher::watch_directory,
    },
};
use std::{ thread, time::Duration };

#[cfg(feature = "cli")]
pub fn handle_check_command(
    config: Option<Config>,
    entry: Option<String>,
    output: Option<String>,
    watch: bool
) {
    let fetched_entry = if entry.is_none() {
        config
            .as_ref()
            .and_then(|c| c.defaults.entry.clone())
            .unwrap_or_else(|| "".to_string())
    } else {
        entry.clone().unwrap_or_else(|| "".to_string())
    };

    let fetched_output = if output.is_none() {
        config
            .as_ref()
            .and_then(|c| c.defaults.output.clone())
            .unwrap_or_else(|| "".to_string())
    } else {
        output.clone().unwrap_or_else(|| "".to_string())
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
            "Entry path is not specified. Please provide a valid entry path."
        );
        std::process::exit(1);
    }
    if fetched_output.is_empty() {
        logger.log_message(
            LogLevel::Error,
            "Output directory is not specified. Please provide a valid output directory."
        );
        std::process::exit(1);
    }

    let entry_file = find_entry_file(&fetched_entry).unwrap_or_else(|| {
        logger.log_message(
            LogLevel::Error,
            &format!("❌ index.deva not found in directory: {}", fetched_entry)
        );
        std::process::exit(1);
    });

    // SECTION Begin check
    if fetched_watch {
        begin_check(entry_file.clone(), fetched_output.clone());

        logger.log_message(
            LogLevel::Watcher,
            &format!("Watching for changes in '{}'...", fetched_entry)
        );

        watch_directory(entry_file.clone(), move || {
            logger.log_message(LogLevel::Watcher, "Detected changes, re-checking...");

            begin_check(entry_file.clone(), fetched_output.clone());
        }).unwrap();
    } else {
        begin_check(entry_file.clone(), fetched_output.clone());
    }
}

fn begin_check(entry: String, output: String) {
    let spinner = with_spinner("Checking...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry);
    let normalized_output_dir = normalize_path(&output);

    let mut global_store = GlobalStore::new();
    let module_loader = ModuleLoader::new(&normalized_entry_file, &normalized_output_dir);

    // SECTION Load
    // NOTE: We don't use modules in the check command, but we still need to load them
    let modules = module_loader.load_all_modules(&mut global_store);

    // Debugging: Log loaded modules and errors
    let logger = Logger::new();
    logger.log_message(LogLevel::Info, "Loaded modules:");
    for (module_name, _) in &modules.0 {
        logger.log_message(LogLevel::Info, &format!("- {}", module_name));
    }

    let mut all_errors = Vec::new();
    for (_, statements) in &modules.1 {
        all_errors.extend(collect_errors_recursively(statements));
    }

    if !all_errors.is_empty() {
        logger.log_message(LogLevel::Error, "Errors detected during check:");
        for error in all_errors {
            logger.log_message(LogLevel::Error, &format!("- {}", error.message));
        }
    } else {
        logger.log_message(LogLevel::Success, "No errors detected.");
    }

    let success_message = format!(
        "Check completed successfully in {:.2?}. Output files written to: '{}'",
        duration.elapsed(),
        normalized_output_dir
    );

    logger.log_message(LogLevel::Success, &success_message);
}
