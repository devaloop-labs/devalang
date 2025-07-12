use crate::core::{
    audio::engine::AudioEngine,
    parser::statement::{ Statement, StatementKind },
    shared::value::Value,
    store::variable::VariableTable,
};

pub fn interprete_let_statement(
    stmt: &Statement,
    variable_table: &mut VariableTable
) -> Option<VariableTable> {
    if let StatementKind::Let { name } = &stmt.kind {
        variable_table.set(name.to_string(), stmt.value.clone());

        return Some(variable_table.clone())
    } 
    
    None
}
