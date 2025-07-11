use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{ module::Module, resolver::trigger::resolve_trigger },
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_group(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &GlobalStore
) -> Statement {
    let logger = Logger::new();

    if let Value::Map(value_map) = &stmt.value {
        let group_name = match value_map.get("identifier") {
            Some(Value::String(name)) => name.clone(),
            Some(other) => {
                log_type_error(
                    &logger,
                    module,
                    stmt,
                    format!("Group name must be a string, found {:?}", other)
                );
                return stmt.clone();
            }
            None => {
                log_type_error(&logger, module, stmt, "Group name is required".to_string());
                return stmt.clone();
            }
        };

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
                            println!("Unhandled group body statement: {:?}", statement);
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
                    format!("Unexpected value for group body: {:?}", other)
                );
                Value::Null
            }
            None => {
                log_type_error(
                    &logger,
                    module,
                    stmt,
                    "Missing 'body' key in group statement map".to_string()
                );
                Value::Null
            }
        };

        let mut resolved_map = HashMap::new();

        resolved_map.insert("identifier".to_string(), Value::String(group_name));
        resolved_map.insert("body".to_string(), body_value);

        return Statement {
            kind: StatementKind::Group,
            value: Value::Map(resolved_map),
            ..stmt.clone()
        };
    } else {
        log_type_error(
            &logger,
            module,
            stmt,
            format!("Expected Map for group statement, found {:?}", stmt.value)
        );

        Statement {
            kind: StatementKind::Error {
                message: "Expected a map for group statement".to_string(),
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
