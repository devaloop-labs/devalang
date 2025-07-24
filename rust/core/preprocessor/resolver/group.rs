use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{ module::Module, resolver::driver::resolve_statement },
        shared::value::Value,
        store::{ global::GlobalStore, variable::VariableTable },
    },
    utils::logger::{ LogLevel, Logger },
};

pub fn resolve_group(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    let logger = Logger::new();

    let Value::Map(group_map) = &stmt.value else {
        return type_error(&logger, module, stmt, "Expected a map in group statement".to_string());
    };

    let mut resolved_map = group_map.clone();

    if let Some(Value::Block(body)) = group_map.get("body") {
        let resolved_body = resolve_block_statements(body, module, path, global_store);
        resolved_map.insert("body".to_string(), Value::Block(resolved_body));
    } else {
        logger.log_message(LogLevel::Warning, "group without a body");
    }

    Statement {
        kind: StatementKind::Group,
        value: Value::Map(resolved_map),
        ..stmt.clone()
    }
}

fn resolve_block_statements(
    body: &[Statement],
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Vec<Statement> {
    body.iter()
        .map(|stmt| resolve_statement(stmt, module, path, global_store))
        .collect()
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
