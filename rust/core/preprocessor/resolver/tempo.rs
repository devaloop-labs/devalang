use crate::core::types::{module::Module, statement::{Statement, StatementKind, StatementResolved, StatementResolvedValue}, variable::VariableValue};

pub fn resolve_tempo_statement(
    tempo_statement: &Statement,
    module: &Module
) -> StatementResolved {
    match &tempo_statement.value {
        VariableValue::Number(num) => {
            if *num > 0.0 {
                StatementResolved {
                    kind: StatementKind::Tempo,
                    value: StatementResolvedValue::Number(*num),
                    indent: tempo_statement.indent,
                    line: tempo_statement.line,
                    column: tempo_statement.column,
                }
            } else {
                eprintln!("⚠️ Invalid tempo value: {}", num);
                StatementResolved {
                    kind: StatementKind::Tempo,
                    value: StatementResolvedValue::Unknown,
                    indent: tempo_statement.indent,
                    line: tempo_statement.line,
                    column: tempo_statement.column,
                }
            }
        }
        VariableValue::Text(text) => {
            if let Some(value) = module.variable_table.variables.get(text) {
                let variable_value: StatementResolvedValue = match value {
                    VariableValue::Number(num) if *num > 0.0 => {
                        StatementResolvedValue::Number(*num)
                    }
                    VariableValue::Text(t) => StatementResolvedValue::String(t.clone()),
                    _ => {
                        eprintln!("⚠️ Unsupported variable type for Tempo: {:?}", value);
                        StatementResolvedValue::Unknown
                    }
                };

                StatementResolved {
                    kind: StatementKind::Tempo,
                    value: variable_value,
                    indent: tempo_statement.indent,
                    line: tempo_statement.line,
                    column: tempo_statement.column,
                }
            } else {
                eprintln!("⚠️ Tempo variable '{}' not found", text);
                StatementResolved {
                    kind: StatementKind::Tempo,
                    value: StatementResolvedValue::Unknown,
                    indent: tempo_statement.indent,
                    line: tempo_statement.line,
                    column: tempo_statement.column,
                }
            }
        }
        _ => {
            eprintln!("⚠️ Invalid value type for Tempo statement: {:?}", tempo_statement.value);
            StatementResolved {
                kind: StatementKind::Tempo,
                value: StatementResolvedValue::Unknown,
                indent: tempo_statement.indent,
                line: tempo_statement.line,
                column: tempo_statement.column,
            }
        }
    }
}
