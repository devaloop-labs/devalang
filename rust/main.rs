pub mod core;

use std::fs;
use crate::core::{
    lexer::lex,
    parser::parse,
    preprocessor::{ collect_dependencies_recursively, module::load_all_modules, preprocess },
    types::{
        module::Module,
        parser::Parser,
        statement::{ Statement, StatementKind },
        store::{ ExportTable, GlobalStore, ImportTable, VariableTable },
        token::Token,
        variable::VariableValue,
    },
};

fn main() {
    let entry_file = "./examples/index.deva";

    let global_store = load_all_modules(entry_file); // ← tout est géré là

    println!("\n✅ Résumé des modules chargés :\n");
    for (path, module) in &global_store.modules {
        println!("📁 {}", path);
        println!("  ▸ {} statements", module.statements.len());
        println!("  🔹 Variables: {:?}", module.variable_table.variables);
        println!("  🔸 Exports  : {:?}", module.export_table.exports);
        println!("  🔸 Imports  : {:?}", module.import_table.imports);
        println!();
    }
}

pub fn resolve_exports(statements: &[Statement], variable_table: &VariableTable) -> ExportTable {
    let mut table = ExportTable::default();

    for stmt in statements {
        if let StatementKind::Export = &stmt.kind {
            if let VariableValue::Array(tokens) = &stmt.value {
                for token in tokens {
                    let var_name = &token.lexeme;
                    if let Some(value) = variable_table.variables.get(var_name) {
                        table.exports.insert(var_name.clone(), value.clone());
                    } else {
                        eprintln!("⚠️ Variable '{}' not found in scope, export skipped", var_name);
                    }
                }
            }
        }
    }

    table
}

fn resolve_imports(module: &Module, global_store: &mut GlobalStore) -> ImportTable {
    let mut import_table = module.import_table.clone();
    let mut variable_table = module.variable_table.clone();

    println!("{:?}", global_store.modules.keys());

    println!("Resolving imports for module '{}'", module.path);
    for stmt in &module.statements {
        if let StatementKind::Import { names, source } = &stmt.kind {
            println!("  ↳ trying to import {:?} from {}", names, source);
            if !global_store.modules.contains_key(source) {
                println!("  ❌ source module '{}' NOT FOUND", source);
            } else {
                println!("  ✅ source module '{}' found", source);
                if let Some(from_module) = global_store.modules.get(source) {
                    for name in names {
                        if let Some(value) = from_module.export_table.exports.get(name) {
                            import_table.imports.insert(name.clone(), value.clone());
                            variable_table.variables.insert(name.clone(), value.clone());
                            println!(
                                "🔍 EXPORTS from {}: {:?}",
                                source,
                                from_module.export_table.exports
                            );
                            println!("  ↳ Imported '{}' from '{}'", name, source);
                        } else {
                            eprintln!("⚠️ Variable '{}' not found in module '{}'", name, source);
                        }
                    }
                } else {
                    eprintln!("⚠️ Module '{}' not found", source);
                }
            }
        }
    }

    import_table
}
