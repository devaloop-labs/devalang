pub mod core;
pub mod utils;
pub mod config;

use serde::{ Deserialize, Serialize };
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;

use crate::core::{
    parser::statement::{ Statement, StatementKind },
    preprocessor::loader::ModuleLoader,
    shared::value::Value,
    store::global::GlobalStore,
    utils::path::normalize_path,
};

#[derive(Serialize, Deserialize)]
struct ParseResult {
    ok: bool,
    ast: String,
    errors: Vec<ErrorResult>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResult {
    message: String,
    line: usize,
    column: usize,
}

#[wasm_bindgen]
pub fn parse(entry_path: &str, source: &str) -> Result<JsValue, JsValue> {
    let statements = parse_internal_from_string(entry_path, source);

    match statements {
        Ok(value) => {
            let ast_string = value;
            to_value(&ast_string).map_err(|e|
                JsValue::from_str(&format!("Error converting AST to JS value: {}", e))
            )
        }
        Err(e) => { Err(JsValue::from_str(&format!("Error: {}", e))) }
    }
}

fn parse_internal_from_string(virtual_path: &str, source: &str) -> Result<ParseResult, String> {
    let entry_path = normalize_path(virtual_path);
    let output_path = normalize_path("./temp");

    let mut global_store = GlobalStore::new();
    let loader = ModuleLoader::from_raw_source(
        &entry_path,
        &output_path,
        source,
        &mut global_store
    );

    let module = loader
        .load_single_module(&mut global_store)
        .map_err(|e| format!("Error loading module: {}", e))?;

    let raw_ast = ast_to_string(module.statements.clone());

    let found_errors = collect_errors_recursively(&module.statements);

    let result = ParseResult {
        ok: true,
        ast: raw_ast,
        errors: found_errors,
    };

    Ok(result)
}

fn collect_errors_recursively(statements: &[Statement]) -> Vec<ErrorResult> {
    let mut errors: Vec<ErrorResult> = Vec::new();

    for stmt in statements {
        match &stmt.kind {
            StatementKind::Unknown => {
                errors.push(ErrorResult {
                    message: format!("Unknown statement at line {}:{}", stmt.line, stmt.column),
                    line: stmt.line,
                    column: stmt.column,
                });
            }
            StatementKind::Error { message } => {
                errors.push(ErrorResult {
                    message: message.clone(),
                    line: stmt.line,
                    column: stmt.column,
                });
            }
            StatementKind::Loop => {
                if let Some(body_statements) = extract_loop_body_statements(&stmt.value) {
                    errors.extend(collect_errors_recursively(body_statements));
                }
            }
            _ => {}
        }
    }

    errors
}

fn extract_loop_body_statements(value: &Value) -> Option<&[Statement]> {
    if let Value::Map(map) = value {
        if let Some(Value::Block(statements)) = map.get("body") {
            return Some(statements);
        }
    }
    None
}

fn ast_to_string(statements: Vec<Statement>) -> String {
    serde_json::to_string_pretty(&statements).expect("Failed to serialize AST")
}
