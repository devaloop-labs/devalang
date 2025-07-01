use crate::{
    config::Config,
    core::{
        builder::Builder,
        preprocessor::loader::ModuleLoader,
        store::global::GlobalStore,
        utils::path::{ find_entry_file, normalize_path },
    },
    utils::{ logger::{ LogLevel, Logger }, spinner::with_spinner, watcher::watch_directory },
};
use std::{ thread, time::Duration };

pub fn handle_build_command(
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

    // SECTION Begin build
    if fetched_watch {
        begin_build(entry_file.clone(), fetched_output.clone());

        logger.log_message(
            LogLevel::Watcher,
            &format!("Watching for changes in '{}'...", fetched_entry)
        );

        watch_directory(entry_file.clone(), move || {
            logger.log_message(LogLevel::Watcher, "Detected changes, re-building...");

            begin_build(entry_file.clone(), fetched_output.clone());
        }).unwrap();
    } else {
        begin_build(entry_file.clone(), fetched_output.clone());
    }
}

fn begin_build(entry: String, output: String) {
    let spinner = with_spinner("Building...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry);
    let normalized_output_dir = normalize_path(&output);

    let mut global_store = GlobalStore::new();
    let module_loader = ModuleLoader::new(&normalized_entry_file, &normalized_output_dir);

    // SECTION Load
    // NOTE: We use modules in the build command, so we need to load them
    let modules = module_loader.load_all(&mut global_store);

    // SECTION Build
    let builder = Builder::new();
    builder.build_ast(&modules);

    // TODO: Implement debugging

    let success_message = format!(
        "Build completed successfully in {:.2?}. Output files written to: '{}'",
        duration.elapsed(),
        normalized_output_dir
    );

    let logger = Logger::new();
    logger.log_message(LogLevel::Success, &success_message);
}
