use std::collections::HashMap;
use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{
            loader::ModuleLoader,
            module::Module,
            resolver::{
                bank::resolve_bank,
                call::resolve_call,
                condition::resolve_condition,
                group::resolve_group,
                let_::resolve_let,
                loop_::resolve_loop,
                spawn::resolve_spawn,
                tempo::resolve_tempo,
                trigger::resolve_trigger,
            },
        },
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_all_modules(module_loader: &ModuleLoader, global_store: &mut GlobalStore) {
    for module in global_store.clone().modules.values_mut() {
        resolve_imports(module_loader, global_store);
    }
}

pub fn resolve_statement(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    match &stmt.kind {
        StatementKind::Trigger { entity, duration } =>
            resolve_trigger(stmt, entity, &mut duration.clone(), module, path, global_store),
        StatementKind::If => resolve_condition(stmt, module, path, global_store),
        StatementKind::Group => resolve_group(stmt, module, path, global_store),
        StatementKind::Call => resolve_call(stmt, module, path, global_store),
        StatementKind::Spawn => resolve_spawn(stmt, module, path, global_store),
        StatementKind::Bank => resolve_bank(stmt, module, path, global_store),
        StatementKind::Tempo => resolve_tempo(stmt, module, path, global_store),
        StatementKind::Loop => resolve_loop(stmt, module, path, global_store),
        StatementKind::Let { name, .. } => resolve_let(stmt, name, module, path, global_store),

        _ => {
            let resolved_value = resolve_value(&stmt.value, module, global_store);

            Statement {
                value: resolved_value,
                ..stmt.clone()
            }
        }
    }
}

fn resolve_value(value: &Value, module: &Module, global_store: &mut GlobalStore) -> Value {
    match value {
        Value::Identifier(name) => {
            if let Some(original_val) = module.variable_table.get(name) {
                return resolve_value(original_val, module, global_store);
            }

            if let Some(export_val) = find_export_value(name, global_store) {
                return resolve_value(&export_val, module, global_store);
            }

            eprintln!("⚠️ Unresolved identifier '{}'", name);
            Value::Null
        }

        Value::Beat(beat_str) => {
            println!("[warn] '{:?}': unresolved beat '{}'", module.path, beat_str);
            Value::Beat(beat_str.clone())
        }

        Value::Map(map) => {
            let mut resolved = HashMap::new();
            for (k, v) in map {
                resolved.insert(k.clone(), resolve_value(v, module, global_store));
            }
            Value::Map(resolved)
        }

        Value::Block(stmts) => {
            let resolved_stmts = stmts
                .iter()
                .map(|stmt| resolve_statement(stmt, module, &module.path, global_store))
                .collect();
            Value::Block(resolved_stmts)
        }

        other => other.clone(),
    }
}

fn find_export_value(name: &str, global_store: &GlobalStore) -> Option<Value> {
    for (_path, module) in &global_store.modules {
        if let Some(val) = module.export_table.get_export(name) {
            return Some(val.clone());
        }
    }
    None
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
    let snapshot = global_store.clone();

    // 1. Imports resolution
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

    // 2. Statements resolution
    let mut resolved_map: HashMap<String, Vec<Statement>> = HashMap::new();
    for (path, module) in global_store.modules.clone() {
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
                        global_store
                    );
                    resolved.push(resolved_stmt);
                }

                StatementKind::Loop => {
                    let resolved_stmt = resolve_loop(&stmt, &module, &path, global_store);
                    resolved.push(resolved_stmt);
                }

                StatementKind::Bank => {
                    let resolved_stmt = resolve_bank(&stmt, &module, &path, global_store);
                    resolved.push(resolved_stmt);
                }

                StatementKind::Tempo => {
                    let resolved_stmt = resolve_tempo(&stmt, &module, &path, global_store);
                    resolved.push(resolved_stmt);
                }

                StatementKind::Import { .. } | StatementKind::Export { .. } => {
                    resolved.push(stmt.clone());
                }

                StatementKind::Group => {
                    let resolved_stmt = resolve_group(&stmt, &module, &path, global_store);
                    resolved.push(resolved_stmt);
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
