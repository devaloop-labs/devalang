use crate::{
    core::{
        parser::statement::{Statement, StatementKind},
        preprocessor::{module::Module, resolver::driver::resolve_statement},
        shared::value::Value,
        store::global::GlobalStore,
    },
    utils::logger::{LogLevel, Logger},
};

pub fn resolve_synth(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore,
) -> Statement {
    let logger = Logger::new();

    let Value::Map(synth_map) = &stmt.value else {
        return type_error(
            &logger,
            module,
            stmt,
            "Expected a map in synth statement".to_string(),
        );
    };

    let mut resolved_map = synth_map.clone();

    if let Some(Value::Block(body)) = synth_map.get("body") {
        let resolved_body = body
            .iter()
            .map(|s| resolve_statement(s, module, path, global_store))
            .collect::<Vec<_>>();
        resolved_map.insert("body".to_string(), Value::Block(resolved_body));
    } else {
        logger.log_message(LogLevel::Warning, "synth without a body");
    }

    Statement {
        kind: StatementKind::Synth,
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
