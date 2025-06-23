use std::{ collections::HashMap, hash::Hash };

use toml::value::Array;

use crate::core::types::{
    module::Module,
    parser::Parser,
    statement::{ Statement, StatementIterator, StatementKind },
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
        StatementKind::Loop { iterator } => {
            let mut resolved_iterator = StatementIterator::Unknown;

            match iterator.clone() {
                StatementIterator::Identifier(id) => {
                    // Check if the identifier is a variable in the module
                    if let Some(value) = module.variable_table.variables.get(&id) {
                        match value {
                            VariableValue::Array(arr) => {
                                resolved_iterator = StatementIterator::Array(arr.clone());
                            }
                            VariableValue::Number(num) => {
                                resolved_iterator = StatementIterator::Number(*num);
                            }
                            _ => {
                                eprintln!(
                                    "⚠️ Unsupported variable type for loop iterator: {:?}",
                                    value
                                );
                                resolved_iterator = StatementIterator::Unknown;
                            }
                        }
                    } else {
                        eprintln!("⚠️ Loop iterator variable '{}' not found", id);
                        resolved_iterator = StatementIterator::Unknown;
                    }
                }
                StatementIterator::Number(num) => {
                    resolved_iterator = StatementIterator::Number(num);
                }
                _ => {
                    resolved_iterator = iterator.clone();
                }
            }

            return Statement {
                kind: StatementKind::Loop { iterator: resolved_iterator },
                value: stmt.value.clone(),
                indent: stmt.indent,
                line: stmt.line,
                column: stmt.column,
            };
        }

        StatementKind::Trigger { entity, duration } => {
            // Parsing the duration
            let duration_raw_value = match duration {
                TokenDuration::Auto => "auto",
                TokenDuration::Infinite => "infinite",
                TokenDuration::Number(n) => &n.to_string(),
                TokenDuration::Identifier(id) => id.as_str(),
                _ => "unknown",
            };

            let duration_variable_value = module.variable_table.variables.get(duration_raw_value);
            let mut parsed_duration_value = TokenDuration::Unknown;

            if duration_variable_value.is_some() {
                parsed_duration_value = duration_variable_value
                    .as_ref()
                    .map_or(TokenDuration::Unknown, |value| {
                        match value {
                            VariableValue::Text(text) => TokenDuration::Identifier(text.clone()),
                            VariableValue::Number(num) => TokenDuration::Number(*num),
                            VariableValue::Boolean(_) => TokenDuration::Unknown,
                            _ => {
                                eprintln!("⚠️ Invalid duration type for Trigger: {:?}", value);
                                TokenDuration::Unknown
                            }
                        }
                    });
            } else if let Ok(num) = duration_raw_value.parse::<f32>() {
                parsed_duration_value = TokenDuration::Number(num);
            } else if duration_raw_value == "auto" {
                parsed_duration_value = TokenDuration::Auto;
            } else if duration_raw_value == "infinite" {
                parsed_duration_value = TokenDuration::Infinite;
            } else {
                eprintln!("⚠️ Invalid duration format: {}", duration_raw_value);
            }

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
                VariableValue::Map(map) => { stmt.clone() }
                VariableValue::Null => { stmt.clone() }
                // TODO Parse other parameters
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

        StatementKind::Tempo => {
            match &stmt.value {
                VariableValue::Number(num) => {
                    if *num > 0.0 {
                        Statement {
                            kind: StatementKind::Tempo,
                            value: VariableValue::Number(*num),
                            indent: stmt.indent,
                            line: stmt.line,
                            column: stmt.column,
                        }
                    } else {
                        eprintln!("⚠️ Invalid tempo value: {}", num);
                        stmt.clone()
                    }
                }
                VariableValue::Text(text) => {
                    // Check if the text is a valid variable in the module
                    if let Some(value) = module.variable_table.variables.get(text) {
                        Statement {
                            kind: StatementKind::Tempo,
                            value: value.clone(),
                            indent: stmt.indent,
                            line: stmt.line,
                            column: stmt.column,
                        }
                    } else {
                        eprintln!("⚠️ Tempo variable '{}' not found", text);
                        Statement {
                            kind: StatementKind::Tempo,
                            value: VariableValue::Text(text.clone()),
                            indent: stmt.indent,
                            line: stmt.line,
                            column: stmt.column,
                        }
                    }
                }
                _ => {
                    eprintln!("⚠️ Invalid value type for Tempo statement: {:?}", stmt.value);
                    stmt.clone()
                }
            }
        }

        // TODO Handle other statement kinds

        _ => stmt.clone(),
    }
}
