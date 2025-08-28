use crate::{
    config::driver::ProjectConfig,
    core::{
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
use std::{thread, time::Duration};

#[cfg(feature = "cli")]
pub fn handle_check_command(
    config: Option<ProjectConfig>,
    entry: Option<String>,
    output: Option<String>,
    watch: bool,
    debug: bool,
) -> Result<(), String> {
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

    // SECTION Begin check
    if fetched_watch {
        let _ = begin_check(
            entry_file.clone(),
            fetched_output.clone(),
            debug,
            config.clone(),
        );

        logger.log_message(
            LogLevel::Watcher,
            &format!("Watching for changes in '{}'...", fetched_entry),
        );

        let cfg_for_watch = config.clone();
        watch_directory(entry_file.clone(), move || {
            logger.log_message(LogLevel::Watcher, "Detected changes, re-checking...");
            if let Err(e) = begin_check(
                entry_file.clone(),
                fetched_output.clone(),
                debug,
                cfg_for_watch.clone(),
            ) {
                eprintln!("[check] failed: {}", e);
            }
        })
        .unwrap();
    } else {
        begin_check(
            entry_file.clone(),
            fetched_output.clone(),
            debug,
            config.clone(),
        )?;
    }
    Ok(())
}

fn begin_check(
    entry: String,
    output: String,
    debug: bool,
    config: Option<ProjectConfig>,
) -> Result<(), String> {
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
            modules.0.clone(),
        );
        write_preprocessor_log_file(
            &normalized_output_dir,
            "resolved_statements.log",
            modules.1.clone(),
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

    let all_errors = crate::utils::error::collect_all_errors_with_modules(&modules.1);

    let (warnings, criticals) = crate::utils::error::partition_errors(all_errors);
    crate::utils::error::log_errors_with_stack("Check", &warnings, &criticals);

    if !criticals.is_empty() {
        spinner.finish_and_clear();
        return Err("check failed with critical errors".to_string());
    } else {
        logger.log_message(LogLevel::Success, "No errors detected.");

        // Compute and persist rich stats
        let stats = crate::config::stats::compute_from(
            &modules.1,
            &global_store,
            &config,
            Some(&normalized_output_dir),
        );
        crate::config::stats::set_memory_stats(stats.clone());
        if let Err(e) = crate::config::stats::save_to_file(&stats) {
            eprintln!("[stats] failed to save: {}", e);
        }

        let success_message = format!(
            "Check completed successfully in {:.2?}. Output files written to: '{}'",
            duration.elapsed(),
            normalized_output_dir
        );

        spinner.finish_and_clear();
        logger.log_message(LogLevel::Success, &success_message);
        Ok(())
    }
}
