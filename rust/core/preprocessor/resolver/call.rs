use crate::{
    core::{
        parser::statement::{Statement, StatementKind},
        preprocessor::{module::Module, resolver::driver::resolve_statement},
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::{Logger, LogLevel},
};

pub fn resolve_call(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    match &stmt.kind {
        StatementKind::Call { name, args } => {
            if let Some(func) = global_store.functions.functions.get(name) {
                let mut call_map = std::collections::HashMap::new();
                call_map.insert("name".to_string(), Value::Identifier(name.clone()));
                call_map.insert(
                    "parameters".to_string(),
                    Value::Array(
                        func.parameters
                            .iter()
                            .map(|p| Value::Identifier(p.clone()))
                            .collect(),
                    ),
                );
                call_map.insert(
                    "args".to_string(),
                    Value::Array(args.clone())
                );
                call_map.insert("body".to_string(), Value::Block(func.body.clone()));

                Statement {
                    kind: StatementKind::Call {
                        name: name.clone(),
                        args: args.clone(),
                    },
                    value: Value::Map(call_map),
                    ..stmt.clone()
                }
            } else {
                error_stmt(
                    &logger,
                    module,
                    stmt,
                    &format!("Function '{}' not found in GlobalStore", name),
                )
            }
        }

        _ => error_stmt(
            &logger,
            module,
            stmt,
            "Expected StatementKind::Call in resolve_call()",
        ),
    }
}

fn error_stmt(logger: &Logger, module: &Module, stmt: &Statement, message: &str) -> Statement {
    let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
    logger.log_message(LogLevel::Error, &format!("{message}\n  → at {stacktrace}"));

    Statement {
        kind: StatementKind::Error {
            message: message.to_string(),
        },
        value: Value::Null,
        ..stmt.clone()
    }
}
