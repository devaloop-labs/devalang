use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{Statement, StatementKind},
        preprocessor::{
            module::Module,
            resolver::{driver::resolve_statement, value::resolve_value},
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
    global_store: &mut GlobalStore,
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
    if let (Some(Value::Identifier(var_name)), Some(array_val)) =
        (resolved_map.get("foreach"), resolved_map.get("array"))
    {
        // Normalize array_val into an iterable Array
        let resolved_array = match array_val {
            Value::Array(items) => Value::Array(
                items
                    .iter()
                    .map(|v| resolve_value(v, module, global_store))
                    .collect(),
            ),
            Value::Number(n) => {
                // Iterate 0..n-1
                let count = (*n).max(0.0) as usize;
                let mut items = Vec::with_capacity(count);
                for i in 0..count {
                    items.push(Value::Number(i as f32));
                }
                Value::Array(items)
            }
            Value::String(s) => {
                // Try to parse a simple comma-separated list: "a,b,c" -> ["a","b","c"]
                // If numeric string: iterate 0..n-1
                if let Ok(n) = s.parse::<f32>() {
                    let count = n.max(0.0) as usize;
                    let mut items = Vec::with_capacity(count);
                    for i in 0..count {
                        items.push(Value::Number(i as f32));
                    }
                    Value::Array(items)
                } else if s.contains(',') {
                    let parts: Vec<Value> = s
                        .split(',')
                        .map(|p| Value::String(p.trim().to_string()))
                        .collect();
                    Value::Array(parts)
                } else {
                    // Fallback: iterate characters
                    let parts: Vec<Value> =
                        s.chars().map(|c| Value::String(c.to_string())).collect();
                    Value::Array(parts)
                }
            }
            Value::Identifier(name) => {
                // Resolve identifier from module variables (already resolved map above)
                let v = if let Some(v) = module.variable_table.get(name) {
                    v.clone()
                } else {
                    Value::Null
                };
                match v {
                    Value::Array(items) => Value::Array(
                        items
                            .iter()
                            .map(|v| resolve_value(v, module, global_store))
                            .collect(),
                    ),
                    Value::Number(n) => {
                        let count = n.max(0.0) as usize;
                        let mut items = Vec::with_capacity(count);
                        for i in 0..count {
                            items.push(Value::Number(i as f32));
                        }
                        Value::Array(items)
                    }
                    Value::String(s) => {
                        if let Ok(n) = s.parse::<f32>() {
                            let count = n.max(0.0) as usize;
                            let mut items = Vec::with_capacity(count);
                            for i in 0..count {
                                items.push(Value::Number(i as f32));
                            }
                            Value::Array(items)
                        } else if s.contains(',') {
                            let parts: Vec<Value> = s
                                .split(',')
                                .map(|p| Value::String(p.trim().to_string()))
                                .collect();
                            Value::Array(parts)
                        } else {
                            let parts: Vec<Value> =
                                s.chars().map(|c| Value::String(c.to_string())).collect();
                            Value::Array(parts)
                        }
                    }
                    other => {
                        error_value(
                            &logger,
                            module,
                            stmt,
                            &format!(
                                "Foreach identifier '{}' resolves to unsupported value: {:?}",
                                name, other
                            ),
                        );
                        Value::Array(vec![])
                    }
                }
            }
            other => {
                // Resolve and normalize if possible
                let v = resolve_value(other, module, global_store);
                match v {
                    Value::Array(items) => Value::Array(items),
                    Value::Number(n) => {
                        let count = n.max(0.0) as usize;
                        let mut items = Vec::with_capacity(count);
                        for i in 0..count {
                            items.push(Value::Number(i as f32));
                        }
                        Value::Array(items)
                    }
                    Value::String(s) => {
                        if let Ok(n) = s.parse::<f32>() {
                            let count = n.max(0.0) as usize;
                            let mut items = Vec::with_capacity(count);
                            for i in 0..count {
                                items.push(Value::Number(i as f32));
                            }
                            Value::Array(items)
                        } else if s.contains(',') {
                            let parts: Vec<Value> = s
                                .split(',')
                                .map(|p| Value::String(p.trim().to_string()))
                                .collect();
                            Value::Array(parts)
                        } else {
                            let parts: Vec<Value> =
                                s.chars().map(|c| Value::String(c.to_string())).collect();
                            Value::Array(parts)
                        }
                    }
                    other => {
                        error_value(
                            &logger,
                            module,
                            stmt,
                            &format!("Unsupported foreach array value: {:?}", other),
                        );
                        Value::Array(vec![])
                    }
                }
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
        final_map.insert("foreach".to_string(), Value::Identifier(var_name.clone()));
        final_map.insert("array".to_string(), resolved_array);
        final_map.insert("body".to_string(), body_value);

        return Statement {
            kind: StatementKind::Loop,
            value: Value::Map(final_map),
            ..stmt.clone()
        };
    }

    let iterator_value = match resolved_map.get("iterator") {
        Some(Value::Number(n)) => Value::Number(*n),
        Some(Value::String(s)) => {
            if let Ok(n) = s.parse::<f32>() {
                Value::Number(n)
            } else {
                error_value(
                    &logger,
                    module,
                    stmt,
                    &format!("Loop iterator string not numeric: '{}'", s),
                );
                Value::Number(1.0)
            }
        }
        Some(Value::Identifier(name)) => {
            // Try resolving from module vars (may be number or numeric string)
            if let Some(v) = module.variable_table.get(name) {
                match v {
                    Value::Number(n) => Value::Number(*n),
                    Value::String(s) => {
                        if let Ok(n) = s.parse::<f32>() {
                            Value::Number(n)
                        } else {
                            error_value(
                                &logger,
                                module,
                                stmt,
                                &format!(
                                    "Loop iterator '{}' resolves to non-numeric string: '{}'",
                                    name, s
                                ),
                            );
                            Value::Number(1.0)
                        }
                    }
                    other => {
                        error_value(
                            &logger,
                            module,
                            stmt,
                            &format!(
                                "Loop iterator '{}' resolves to non-number: {:?}",
                                name, other
                            ),
                        );
                        Value::Number(1.0)
                    }
                }
            } else {
                error_value(
                    &logger,
                    module,
                    stmt,
                    &format!("Loop iterator identifier '{}' not found", name),
                );
                Value::Number(1.0)
            }
        }
        Some(other) => {
            error_value(
                &logger,
                module,
                stmt,
                &format!("Loop iterator must be a number, found: {:?}", other),
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
