use crate::core::{parser::parse_with_resolving_with_module, types::{
    module::Module,
    statement::{ Statement, StatementKind, StatementResolved, StatementResolvedValue },
    variable::VariableValue,
}};

pub fn resolve_bank_statement(stmt: &Statement, module: &Module) -> StatementResolved {
    match &stmt.value {
        VariableValue::Text(name) => {
            if
                let Some(value) = module.import_table.imports
                    .get(name)
                    .or_else(|| module.variable_table.variables.get(name))
            {
                let statement_value: StatementResolvedValue = match value {
                    VariableValue::Array(arr) => {
                        StatementResolvedValue::Array(
                            parse_with_resolving_with_module(arr.clone(), module)
                        )
                    }
                    VariableValue::Text(text) => StatementResolvedValue::String(text.clone()),
                    VariableValue::Number(num) => StatementResolvedValue::Number(*num),
                    VariableValue::Boolean(b) => StatementResolvedValue::Boolean(*b),
                    _ => {
                        eprintln!("⚠️ Unsupported variable type for Bank: {:?}", value);
                        StatementResolvedValue::Unknown
                    }
                };

                StatementResolved {
                    kind: StatementKind::Bank,
                    value: statement_value,
                    indent: stmt.indent,
                    line: stmt.line,
                    column: stmt.column,
                }
            } else {
                eprintln!("⚠️ Bank variable '{}' not found", name);
                StatementResolved {
                    kind: StatementKind::Bank,
                    value: StatementResolvedValue::Unknown,
                    indent: stmt.indent,
                    line: stmt.line,
                    column: stmt.column,
                }
            }
        }
        _ => {
            eprintln!("⚠️ Invalid value type for Bank statement: {:?}", stmt.value);
            StatementResolved {
                kind: StatementKind::Bank,
                value: StatementResolvedValue::Unknown,
                indent: stmt.indent,
                line: stmt.line,
                column: stmt.column,
            }
        }
    }
}
