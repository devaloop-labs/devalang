use devalang_types::Value;

use crate::core::{
    parser::statement::{Statement, StatementKind},
    store::variable::VariableTable,
};

pub fn interprete_load_statement(
    stmt: &Statement,
    variable_table: &mut VariableTable,
) -> Option<VariableTable> {
    if let StatementKind::Load { source, alias } = &stmt.kind {
        variable_table.set(alias.to_string(), Value::Sample(source.clone()));

        return Some(variable_table.clone());
    }

    None
}
