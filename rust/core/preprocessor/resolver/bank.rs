use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::module::Module,
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::Logger,
};

pub fn resolve_bank(
    stmt: &Statement,
    module: &Module,
    path: &str,
    _global_store: &GlobalStore
) -> Statement {
    let mut new_stmt = stmt.clone();
    let logger = Logger::new();

    match &stmt.value {
        Value::Identifier(ident) => {
            if let Some(val) = module.variable_table.get(ident) {
                new_stmt.value = val.clone();
            } else {
                let message = format!("Bank identifier '{ident}' not found in variable table");
                logger.log_error_with_stacktrace(&message, &module.path);
                new_stmt.kind = StatementKind::Error {
                    message: message.clone(),
                };
                new_stmt.value = Value::Null;
            }
        }

        Value::String(_) => {}

        Value::Number(_) => {}

        other => {
            let message = format!("Expected a string or identifier for bank, found {:?}", other);
            logger.log_error_with_stacktrace(&message, &module.path);
            new_stmt.kind = StatementKind::Error {
                message: "Expected a string or identifier for bank".to_string(),
            };
            new_stmt.value = Value::Null;
        }
    }

    new_stmt
}
