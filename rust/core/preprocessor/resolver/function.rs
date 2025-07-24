use std::collections::HashMap;

use crate::{
    core::{
        parser::statement::{ Statement, StatementKind },
        preprocessor::{ module::Module, resolver::driver::resolve_statement },
        shared::value::Value,
        store::{ function::FunctionDef, global::GlobalStore, variable::VariableTable },
    },
    utils::logger::{ LogLevel, Logger },
};

pub fn resolve_function(
    stmt: &Statement,
    module: &Module,
    path: &str,
    global_store: &mut GlobalStore
) -> Statement {
    if let StatementKind::Function { name, parameters, body } = &stmt.kind {
        global_store.functions.functions.insert(name.clone(), FunctionDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
        });

        let resolved_body = resolve_block_statements(body, &module, path, global_store);

        return Statement {
            kind: StatementKind::Function {
                name: name.clone(),
                parameters: parameters.clone(),
                body: resolved_body,
            },
            value: Value::Null,
            ..stmt.clone()
        };
    } else {
        return Statement {
            kind: StatementKind::Error {
                message: "Expected a function statement".to_string(),
            },
            value: Value::Null,
            ..stmt.clone()
        };
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
