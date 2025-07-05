use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{ module::Module, resolver::trigger::resolve_trigger },
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_loop(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &GlobalStore
) -> Statement {
    let logger = Logger::new();

    // Vérifie que stmt.value est bien une Map
    if let Value::Map(value_map) = &stmt.value {
        // Résolution de l'iterator
        let iterator_value = match value_map.get("iterator") {
            Some(Value::Identifier(ident)) => {
                match module.variable_table.get(ident) {
                    Some(Value::Number(n)) => Value::Number(*n),
                    Some(_) => {
                        log_type_error(
                            &logger,
                            module,
                            stmt,
                            format!("Loop iterator '{ident}' must resolve to a number")
                        );
                        Value::Null
                    }
                    None => {
                        // Value is not a variable so we assume it's a number
                        if let Ok(n) = ident.parse::<f32>() {
                            Value::Number(n)
                        } else {
                            log_type_error(
                                &logger,
                                module,
                                stmt,
                                format!("Loop iterator '{ident}' is not a valid number")
                            );
                            Value::Null
                        }
                    }
                }
            }
            Some(Value::Number(n)) => Value::Number(*n),
            Some(other) => {
                log_type_error(
                    &logger,
                    module,
                    stmt,
                    format!("Unexpected value for loop iterator: {:?}", other)
                );
                Value::Null
            }
            None => {
                log_type_error(
                    &logger,
                    module,
                    stmt,
                    "Missing 'iterator' key in loop statement map".to_string()
                );
                Value::Null
            }
        };

        // Résolution du body
        let body_value = match value_map.get("body") {
            Some(Value::Block(block)) => {
                let mut resolved_block = Vec::new();
                for ref statement in block.clone() {
                    match &statement.kind {
                        StatementKind::Trigger { entity, duration } => {
                            let resolved = resolve_trigger(
                                &mut statement.clone(),
                                &entity,
                                &mut duration.clone(),
                                module,
                                path,
                                global_store
                            );
                            resolved_block.push(resolved);
                        }
                        _ => {
                            println!("Unhandled loop body statement: {:?}", statement);
                        }
                    }
                }
                Value::Block(resolved_block)
            }
            Some(other) => {
                log_type_error(
                    &logger,
                    module,
                    stmt,
                    format!("Unexpected value for loop body: {:?}", other)
                );
                Value::Null
            }
            None => {
                log_type_error(
                    &logger,
                    module,
                    stmt,
                    "Missing 'body' key in loop statement map".to_string()
                );
                Value::Null
            }
        };

        // ✅ Reconstruit proprement la valeur résolue
        let mut resolved_map = std::collections::HashMap::new();
        resolved_map.insert("iterator".to_string(), iterator_value);
        resolved_map.insert("body".to_string(), body_value);

        // ✅ Reconstruit le StatementLoop à partir des éléments résolus
        Statement {
            kind: StatementKind::Loop,
            value: Value::Map(resolved_map),
            ..stmt.clone()
        }
    } else {
        log_type_error(
            &logger,
            module,
            stmt,
            format!("Expected a map for loop value, found {:?}", stmt.value)
        );

        Statement {
            kind: StatementKind::Error {
                message: "Expected a map for loop value".to_string(),
            },
            value: Value::Null,
            ..stmt.clone()
        }
    }
}

fn log_type_error(logger: &Logger, module: &Module, stmt: &Statement, message: String) {
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_error_with_stacktrace(&message, &stacktrace);
}
