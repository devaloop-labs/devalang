use std::{ thread, time::Duration };

use crate::{
    core::{
        debugger::Debugger,
        preprocessor::module::load_all_modules,
        types::statement::{ StatementKind, StatementResolved, StatementResolvedValue },
    },
    runner::executer::execute_statements,
    utils::{
        config::load_config,
        loader::with_spinner,
        logger::log_message,
        path::{ find_entry_file, normalize_path },
        watcher::watch_directory,
    },
};

pub fn handle_check_command(entry: Option<String>, output: Option<String>, watch: bool) -> () {
    let config = load_config(None);

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

    if fetched_entry.is_empty() {
        eprintln!("❌ Entry path is not specified. Please provide a valid entry path.");
        std::process::exit(1);
    }

    if fetched_output.is_empty() {
        eprintln!("❌ Output directory is not specified. Please provide a valid output directory.");
        std::process::exit(1);
    }

    let entry_file = find_entry_file(&fetched_entry).unwrap_or_else(|| {
        eprintln!("❌ index.deva not found in directory: {}", fetched_entry);
        std::process::exit(1);
    });

    if fetched_watch.clone() == true {
        log_message("Watch mode enabled, waiting for file changes...", "INFO");

        begin_check(entry_file.clone(), fetched_output.clone(), fetched_watch.clone());

        watch_directory(entry_file.clone(), move || {
            log_message("File change detected, rebuilding...", "INFO");
            begin_check(entry_file.clone(), fetched_output.clone(), fetched_watch.clone());
        }).unwrap();
    } else {
        begin_check(entry_file.clone(), fetched_output.clone(), fetched_watch.clone());
    }
}

fn begin_check(entry: String, output: String, watch: bool) {
    let spinner = with_spinner("Checking...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry);
    let normalized_output_dir = normalize_path(&output);

    let global_store = load_all_modules(&normalized_entry_file);

    if let Some(module) = global_store.modules.get(&normalized_entry_file) {
        let mut module_clone = module.clone();

        let resolved_statements = execute_statements(&mut module_clone);

        let debugger = Debugger::new(&module_clone);
        let debug_dir = format!("{}/debug/", normalized_output_dir.clone());
        debugger.write_files(debug_dir.as_str(), resolved_statements.clone());

        let has_errors = resolved_statements
            .iter()
            .any(|stmt| { match_error_recursively_resolved(&stmt.clone()) });

        if has_errors {
            let warning_message = format!(
                "Check completed with errors in {:.2?}. Output files written to: '{}'",
                duration.elapsed(),
                normalized_output_dir
            );

            log_message(&warning_message, "WARNING");
        } else {
            let success_message = format!(
                "Check completed successfully in {:.2?}. Output files written to: '{}'",
                duration.elapsed(),
                normalized_output_dir
            );

            log_message(&success_message, "SUCCESS");
        }
    }
}

fn match_error_recursively_resolved(stmt: &StatementResolved) -> bool {
    match stmt.value.clone() {
        // TODO Other statement value types here

        StatementResolvedValue::Map(map) => {
            for (key, value) in map {
                if match_error_recursively_resolved_value(&value) {
                    return true;
                }
            }
        }

        StatementResolvedValue::Array(array) => {
            for item in array {
                if match_error_recursively_resolved(&item) {
                    return true;
                }
            }
        }

        _ => {
            if let StatementKind::Error = stmt.kind {
                eprintln!("❌ Error found in statement: {:?}", stmt);
                return true;
            }
        }
    }

    false
}

fn match_error_recursively_resolved_value(value: &StatementResolvedValue) -> bool {
    match value {
        StatementResolvedValue::Map(map) => {
            for (_, v) in map {
                if match_error_recursively_resolved_value(v) {
                    return true;
                }
            }
        }

        StatementResolvedValue::Array(array) => {
            for item in array {
                if match_error_recursively_resolved(item) {
                    return true;
                }
            }
        }
        _ => {}
    }

    false
}
