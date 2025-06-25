use crate::core::{
    preprocessor::resolver::resolve_statement,
    types::{
        module::Module,
        statement::{ StatementKind, StatementResolved, StatementResolvedValue },
    },
};

/// Exécute tous les statements d'un module avec résolution des variables
pub fn execute_statements(module: &mut Module) -> Vec<StatementResolved> {
    let mut resolved_statements: Vec<StatementResolved> = Vec::new();

    if module.clone().statements.is_empty() {
        return resolved_statements;
    }

    for stmt in module.clone().statements {
        match &stmt.kind {
            StatementKind::Tempo { .. } => {
                let resolved = resolve_statement(&stmt, module);
                resolved_statements.push(resolved);
            }
            StatementKind::Trigger { .. } => {
                let resolved = resolve_statement(&stmt, module);
                resolved_statements.push(resolved);
            }
            StatementKind::Bank { .. } => {
                let resolved = resolve_statement(&stmt, module);
                resolved_statements.push(resolved);
            }
            StatementKind::Loop { .. } => {
                let resolved = resolve_statement(&stmt, module);
                resolved_statements.push(resolved);
            }
            StatementKind::Load { .. } => {
                let resolved = resolve_statement(&stmt, module);
                resolved_statements.push(resolved);
            }
            _ => {}
        }
    }

    resolved_statements
}
