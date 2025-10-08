//! Parsing API for WASM
//!
//! Exposes Devalang parser to JavaScript.

use serde::Serialize;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

use crate::language::syntax::parser::driver::SimpleParser;

/// Parse result returned to JavaScript
#[derive(Serialize)]
pub struct ParseResult {
    pub success: bool,
    pub statements: Vec<StatementInfo>,
    pub errors: Vec<String>,
}

/// Simplified statement info for JS
#[derive(Serialize)]
pub struct StatementInfo {
    pub kind: String,
    pub line: usize,
}

/// Parse Devalang source code
///
/// # Arguments
/// * `entry_path` - Path to the source file (for error messages)
/// * `source` - Source code to parse
///
/// # Returns
/// JSON object with parse results
#[wasm_bindgen]
pub fn parse(entry_path: &str, source: &str) -> Result<JsValue, JsValue> {
    // TODO: Implement full parsing with error collection
    // For now, basic implementation

    match SimpleParser::parse(source, PathBuf::from(entry_path)) {
        Ok(statements) => {
            let statement_infos: Vec<StatementInfo> = statements
                .iter()
                .map(|stmt| StatementInfo {
                    kind: format!("{:?}", stmt.kind),
                    line: stmt.line_number,
                })
                .collect();

            let result = ParseResult {
                success: true,
                statements: statement_infos,
                errors: vec![],
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
        Err(e) => {
            let result = ParseResult {
                success: false,
                statements: vec![],
                errors: vec![format!("{}", e)],
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
        }
    }
}

/// Quick parse check - returns true if code parses without errors
#[wasm_bindgen]
pub fn check_syntax(source: &str) -> bool {
    SimpleParser::parse(source, PathBuf::from("check.deva")).is_ok()
}
