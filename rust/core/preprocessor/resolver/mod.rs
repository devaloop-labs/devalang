pub mod trigger;
pub mod loop_;
pub mod bank;
pub mod tempo;

use std::collections::HashMap;
use crate::{
    core::{
        parser::statement::{ self, Statement, StatementKind },
        preprocessor::{
            loader::ModuleLoader,
            resolver::{
                bank::resolve_bank,
                loop_::resolve_loop,
                tempo::resolve_tempo,
                trigger::resolve_trigger,
            },
        },
        shared::{ duration::Duration, value::Value },
        store::global::GlobalStore,
        utils::validation::{ is_valid_entity, is_valid_identifier },
    },
    utils::logger::Logger,
};

pub fn resolve_all_modules(module_loader: &ModuleLoader, global_store: &mut GlobalStore) {
    for module in global_store.clone().modules.values_mut() {
        resolve_imports(module_loader, global_store);
    }
}

pub fn resolve_imports(module_loader: &ModuleLoader, global_store: &mut GlobalStore) {
    for (module_path, module) in global_store.clone().modules.iter_mut() {
        for (name, source_path) in &module.import_table.imports {
            match source_path {
                Value::String(source_path) => {
                    if let Some(source_module) = global_store.modules.get(source_path) {
                        if let Some(value) = source_module.export_table.get_export(name) {
                            module.variable_table.set(name.clone(), value.clone());
                        } else {
                            println!(
                                "[warn] '{module_path}': '{name}' not found in exports of '{source_path}'"
                            );
                        }
                    } else {
                        println!(
                            "[warn] '{module_path}': cannot find source module '{source_path}'"
                        );
                    }
                }
                _ => {
                    println!(
                        "[warn] '{module_path}': expected string for import source, found {:?}",
                        source_path
                    );
                }
            }
        }
    }
}

pub fn resolve_and_flatten_all_modules(
    global_store: &mut GlobalStore
) -> HashMap<String, Vec<Statement>> {
    let logger = Logger::new();
    let snapshot = global_store.clone(); // pour éviter les emprunts mutables

    // 1. Résolution des imports
    for (module_path, module) in global_store.modules.iter_mut() {
        for (name, source_path) in &module.import_table.imports {
            if let Value::String(source_path_str) = source_path {
                match snapshot.modules.get(source_path_str) {
                    Some(source_module) => {
                        if let Some(value) = source_module.export_table.get_export(name) {
                            module.variable_table.set(name.clone(), value.clone());
                        } else {
                            logger.log_error_with_stacktrace(
                                &format!("'{name}' not found in exports of '{source_path_str}'"),
                                module_path
                            );
                        }
                    }
                    None => {
                        logger.log_error_with_stacktrace(
                            &format!("Cannot find source module '{source_path_str}'"),
                            module_path
                        );
                    }
                }
            } else {
                logger.log_error_with_stacktrace(
                    &format!("Expected string for import source, found {:?}", source_path),
                    module_path
                );
            }
        }
    }

    // 2. Résolution des statements
    let mut resolved_map = HashMap::new();
    let store_snapshot = global_store.clone();

    for (path, module) in &store_snapshot.modules {
        let mut resolved = Vec::new();

        for stmt in &module.statements {
            let mut stmt = stmt.clone();

            match &stmt.kind {
                StatementKind::Trigger { entity, duration } => {
                    let resolved_stmt = resolve_trigger(
                        &stmt,
                        entity.as_str(),
                        &mut duration.clone(),
                        &module,
                        &path,
                        &store_snapshot
                    );
                    resolved.push(resolved_stmt);
                }

                StatementKind::Loop => {
                    let resolved_stmt = resolve_loop(&stmt, &module, &path, &store_snapshot);
                    resolved.push(resolved_stmt);
                }

                StatementKind::Bank => {
                    let resolved_stmt = resolve_bank(&stmt, &module, &path, &store_snapshot);
                    resolved.push(resolved_stmt);
                }

                StatementKind::Tempo => {
                    let resolved_stmt = resolve_tempo(&stmt, &module, &path, &store_snapshot);
                    resolved.push(resolved_stmt);
                }

                StatementKind::Import { .. } | StatementKind::Export { .. } => {
                    // Rien à faire
                    resolved.push(stmt.clone());
                }

                _ => {
                    resolved.push(stmt);
                }
            }
        }

        resolved_map.insert(path.clone(), resolved);
    }

    resolved_map
}
