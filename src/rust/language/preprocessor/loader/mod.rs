use crate::language::syntax::ast::{Statement, StatementKind, Value};
use crate::language::syntax::parser::driver::SimpleParser;
/// Module loader - handles file loading and module dependencies
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::Path;

/// Load and parse a module from a file path.
pub fn load_module_from_path(path: &Path) -> Result<Vec<Statement>> {
    SimpleParser::parse_file(path)
}

/// Inject exported variables into a destination variable map.
pub fn inject_dependencies(
    exports: &ModuleExports,
    destination: &mut HashMap<String, Value>,
) -> Result<()> {
    for (name, value) in &exports.variables {
        if destination.insert(name.clone(), value.clone()).is_some() {
            return Err(anyhow!("Duplicate dependency '{}'", name));
        }
    }

    Ok(())
}

/// Read a module file and return a map of exported variable names -> Value
pub struct ModuleExports {
    pub variables: HashMap<String, Value>,
    pub groups: HashMap<String, Vec<Statement>>,
    pub patterns: HashMap<String, Statement>,
}

pub fn load_module_exports(path: &Path) -> Result<ModuleExports> {
    let mut variables: HashMap<String, Value> = HashMap::new();
    let mut groups: HashMap<String, Vec<Statement>> = HashMap::new();
    let mut patterns: HashMap<String, Statement> = HashMap::new();

    let stmts = SimpleParser::parse_file(path)?;

    // collect exported names
    let mut exports: Vec<String> = Vec::new();
    for s in &stmts {
        if let StatementKind::Export { names, .. } = &s.kind {
            for n in names {
                exports.push(n.clone());
            }
        }
    }

    for s in &stmts {
        match &s.kind {
            StatementKind::Let { name, value }
            | StatementKind::Var { name, value }
            | StatementKind::Const { name, value } => {
                if exports.contains(name) {
                    if let Some(v) = value {
                        variables.insert(name.clone(), v.clone());
                    }
                }
            }
            StatementKind::Group {
                name,
                body,
                duration: _,
            } => {
                if exports.contains(name) {
                    groups.insert(name.clone(), body.clone());
                }
            }
            StatementKind::Pattern { name, .. } => {
                if exports.contains(name) {
                    patterns.insert(name.clone(), s.clone());
                }
            }
            _ => {}
        }
    }

    Ok(ModuleExports {
        variables,
        groups,
        patterns,
    })
}
