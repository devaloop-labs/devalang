use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{Statement, StatementKind},
        preprocessor::{module::Module, resolver::driver::resolve_statement},
        shared::value::Value, store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_loop(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &GlobalStore,
) -> Statement {
    let logger = Logger::new();

    let Value::Map(value_map) = &stmt.value else {
        return error_stmt(&logger, module, stmt, "Expected a map for loop value");
    };

    // Iterator resolution
    let iterator_value = match value_map.get("iterator") {
        Some(Value::Identifier(ident)) => {
            resolve_identifier_number(ident, module, global_store).unwrap_or_else(|| {
                error_value(&logger, module, stmt, &format!("Loop iterator '{ident}' could not be resolved to a number"));
                Value::Null
            })
        }

        Some(Value::Number(n)) => Value::Number(*n),

        Some(other) => {
            error_value(&logger, module, stmt, &format!("Unexpected value for loop iterator: {:?}", other));
            Value::Null
        }

        None => {
            error_value(&logger, module, stmt, "Missing 'iterator' key in loop statement map");
            Value::Null
        }
    };

    // Body resolution
    let body_value = match value_map.get("body") {
        Some(Value::Block(body)) => {
            let resolved = body
                .iter()
                .flat_map(|stmt| {
                    let resolved = resolve_statement(stmt, module, path, global_store);
                    if let StatementKind::Call = resolved.kind {
                        if let Value::Block(nested) = &resolved.value {
                            return nested
                                .iter()
                                .map(|s| resolve_statement(s, module, path, global_store))
                                .collect::<Vec<_>>();
                        }
                    }
                    vec![resolved]
                })
                .collect();

            Value::Block(resolved)
        }

        Some(other) => {
            error_value(&logger, module, stmt, &format!("Unexpected value for loop body: {:?}", other));
            Value::Null
        }

        None => {
            error_value(&logger, module, stmt, "Missing 'body' key in loop statement map");
            Value::Null
        }
    };

    let mut resolved_map = HashMap::new();
    resolved_map.insert("iterator".to_string(), iterator_value);
    resolved_map.insert("body".to_string(), body_value);

    Statement {
        kind: StatementKind::Loop,
        value: Value::Map(resolved_map),
        ..stmt.clone()
    }
}

fn resolve_identifier_number(
    ident: &str,
    module: &Module,
    global_store: &GlobalStore,
) -> Option<Value> {
    if let Some(Value::Number(n)) = module.variable_table.get(ident) {
        return Some(Value::Number(*n));
    }

    for other_mod in global_store.modules.values() {
        if let Some(Value::Number(n)) = other_mod.export_table.get_export(ident) {
            return Some(Value::Number(*n));
        }
    }

    ident.parse::<f32>().ok().map(Value::Number)
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
