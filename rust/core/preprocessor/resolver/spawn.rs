use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{
            module::Module,
            resolver::driver::resolve_statement,
            resolver::value::resolve_value,
        },
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::{ Logger, LogLevel },
};

pub fn resolve_spawn(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    let resolved_value = resolve_value(&stmt.value, module, global_store);

    match resolved_value {
        Value::Map(mut map) => {
            if let Some(Value::Block(stmts)) = map.get("body") {
                let resolved_block = stmts
                    .iter()
                    .map(|s| resolve_statement(s, module, path, global_store))
                    .collect();
                map.insert("body".to_string(), Value::Block(resolved_block));
            }

            Statement {
                kind: StatementKind::Spawn,
                value: Value::Map(map),
                ..stmt.clone()
            }
        }

        _ => {
            let stacktrace = format!("{}:{}:{}", module.path, stmt.line, stmt.column);
            logger.log_message(
                LogLevel::Error,
                &format!("Expected a map in spawn statement\n  → at {stacktrace}")
            );

            Statement {
                kind: StatementKind::Error {
                    message: "Invalid spawn value".to_string(),
                },
                value: Value::Null,
                ..stmt.clone()
            }
        }
    }
}