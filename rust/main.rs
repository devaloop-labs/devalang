pub mod core;

use std::fs;
use crate::core::{
    debugger::Debugger,
    preprocessor::{ module::load_all_modules, resolver::resolve_statement },
    types::{
        module::Module,
        parser::Parser,
        statement::{ Statement, StatementKind, StatementResolved },
        store::{ ExportTable, GlobalStore, ImportTable },
        variable::VariableValue,
    },
};

fn main() {
    let entry_file = "./examples/index.deva";

    // 📦 Charge tous les modules + résout les imports
    let global_store = load_all_modules(entry_file);

    // ✅ Affichage des modules et de leur contenu
    println!("\n✅ Résumé des modules chargés :\n");
    for (path, module) in &global_store.modules {
        println!("📁 {}", path);
        println!("  ▸ {} statements", module.statements.len());
        println!("  🔹 Variables: {:?}", module.variable_table.variables);
        println!("  🔸 Exports  : {:?}", module.export_table.exports);
        println!("  🔸 Imports  : {:?}", module.import_table.imports);
        println!();

        for stmt in &module.statements {
            println!("    → {:?}", stmt);
        }
        println!("\n-----------------------------\n");
    }

    if let Some(module) = global_store.modules.get("./examples/index.deva") {
        let module_clone = module.clone();
        let debugger = Debugger::new(&module_clone);

        // Exécute les statements du module
        let resolved_statements = run_statements(&module_clone, &debugger);

        // Exécute le débogueur
        debugger.run();
        debugger.write_files("./output/debug/", resolved_statements);
    }
}

/// Exécute tous les statements d'un module avec résolution des variables
pub fn run_statements(module: &Module, debugger: &Debugger) -> Vec<StatementResolved> {
    println!("▶️ Executing statements for module: {}", module.path);

    let mut resolved_statements: Vec<StatementResolved> = Vec::new();

    for stmt in &module.statements {
        match &stmt.kind {
            StatementKind::Tempo { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            StatementKind::Trigger { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            StatementKind::Bank { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            StatementKind::Loop { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            _ => {}
        }
    }

    resolved_statements
}
