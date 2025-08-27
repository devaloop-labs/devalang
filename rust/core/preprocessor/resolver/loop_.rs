use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{
            module::Module,
            resolver::{ driver::resolve_statement, value::resolve_value },
        },
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_loop(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    let resolved_value = resolve_value(&stmt.value, module, global_store);

    let Value::Map(value_map) = &resolved_value else {
        return error_stmt(&logger, module, stmt, "Expected a map for loop value");
    };

    let mut resolved_map: HashMap<String, Value> = HashMap::new();
    for (key, val) in value_map {
        resolved_map.insert(key.clone(), resolve_value(val, module, global_store));
    }

    // Foreach form takes precedence if present
    if let (Some(Value::Identifier(var_name)), Some(array_val)) = (resolved_map.get("foreach"), resolved_map.get("array")) {
        // Resolve array elements
        let resolved_array = match array_val {
            Value::Array(items) => Value::Array(items.iter().map(|v| resolve_value(v, module, global_store)).collect()),
            other => resolve_value(other, module, global_store),
        };

        let body_value = match resolved_map.get("body") {
            Some(Value::Block(stmts)) => {
                let resolved = stmts
                    .iter()
                    .map(|s| resolve_statement(s, module, path, global_store))
                    .collect();
                Value::Block(resolved)
            }
            _ => {
                error_value(&logger, module, stmt, "Invalid or missing loop body");
                Value::Block(vec![])
            }
        };

        let mut final_map = HashMap::new();
        final_map.insert("foreach".to_string(), Value::Identifier(var_name.clone()));
        final_map.insert("array".to_string(), resolved_array);
        final_map.insert("body".to_string(), body_value);

        return Statement { kind: StatementKind::Loop, value: Value::Map(final_map), ..stmt.clone() };
    }

    let iterator_value = match resolved_map.get("iterator") {
        Some(Value::Number(n)) => Value::Number(*n),
        Some(other) => {
            error_value(
                &logger,
                module,
                stmt,
                &format!("Loop iterator must be a number, found: {:?}", other)
            );
            Value::Number(1.0)
        }
        None => {
            error_value(&logger, module, stmt, "Missing 'iterator' in loop");
            Value::Number(1.0)
        }
    };

    let body_value = match resolved_map.get("body") {
        Some(Value::Block(stmts)) => {
            let resolved = stmts
                .iter()
                .map(|s| resolve_statement(s, module, path, global_store))
                .collect();
            Value::Block(resolved)
        }
        _ => {
            error_value(&logger, module, stmt, "Invalid or missing loop body");
            Value::Block(vec![])
        }
    };

    let mut final_map = HashMap::new();
    final_map.insert("iterator".to_string(), iterator_value);
    final_map.insert("body".to_string(), body_value);

    Statement {
        kind: StatementKind::Loop,
        value: Value::Map(final_map),
        ..stmt.clone()
    }
}

fn error_value(logger: &Logger, module: &Module, stmt: &Statement, msg: &str) {
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_error_with_stacktrace(msg, &stacktrace);
}

fn error_stmt(logger: &Logger, module: &Module, stmt: &Statement, msg: &str) -> Statement {
    error_value(logger, module, stmt, msg);
    Statement {
        kind: StatementKind::Error {
            message: msg.to_string(),
        },
        value: Value::Null,
        ..stmt.clone()
    }
}
