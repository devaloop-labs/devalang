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
        let resolved_body = resolve_block_statements(body, &module, path, global_store);

        global_store.functions.add_function(FunctionDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: resolved_body.clone(),
        });

        if let Some(current_mod) = global_store.modules.get_mut(path) {
            current_mod.function_table.add_function(FunctionDef {
                name: name.clone(),
                parameters: parameters.clone(),
                body: resolved_body.clone(),
            });
        } else {
            eprintln!("[resolve_statement] ❌ Module path not found: {path}");
        }

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
