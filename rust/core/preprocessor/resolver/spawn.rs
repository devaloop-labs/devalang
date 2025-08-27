use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::module::Module,
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::{ Logger, LogLevel },
};

pub fn resolve_spawn(
    stmt: &Statement,
    name: String,
    args: Vec<Value>,
    module: &Module,
    _path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    // ✅ Si c'est une fonction
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

    // ✅ Si c'est un group dans les variables
    if let Some(variable) = global_store.variables.variables.get(&name) {
        if let Value::Statement(stmt_box) = variable {
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
        }
    }

    // ❌ Sinon erreur
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_message(
        LogLevel::Error,
        &format!("Function or group '{}' not found for spawn\n  → at {stacktrace}", name)
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
