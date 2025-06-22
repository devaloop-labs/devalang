pub mod core;

use std::fs;
use crate::core::{
    lexer::lex,
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
        run_statements(module);
    }
}

pub fn resolve_exports(statements: &[Statement], parser: &Parser) -> ExportTable {
    let mut export_table = parser.export_table.clone();

    for stmt in statements {
        if let StatementKind::Export = &stmt.kind {
            if let VariableValue::Array(tokens) = &stmt.value {
                for token in tokens {
                    let var_name = &token.lexeme;
                    if let Some(value) = parser.variable_table.variables.get(var_name) {
                        export_table.add_export(var_name.clone(), value.clone());
                    } else {
                        eprintln!("⚠️ Variable '{}' not found in scope, export skipped", var_name);
                    }
                }
            } else {
                eprintln!("⚠️ Unexpected value type in export: {:?}", stmt.value);
            }
        }
    }

    export_table
}

pub fn resolve_imports(module: &mut Module, global_store: &GlobalStore) -> ImportTable {
    println!("Resolving imports for module '{}'", module.path);
    for stmt in &module.statements {
        if let StatementKind::Import { names, source } = &stmt.kind {
            println!("  ↳ trying to import {:?} from {}", names, source);
            if let Some(from_module) = global_store.modules.get(source) {
                for name in names {
                    if let Some(value) = from_module.export_table.exports.get(name) {
                        module.import_table.add_import(name.clone(), value.clone());
                        module.variable_table.variables.insert(name.clone(), value.clone());
                        println!("  ↳ Imported '{}' from '{}': {:?}", name, source, value);
                    } else {
                        eprintln!("⚠️ Variable '{}' not found in module '{}'", name, source);
                    }
                }
            } else {
                eprintln!("  ❌ source module '{}' NOT FOUND", source);
            }
        }
    }

    println!("Imports resolved for module '{}': {:?}", module.path, module.import_table.imports);
    module.import_table.clone()
}

/// Exécute tous les statements d'un module avec résolution des variables
pub fn run_statements(module: &crate::core::types::module::Module) {
    println!("▶️ Execution for module: {}", module.path);

    for stmt in &module.statements {
        // match &stmt.kind {
        //     crate::core::types::statement::StatementKind::Trigger { entity } => {
        //         // On attend une valeur de type Text contenant le nom de variable
        //         // if let crate::core::types::variable::VariableValue::Text(var_name) = &stmt.value {
        //         //     let value = module.import_table.imports
        //         //         .get(var_name)
        //         //         .or_else(|| module.variable_table.variables.get(var_name));

        //         //     match value {
        //         //         Some(v) => println!("▶️ .{}[{}: {:?}]", entity, var_name, v),
        //         //         None => println!("❌ .{}[{}: not found]", entity, var_name),
        //         //     }
        //         // } else {
        //         //     println!("⚠️ .{}[raw value: {:?}]", entity, stmt.value);
        //         // }
        //         if let VariableValue::Text(var_name) = &stmt.value {
        //             let value = module.variable_table.variables.get(var_name);

        //             match value {
        //                 Some(v) => println!("▶️ .{}[{}: {:?}]", entity, var_name, v),
        //                 None => println!("❌ .{}[{}: not found]", entity, var_name),
        //             }
        //         } else {
        //             println!("⚠️ .{}[raw value: {:?}]", entity, stmt.value);
        //         }
        //     }

        //     _ => {
        //         // Tu peux gérer d'autres StatementKind ici si besoin
        //         println!("▶️ Executing statement: {:?} ({:?})", stmt.kind, stmt.value);
        //     }
        // }

        match &stmt.value {
            crate::core::types::variable::VariableValue::Text(text) => {
                println!("  ↳ Text value: {}", text);
            }
            crate::core::types::variable::VariableValue::Number(num) => {
                println!("  ↳ Number value: {}", num);
            }
            crate::core::types::variable::VariableValue::Array(tokens) => {
                println!("  ↳ Array value: {:?}", tokens);
            }
            _ => {
                println!("  ↳ Other value type: {:?}", stmt.value);
            }
        }
    }

    println!("Final module : {:?}", module);
}
