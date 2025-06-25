use std::{ thread, time::Duration };

use crate::{
    core::{
        debugger::Debugger,
        preprocessor::module::load_all_modules,
        types::{
            statement::{
                Statement,
                StatementIterator,
                StatementKind,
                StatementResolved,
                StatementResolvedValue,
            },
            variable::VariableValue,
        },
    },
    runner::executer::execute_statements,
    utils::{ loader::with_spinner, logger::log_message, path::{ find_entry_file, normalize_path } },
};

pub fn handle_check_command(entry: String, output: String) -> () {
    let entry_file = find_entry_file(&entry).unwrap_or_else(|| {
        eprintln!("❌ index.deva not found in directory: {}", entry);
        std::process::exit(1);
    });

    let spinner = with_spinner("Checking...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry_file);
    let normalized_output_dir = normalize_path(&output);

    let global_store = load_all_modules(&normalized_entry_file);

    if let Some(module) = global_store.modules.get(&normalized_entry_file) {
        let mut module_clone = module.clone();

        let resolved_statements = execute_statements(&mut module_clone);

        let debugger = Debugger::new(&module_clone);
        let debug_dir = format!("{}/debug/", normalized_output_dir.clone());
        debugger.write_files(debug_dir.as_str(), resolved_statements.clone());

        let has_errors = resolved_statements.iter().any(|stmt| {
            match_error_recursively_resolved(&stmt.clone())
        });

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
