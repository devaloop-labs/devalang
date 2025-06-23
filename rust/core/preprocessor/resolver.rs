use std::{ collections::HashMap, hash::Hash };

use toml::value::Array;

use crate::core::types::{
    module::Module,
    parser::Parser,
    statement::{ Statement, StatementKind },
    store::{ ExportTable, GlobalStore, ImportTable },
    token::{ Token, TokenDuration, TokenKind, TokenParam, TokenParamValue },
    variable::VariableValue,
};

pub fn resolve_exports(statements: &[Statement], parser: &Parser) -> ExportTable {
    let mut export_table = parser.export_table.clone();

    for stmt in statements {
        if let StatementKind::Export = &stmt.kind {
            if let VariableValue::Array(tokens) = &stmt.value {
                for token in tokens {
                    let var_name = &token.lexeme;
                    if let Some(value) = parser.variable_table.variables.get(var_name) {
                        export_table.add_export(var_name.clone(), value.clone());
                    } else {
                        eprintln!("⚠️ Variable '{}' not found in scope, export skipped", var_name);
                    }
                }
            } else {
                eprintln!("⚠️ Unexpected value type in export: {:?}", stmt.value);
            }
        }
    }

    export_table
}

pub fn resolve_imports(module: &mut Module, global_store: &GlobalStore) -> ImportTable {
    let mut import_table = ImportTable::default();

    for stmt in &module.statements {
        if let StatementKind::Import { names, source } = &stmt.kind {
            if let Some(from_module) = global_store.modules.get(source) {
                for name in names {
                    if let Some(value) = from_module.export_table.exports.get(name) {
                        module.variable_table.variables.insert(name.clone(), value.clone());
                        import_table.add_import(name.clone(), value.clone());
                    } else {
                        eprintln!("⚠️ '{}' not found in exports of '{}'", name, source);
                    }
                }
            } else {
                eprintln!("⚠️ Module '{}' not found", source);
            }
        }
    }

    import_table
}

pub fn resolve_statement(stmt: &Statement, module: &Module) -> Statement {
    match &stmt.kind {
        StatementKind::Trigger { entity, duration } => {
            // Parsing the duration
            let duration_raw_value = match duration {
                TokenDuration::Auto => "auto",
                TokenDuration::Infinite => "infinite",
                TokenDuration::Number(n) => &n.to_string(),
                TokenDuration::Identifier(id) => id.as_str(),
                _ => "unknown",
            };

            let parsed_duration_value = if
                let Some(duration_value) = module.variable_table.variables.get(duration_raw_value)
            {
                match duration_value {
                    VariableValue::Text(text) => TokenDuration::Identifier(text.clone()),
                    VariableValue::Number(num) => TokenDuration::Number(*num),
                    _ => {
                        eprintln!("⚠️ Invalid duration type for Trigger: {:?}", duration_value);
                        TokenDuration::Unknown
                    }
                }
            } else {
                eprintln!("⚠️ Duration variable '{}' not found in module", duration_raw_value);
                TokenDuration::Unknown
            };

            // Parsing the entity value (params)
            match &stmt.value {
                VariableValue::Text(text) => {
                    // Check if the text is a valid variable in the module
                    if let Some(value) = module.variable_table.variables.get(text) {
                        Statement {
                            kind: StatementKind::Trigger {
                                entity: entity.clone(),
                                duration: duration.clone(),
                            },
                            value: value.clone(),
                            indent: stmt.indent,
                            line: stmt.line,
                            column: stmt.column,
                        }
                    } else {
                        eprintln!("⚠️ Trigger variable '{}' not found", text);
                        stmt.clone()
                    }
                }
                VariableValue::Map(map) => {
                    // Parse other parameters
                    let mut parsed_params_map: HashMap<String, TokenParamValue> = HashMap::new();

                    let params_value = &map
                        .get("params")
                        .and_then(|v| {
                            match v {
                                TokenParamValue::String(text) => Some(text.clone()),
                                TokenParamValue::Number(num) => Some(num.to_string()),
                                TokenParamValue::Boolean(bool) => Some(bool.to_string()),
                                TokenParamValue::Array(array) => {
                                    let mut params = String::new();
                                    for item in array {
                                        if let TokenParamValue::String(text) = item {
                                            params.push_str(&text);
                                            params.push(' ');
                                        } else {
                                            eprintln!(
                                                "⚠️ Invalid type in params array: {:?}",
                                                item
                                            );
                                        }
                                    }
                                    Some(params.trim().to_string())
                                }
                                _ => None,
                            }
                        })
                        .unwrap_or("params".to_string());

                    if let Some(params) = module.variable_table.variables.get(params_value) {
                        match &params {
                            VariableValue::Text(text) => {
                                parsed_params_map.insert(
                                    "params".to_string(),
                                    TokenParamValue::String(text.clone())
                                );
                            }
                            VariableValue::Number(num) => {
                                parsed_params_map.insert(
                                    "params".to_string(),
                                    TokenParamValue::Number(*num)
                                );
                            }
                            VariableValue::Boolean(bool) => {
                                parsed_params_map.insert(
                                    "params".to_string(),
                                    TokenParamValue::Boolean(*bool)
                                );
                            }
                            VariableValue::Map(map) => {
                                for (key, value) in map {
                                    parsed_params_map.insert(key.clone(), value.clone());
                                }
                            }
                            _ => {
                                eprintln!("⚠️ Invalid params type for Trigger: {:?}", params);
                            }
                        }
                    } else {
                        eprintln!("⚠️ Params variable not found in module");
                    }

                    Statement {
                        kind: StatementKind::Trigger {
                            entity: entity.clone(),
                            duration: parsed_duration_value.clone(),
                        },
                        value: VariableValue::Map(parsed_params_map),
                        indent: stmt.indent,
                        line: stmt.line,
                        column: stmt.column,
                    }
                }
                _ => {
                    eprintln!("⚠️ Invalid value type for Trigger statement: {:?}", stmt.value);
                    stmt.clone()
                }
            }
        }

        // SECTION Bank declaration
        StatementKind::Bank { .. } => {
            match &stmt.value {
                VariableValue::Text(name) => {
                    if
                        let Some(value) = module.import_table.imports
                            .get(name)
                            .or_else(|| module.variable_table.variables.get(name))
                    {
                        Statement {
                            kind: StatementKind::Bank,
                            value: value.clone(),
                            indent: stmt.indent,
                            line: stmt.line,
                            column: stmt.column,
                        }
                    } else {
                        eprintln!("⚠️ Bank variable '{}' not found", name);
                        stmt.clone()
                    }
                }
                _ => stmt.clone(),
            }
        }

        // TODO Handle other statement kinds

        _ => stmt.clone(),
    }
}
