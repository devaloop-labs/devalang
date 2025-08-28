use crate::core::{
    parser::statement::{Statement, StatementKind},
    shared::value::Value,
    store::variable::VariableTable,
};

pub fn interprete_let_statement(
    stmt: &Statement,
    variable_table: &mut VariableTable,
) -> Option<VariableTable> {
    if let StatementKind::Let { name } = &stmt.kind {
        // If RHS is a string and looks like an expression, evaluate it
        let evaluated = match &stmt.value {
            Value::String(s) if s.contains("$env") || s.contains("$math") => {
                // We don't have direct env here; use defaults or infer from table
                let bpm = if let Some(Value::Number(n)) = variable_table.get("bpm") {
                    *n
                } else {
                    120.0
                };
                // Try to infer beat from time-based variables if any, else 0.0
                let beat = if let Some(Value::Number(n)) = variable_table.get("beat") {
                    *n
                } else {
                    0.0
                };
                crate::core::audio::evaluator::evaluate_rhs_into_value(s, variable_table, bpm, beat)
            }
            other => other.clone(),
        };

        variable_table.set(name.to_string(), evaluated);

        return Some(variable_table.clone());
    }

    None
}
