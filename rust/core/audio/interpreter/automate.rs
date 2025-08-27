use crate::core::{
    parser::statement::{Statement, StatementKind},
    store::variable::VariableTable,
};

// Store automation configuration into the variable table under a namespaced key
// Key: "<target>__automation" => Value::Map({ target, params })
pub fn interprete_automate_statement(
    stmt: &Statement,
    variable_table: &mut VariableTable,
) -> Option<VariableTable> {
    if let StatementKind::Automate { target } = &stmt.kind {
        let key = format!("{}__automation", target);
        variable_table.set(key, stmt.value.clone());
        return Some(variable_table.clone());
    }
    None
}
