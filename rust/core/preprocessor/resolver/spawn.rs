use crate::core::{
    parser::statement::{Statement, StatementKind},
    preprocessor::module::Module,
    store::global::GlobalStore,
};
use devalang_types::Value;
use devalang_utils::logger::{LogLevel, Logger};

pub fn resolve_spawn(
    stmt: &Statement,
    name: String,
    args: Vec<Value>,
    module: &Module,
    _path: &str,
    global_store: &mut GlobalStore,
) -> Statement {
    let logger = Logger::new();

    // If it's a function
    if let Some(func) = global_store.functions.functions.get(&name) {
        let mut resolved_map = std::collections::HashMap::new();
        resolved_map.insert("name".to_string(), Value::String(name.clone()));
        resolved_map.insert("args".to_string(), Value::Array(args.clone()));
        resolved_map.insert("body".to_string(), Value::Block(func.body.clone()));

        return Statement {
            kind: StatementKind::Spawn { name, args },
            value: Value::Map(resolved_map),
            ..stmt.clone()
        };
    }

    // If it's a group stored in variables
    if let Some(Value::Statement(stmt_box)) = global_store.variables.variables.get(&name) {
        if let StatementKind::Group = stmt_box.kind {
            if let Value::Map(map) = &stmt_box.value {
                if let Some(Value::Block(body)) = map.get("body") {
                    let mut resolved_map = std::collections::HashMap::new();
                    resolved_map.insert("identifier".to_string(), Value::String(name.clone()));
                    resolved_map.insert("args".to_string(), Value::Array(args.clone()));
                    resolved_map.insert("body".to_string(), Value::Block(body.clone()));

                    return Statement {
                        kind: StatementKind::Spawn { name, args },
                        value: Value::Map(resolved_map),
                        ..stmt.clone()
                    };
                }
            }
        }
        // Pattern case (make spawn accept patterns stored as variables)
        if let StatementKind::Pattern { .. } = stmt_box.kind {
            let mut resolved_map = std::collections::HashMap::new();
            resolved_map.insert("identifier".to_string(), Value::String(name.clone()));
            // pattern value may be a string or a map stored on the statement
            match &stmt_box.value {
                Value::String(s) => {
                    resolved_map.insert("pattern".to_string(), Value::String(s.clone()));
                }
                Value::Map(m) => {
                    if let Some(val) = m.get("pattern") {
                        resolved_map.insert("pattern".to_string(), val.clone());
                    }
                    if let Some(val) = m.get("target") {
                        resolved_map.insert("target".to_string(), val.clone());
                    }
                }
                _ => {}
            }
            resolved_map.insert("args".to_string(), Value::Array(args.clone()));

            return Statement {
                kind: StatementKind::Spawn { name, args },
                value: Value::Map(resolved_map),
                ..stmt.clone()
            };
        }
    }

    // Otherwise, log an error
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_message(
        LogLevel::Error,
        &format!(
            "Function or group '{}' not found for spawn\n  → at {stacktrace}",
            name
        ),
    );

    Statement {
        kind: StatementKind::Error {
            message: format!("Function or group '{}' not found for spawn", name),
        },
        value: Value::Null,
        ..stmt.clone()
    }
}

// (removed unused helpers get_group_body, error_stmt)
