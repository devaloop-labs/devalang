use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::module::Module,
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::{ Logger, LogLevel },
};

pub fn resolve_call(
    stmt: &Statement,
    name: String,
    args: Vec<Value>,
    module: &Module,
    _path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    match &stmt.kind {
        StatementKind::Call { .. } => {
            // Check if it's a function
            if let Some(func) = global_store.functions.functions.get(&name) {
                let mut call_map = std::collections::HashMap::new();
                call_map.insert("name".to_string(), Value::Identifier(name.clone()));
                call_map.insert(
                    "parameters".to_string(),
                    Value::Array(
                        func.parameters
                            .iter()
                            .map(|p| Value::Identifier(p.clone()))
                            .collect()
                    )
                );
                call_map.insert("args".to_string(), Value::Array(args.clone()));
                call_map.insert("body".to_string(), Value::Block(func.body.clone()));

                return Statement {
                    kind: StatementKind::Call { name, args },
                    value: Value::Map(call_map),
                    ..stmt.clone()
                };
            }

            // Otherwise, check if it's a variable (e.g. group)
            if let Some(variable) = global_store.variables.variables.get(&name) {
                if let Value::Statement(stmt_box) = variable {
                    if let StatementKind::Group = stmt_box.kind {
                        if let Value::Map(map) = &stmt_box.value {
                            if let Some(Value::Block(body)) = map.get("body") {
                                let mut resolved_map = std::collections::HashMap::new();
                                resolved_map.insert(
                                    "identifier".to_string(),
                                    Value::String(name.clone())
                                );
                                resolved_map.insert("args".to_string(), Value::Array(args.clone()));
                                resolved_map.insert("body".to_string(), Value::Block(body.clone()));

                                return Statement {
                                    kind: StatementKind::Call { name, args },
                                    value: Value::Map(resolved_map),
                                    ..stmt.clone()
                                };
                            }
                        }
                    }
                }
            }

            // Otherwise, log an error
            logger.log_message(LogLevel::Error, &format!("Function or group '{}' not found", name));
            Statement {
                kind: StatementKind::Error {
                    message: format!("Function or group '{}' not found", name),
                },
                value: Value::Null,
                ..stmt.clone()
            }
        }
        _ => {
            let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
            logger.log_message(
                LogLevel::Error,
                &format!(
                    "Expected StatementKind::Call in resolve_call()\n  → at {stacktrace}"
                )
            );

            Statement {
                kind: StatementKind::Error {
                    message: "Expected StatementKind::Call in resolve_call()".to_string(),
                },
                value: Value::Null,
                ..stmt.clone()
            }
        },
    }
}

// (removed unused helpers get_group_body, error_stmt)
