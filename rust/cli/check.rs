use std::{ thread, time::Duration };

use crate::{
    core::{ debugger::Debugger, preprocessor::module::load_all_modules },
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
        let module_clone = module.clone();

        let resolved_statements = execute_statements(&module_clone);

        if resolved_statements.is_empty() {
            log_message("Errors found while checking the module.", "WARNING");
        } else {
            let debugger = Debugger::new(&module_clone);
            let debug_dir = format!("{}/debug/", normalized_output_dir.clone());
            debugger.write_files(debug_dir.as_str(), resolved_statements);

            let success_message = format!(
                "Check completed successfully in {:.2?}. Output files written to: '{}'",
                duration.elapsed(),
                normalized_output_dir
            );
            log_message(&success_message, "SUCCESS");
        }
    }
}
