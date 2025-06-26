use std::{ thread, time::Duration };

use crate::{
    core::{
        builder::{ build_ast, write_ast_to_file },
        debugger::Debugger,
        preprocessor::module::load_all_modules,
    },
    runner::executer::execute_statements,
    utils::{
        loader::with_spinner,
        logger::log_message,
        path::{ find_entry_file, normalize_path },
        watcher::watch_directory,
    },
};

pub fn handle_build_command(entry: String, output: String, watch: bool) {
    let entry_file = find_entry_file(&entry).unwrap_or_else(|| {
        eprintln!("❌ index.deva not found in directory: {}", entry);
        std::process::exit(1);
    });

    if watch == true {
        log_message("Watch mode enabled, waiting for file changes...", "INFO");

        begin_build(entry_file.clone(), output.clone(), watch);

        watch_directory(entry_file.clone(), move || {
            log_message("File change detected, rebuilding...", "INFO");
            begin_build(entry_file.clone(), output.clone(), watch);
        }).unwrap();
    } else {
        begin_build(entry_file.clone(), output.clone(), watch);
    }
}

fn begin_build(entry: String, output: String, watch: bool) {
    let spinner = with_spinner("Building...", || {
        thread::sleep(Duration::from_millis(800));
    });

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry);
    let normalized_output_dir = normalize_path(&output);

    let global_store = load_all_modules(&normalized_entry_file);

    if let Some(module) = global_store.modules.get(&normalized_entry_file) {
        let mut module_clone = module.clone();

        let resolved_statements = execute_statements(&mut module_clone);

        let ast = build_ast(&resolved_statements);

        let ast_dir = format!("{}/json", normalized_output_dir.clone());
        write_ast_to_file(&ast, &ast_dir);

        let debugger = Debugger::new(&module_clone);
        let debug_dir = format!("{}/debug/", normalized_output_dir.clone());
        debugger.write_files(debug_dir.as_str(), resolved_statements);

        let success_message = format!(
            "Build completed successfully in {:.2?}. Output files written to: '{}'",
            duration.elapsed(),
            normalized_output_dir
        );
        log_message(&success_message, "SUCCESS");
    }
}
