use crate::core::{
    debugger::Debugger,
    preprocessor::resolver::resolve_statement,
    types::{ module::Module, statement::{ StatementKind, StatementResolved } },
};

/// Exécute tous les statements d'un module avec résolution des variables
pub fn execute_statements(module: &Module, debugger: &Debugger) -> Vec<StatementResolved> {
    println!("▶️ Executing statements for module: {}", module.path);

    let mut resolved_statements: Vec<StatementResolved> = Vec::new();

    for stmt in &module.statements {
        match &stmt.kind {
            StatementKind::Tempo { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            StatementKind::Trigger { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            StatementKind::Bank { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            StatementKind::Loop { .. } => {
                let resolved = resolve_statement(stmt, &module);
                resolved_statements.push(resolved);
            }
            _ => {}
        }
    }

    resolved_statements
}
