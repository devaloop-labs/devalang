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

    let Value::Map(value_map) = &stmt.value else {
        return type_error(&logger, module, stmt, "Expected a map for group statement".to_string());
    };

    let group_name = match value_map.get("identifier") {
        Some(Value::String(name)) => name.clone(),
        Some(other) => {
            return type_error(
                &logger,
                module,
                stmt,
                format!("Group name must be a string, found {:?}", other)
            );
        }
        None => {
            return type_error(&logger, module, stmt, "Group name is required".to_string());
        }
    };

    let resolved_body = match value_map.get("body") {
        Some(Value::Block(statements)) => {
            let mut resolved = Vec::new();

            for stmt in statements {
                match &stmt.kind {
                    StatementKind::Trigger { entity, duration } => {
                        let resolved_trigger = resolve_trigger(
                            &mut stmt.clone(),
                            entity,
                            &mut duration.clone(),
                            module,
                            path,
                            global_store
                        );

                        resolved.push(resolved_trigger);
                    }

                    _ => {
                        println!("Unhandled group body statement: {:?}", stmt);
                    }
                }
            }

            Value::Block(resolved)
        }

        Some(other) => {
            return type_error(
                &logger,
                module,
                stmt,
                format!("Unexpected value for group body: {:?}", other)
            );
        }

        None => {
            return type_error(
                &logger,
                module,
                stmt,
                "Missing 'body' key in group statement".to_string()
            );
        }
    };

    let mut resolved_map = HashMap::new();
    resolved_map.insert("identifier".to_string(), Value::String(group_name));
    resolved_map.insert("body".to_string(), resolved_body);

    Statement {
        kind: StatementKind::Group,
        value: Value::Map(resolved_map),
        ..stmt.clone()
    }
}

fn type_error(logger: &Logger, module: &Module, stmt: &Statement, message: String) -> Statement {
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_error_with_stacktrace(&message, &stacktrace);

    Statement {
        kind: StatementKind::Error { message },
        value: Value::Null,
        ..stmt.clone()
    }
}
