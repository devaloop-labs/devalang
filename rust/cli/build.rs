use std::{ thread, time::Duration };

use crate::{
    core::{
        builder::{ build_ast, write_ast_to_file },
        debugger::Debugger,
        preprocessor::module::load_all_modules,
    },
    runner::executer::execute_statements,
    utils::{ loader::with_spinner, logger::log_message, path::{ find_entry_file, normalize_path } },
};

pub fn handle_build_command(entry: String, output: String) {
    let entry_file = find_entry_file(&entry).unwrap_or_else(|| {
        eprintln!("❌ index.deva not found in directory: {}", entry);
        std::process::exit(1);
    });

    let spinner = with_spinner("Building...", "✅ Build finished !", || {
        // Simulation d’un traitement long
        thread::sleep(Duration::from_millis(800));
    });

    let duration = std::time::Instant::now();

    let normalized_entry_file = normalize_path(&entry_file);
    let normalized_output_dir = normalize_path(&output);

    // 📦 Charge tous les modules + résout les imports
    let global_store = load_all_modules(&normalized_entry_file);

    if let Some(module) = global_store.modules.get(&normalized_entry_file) {
        let module_clone = module.clone();

        // Exécute les statements du module
        let resolved_statements = execute_statements(&module_clone);

        // Construit l'AST à partir des statements résolus
        let ast = build_ast(&resolved_statements);

        // Écrit l'AST dans un fichier
        let ast_dir = format!("{}/json", normalized_output_dir.clone());
        write_ast_to_file(&ast, &ast_dir);

        // Exécute le débogueur
        let debugger = Debugger::new(&module_clone);
        let debug_dir = format!("{}/debug/", normalized_output_dir.clone());
        debugger.write_files(debug_dir.as_str(), resolved_statements);

        // Affiche le message de succès
        let success_message = format!(
            "Build completed successfully in {:.2?}. Output files written to: '{}'",
            duration.elapsed(),
            normalized_output_dir
        );
        log_message(&success_message, "SUCCESS");
    }
}
