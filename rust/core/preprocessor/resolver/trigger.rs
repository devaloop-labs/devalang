use std::collections::HashMap;

use crate::core::{
    parser::parse_with_resolving_with_module,
    types::{
        module::Module,
        statement::{ Statement, StatementKind, StatementResolved, StatementResolvedValue },
        token::{ TokenDuration, TokenParamValue },
        variable::VariableValue,
    },
};

pub fn resolve_trigger_statement(
    stmt: &Statement,
    entity: String,
    duration: TokenDuration,
    module: &Module
) -> StatementResolved {
    let mut entity_value = VariableValue::Unknown;

    if let Some(value) = module.variable_table.variables.get(&entity) {
        entity_value = value.clone();
    } else {
        entity_value = VariableValue::Text(entity.clone());
    }

    let duration_raw_value = match duration {
        TokenDuration::Auto => "auto",
        TokenDuration::Infinite => "infinite",
        TokenDuration::Number(n) => &n.to_string(),
        TokenDuration::Identifier(ref id) => id.as_str(),
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

    match &stmt.value {
        VariableValue::Text(text) => {
            if let Some(value) = module.variable_table.variables.get(text) {
                let parsed_entity_value: StatementResolvedValue = match value {
                    VariableValue::Array(arr) => {
                        StatementResolvedValue::Array(
                            parse_with_resolving_with_module(arr.clone(), module)
                        )
                    }
                    VariableValue::Map(map) => {
                        let mut resolved_map = HashMap::new();

                        for (key, value) in map {
                            let resolved_value = match value {
                                TokenParamValue::String(text) =>
                                    StatementResolvedValue::String(text.clone()),
                                TokenParamValue::Number(num) =>
                                    StatementResolvedValue::Number(*num),
                                TokenParamValue::Boolean(b) => StatementResolvedValue::Boolean(*b),
                                _ => {
                                    eprintln!(
                                        "⚠️ Unsupported variable type for Trigger map: {:?}",
                                        value
                                    );
                                    StatementResolvedValue::Unknown
                                }
                            };
                            resolved_map.insert(key.clone(), resolved_value);
                        }

                        StatementResolvedValue::Map(resolved_map)
                    }

                    | VariableValue::Text(_)
                    | VariableValue::Number(_)
                    | VariableValue::Boolean(_) => {
                        StatementResolvedValue::String(text.clone())
                    }
                    _ => {
                        eprintln!("⚠️ Unsupported variable type for Trigger entity: {:?}", value);
                        StatementResolvedValue::Unknown
                    }
                };

                StatementResolved {
                    kind: StatementKind::Trigger {
                        entity: entity.clone(),
                        duration: parsed_duration_value,
                    },
                    value: parsed_entity_value,
                    indent: stmt.indent,
                    line: stmt.line,
                    column: stmt.column,
                }
            } else {
                eprintln!("⚠️ Trigger variable '{}' not found", text);
                StatementResolved {
                    kind: StatementKind::Trigger {
                        entity: entity.clone(),
                        duration: parsed_duration_value,
                    },
                    value: StatementResolvedValue::Unknown,
                    indent: stmt.indent,
                    line: stmt.line,
                    column: stmt.column,
                }
            }
        }
        VariableValue::Map(map) => {
            let mut resolved_map = HashMap::new();

            // TODO Handle nested maps and arrays

            StatementResolved {
                kind: StatementKind::Trigger {
                    entity: entity.clone(),
                    duration: parsed_duration_value,
                },
                value: StatementResolvedValue::Map(resolved_map),
                indent: stmt.indent,
                line: stmt.line,
                column: stmt.column,
            }
        }
        VariableValue::Null => {
            StatementResolved {
                kind: StatementKind::Trigger {
                    entity: entity.clone(),
                    duration: parsed_duration_value,
                },
                value: StatementResolvedValue::Map(HashMap::new()),
                indent: stmt.indent,
                line: stmt.line,
                column: stmt.column,
            }
        }

        // TODO Parse other parameters

        _ => {
            eprintln!("⚠️ Invalid value type for Trigger statement: {:?}", stmt.value);

            StatementResolved {
                kind: StatementKind::Trigger {
                    entity: entity.clone(),
                    duration: parsed_duration_value,
                },
                value: StatementResolvedValue::Unknown,
                indent: stmt.indent,
                line: stmt.line,
                column: stmt.column,
            }
        }
    }
}
