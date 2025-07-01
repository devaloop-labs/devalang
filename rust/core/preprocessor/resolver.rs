use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ self, Statement, StatementKind },
        preprocessor::loader::ModuleLoader,
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

    // 1. Imports resolving
    let global_store_clone = global_store.clone();
    for (module_path, module) in global_store.modules.iter_mut() {
        for (name, source_path) in &module.import_table.imports {
            match source_path {
                Value::String(source_path) => {
                    if let Some(source_module) = global_store_clone.modules.get(source_path) {
                        if let Some(value) = source_module.export_table.get_export(name) {
                            module.variable_table.set(name.clone(), value.clone());
                        } else {
                            let message = format!(
                                "'{name}' not found in exports of '{source_path}'"
                            );
                            logger.log_error_with_stacktrace(&message, module_path);
                        }
                    } else {
                        let message = format!("cannot find source module '{source_path}'");
                        logger.log_error_with_stacktrace(&message, module_path);
                    }
                }
                _ => {
                    let message = format!(
                        "expected string for import source, found {:?}",
                        source_path
                    );
                    logger.log_error_with_stacktrace(&message, module_path);
                }
            }
        }
    }

    // 2. Full statement resolution
    let mut resolved_map = HashMap::new();

    for (path, module) in &global_store.modules {
        let mut resolved = Vec::new();

        for mut stmt in module.statements.clone() {
            match &mut stmt.kind {
                StatementKind::Bank => {
                    if let Value::Identifier(ident) = &stmt.value {
                        if let Some(val) = module.variable_table.get(ident) {
                            stmt.value = val.clone();
                        } else {
                            let message = format!(
                                "Bank identifier '{ident}' not found in variable table"
                            );

                            logger.log_error_with_stacktrace(&message, &module.path);

                            stmt.kind = StatementKind::Error {
                                message: format!(
                                    "Bank identifier '{ident}' not found in variable table"
                                ),
                            };
                            stmt.value = Value::Null;
                        }
                    } else if let Value::String(_) = &stmt.value {
                        // Value is already a string, no need to modify
                    } else {
                        let message = format!(
                            "Expected a string or identifier for bank, found {:?}",
                            stmt.value
                        );

                        logger.log_error_with_stacktrace(&message, &module.path);

                        stmt.kind = StatementKind::Error {
                            message: "Expected a string or identifier for bank".to_string(),
                        };
                        stmt.value = Value::Null;
                    }
                }

                StatementKind::Tempo => {
                    if let Value::Identifier(ident) = &stmt.value {
                        if let Some(val) = module.variable_table.get(ident) {
                            stmt.value = val.clone();
                        } else {
                            let message = format!(
                                "Tempo identifier '{ident}' not found in variable table"
                            );

                            logger.log_error_with_stacktrace(&message, &module.path);

                            stmt.kind = StatementKind::Error {
                                message: format!(
                                    "Tempo identifier '{ident}' not found in variable table"
                                ),
                            };
                            stmt.value = Value::Null;
                        }
                    } else if let Value::Number(_) = &stmt.value {
                        // Value is already a number, no need to modify
                    } else {
                        let message = format!(
                            "Expected a number or identifier for tempo, found {:?}",
                            stmt.value
                        );

                        logger.log_error_with_stacktrace(&message, &module.path);

                        stmt.kind = StatementKind::Error {
                            message: "Expected a number or identifier for tempo".to_string(),
                        };
                        stmt.value = Value::Null;
                    }
                }

                StatementKind::Trigger { entity, duration } => {
                    if !is_valid_entity(entity, module, global_store) {
                        let message =
                            format!("Invalid entity '{}', expected a valid identifier", entity);

                        let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
                        logger.log_error_with_stacktrace(&message, &stacktrace);

                        stmt.kind = StatementKind::Error {
                            message: format!("Invalid entity '{}'", entity),
                        };
                        stmt.value = Value::Null;

                        resolved.push(stmt);
                        continue;
                    }

                    // Duration resolution
                    if let Duration::Identifier(ident) = duration {
                        if let Some(val) = module.variable_table.get(ident) {
                            match val {
                                Value::Number(num) => {
                                    *duration = Duration::Number(*num);
                                }
                                Value::String(s) => {
                                    *duration = Duration::Identifier(s.clone());
                                }
                                Value::Identifier(id) if id == "auto" => {
                                    *duration = Duration::Auto;
                                }
                                _ => {}
                            }
                        }
                    }

                    // Associated value resolution
                    if let Value::Identifier(ident) = &stmt.value {
                        if let Some(val) = module.variable_table.get(ident) {
                            stmt.value = val.clone();
                        } else {
                            let stacktrace = format!(
                                "{}:{}:{}",
                                module.path,
                                stmt.line,
                                stmt.column
                            );
                            let message = format!(
                                "'{path}': value identifier '{ident}' not found in variable table"
                            );

                            logger.log_error_with_stacktrace(&message, &stacktrace);

                            stmt.kind = StatementKind::Error {
                                message: format!(
                                    "Value identifier '{ident}' not found in variable table"
                                ),
                            };
                            stmt.value = Value::Null;
                        }
                    }
                }

                StatementKind::Let { .. } => {
                    if let Value::Identifier(ident) = &stmt.value {
                        if let Some(val) = module.variable_table.get(ident) {
                            stmt.value = val.clone();
                        } else {
                            if !is_valid_identifier(ident, module) {
                                let message = format!(
                                    "'{path}': value identifier '{ident}' not found in variable table"
                                );

                                logger.log_error_with_stacktrace(&message, &module.path);

                                stmt.kind = StatementKind::Error {
                                    message: format!(
                                        "Value identifier '{ident}' not found in variable table"
                                    ),
                                };
                                stmt.value = Value::Null;
                            } else {
                                stmt.value = Value::Identifier(ident.clone());
                            }
                        }
                    }
                }

                StatementKind::Loop => {
                    if let Value::Map(value_map) = &stmt.value {
                        let iterator_value = match value_map.get("iterator") {
                            Some(Value::Identifier(ident)) => {
                                if let Some(val) = module.variable_table.get(ident) {
                                    match val {
                                        Value::Number(n) => Value::Number(*n),
                                        _ => {
                                            let message = format!(
                                                "Loop iterator '{ident}' must resolve to a number"
                                            );
                                            let stacktrace = format!(
                                                "{}:{}:{}",
                                                module.path,
                                                stmt.line,
                                                stmt.column
                                            );
                                            logger.log_error_with_stacktrace(&message, &stacktrace);

                                            stmt.kind = StatementKind::Error {
                                                message,
                                            };
                                            Value::Null
                                        }
                                    }
                                } else {
                                    let message = format!(
                                        "'{path}': loop iterator '{ident}' not found in variable table"
                                    );
                                    let stacktrace = format!(
                                        "{}:{}:{}",
                                        module.path,
                                        stmt.line,
                                        stmt.column
                                    );
                                    logger.log_error_with_stacktrace(&message, &stacktrace);

                                    stmt.kind = StatementKind::Error {
                                        message: format!("Loop iterator '{ident}' not found"),
                                    };
                                    Value::Null
                                }
                            }

                            Some(Value::Number(n)) => Value::Number(*n),

                            Some(v) => {
                                let message = format!(
                                    "Unexpected value for loop iterator: {:?}, expected number or identifier",
                                    v
                                );
                                let stacktrace = format!(
                                    "{}:{}:{}",
                                    module.path,
                                    stmt.line,
                                    stmt.column
                                );
                                logger.log_error_with_stacktrace(&message, &stacktrace);

                                stmt.kind = StatementKind::Error {
                                    message,
                                };
                                Value::Null
                            }

                            None => {
                                let message = format!(
                                    "Missing 'iterator' key in loop statement map"
                                );
                                let stacktrace = format!(
                                    "{}:{}:{}",
                                    module.path,
                                    stmt.line,
                                    stmt.column
                                );
                                logger.log_error_with_stacktrace(&message, &stacktrace);

                                stmt.kind = StatementKind::Error {
                                    message,
                                };
                                Value::Null
                            }
                        };

                        let body_value = value_map.get("body").cloned().unwrap_or(Value::Null);

                        let mut loop_map = std::collections::HashMap::new();
                        loop_map.insert("iterator".to_string(), iterator_value);
                        loop_map.insert("body".to_string(), body_value);

                        stmt.value = Value::Map(loop_map);
                    } else {
                        let message = format!(
                            "'{path}': expected a map for loop value, found {:?}",
                            stmt.value
                        );
                        let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);

                        logger.log_error_with_stacktrace(&message, &stacktrace);

                        stmt.kind = StatementKind::Error {
                            message: "Expected a map for loop value".to_string(),
                        };
                        stmt.value = Value::Null;
                    }
                }

                StatementKind::Import { names, source } => {}

                StatementKind::Export { names, source } => {}

                _ => {}
            }

            resolved.push(stmt);
        }

        resolved_map.insert(path.clone(), resolved);
    }

    resolved_map
}
