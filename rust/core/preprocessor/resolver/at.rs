use crate::core::types::{
    module::Module,
    statement::{ Statement, StatementResolved, StatementResolvedValue },
    variable::VariableValue,
};

pub fn resolve_load_statement(
    stmt: &Statement,
    source: &str,
    alias: &str,
    module: &mut Module
) -> StatementResolved {
    let source_string = source.to_string();

    module.set_variable(alias.to_string(), VariableValue::Sample(source_string.clone()));

    StatementResolved {
        kind: stmt.kind.clone(),
        value: StatementResolvedValue::Null,
        indent: stmt.indent,
        line: stmt.line,
        column: stmt.column,
    }
}
