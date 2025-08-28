use crate::core::{
    parser::statement::{Statement, StatementKind},
    store::function::{FunctionDef, FunctionTable},
};

pub fn interprete_function_statement(
    stmt: &Statement,
    functions_table: &mut FunctionTable,
) -> Option<FunctionTable> {
    if let StatementKind::Function {
        name,
        parameters,
        body,
    } = &stmt.kind
    {
        functions_table.add_function(FunctionDef {
            name: name.clone(),
            parameters: parameters.clone(),
            body: body.clone(),
        });

        return Some(functions_table.clone());
    }

    None
}
