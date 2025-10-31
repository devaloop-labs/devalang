use crate::language::syntax::ast::{Statement, Value};
use std::collections::HashMap;

pub fn resolve_bank(_stmt: &Statement, _vars: &HashMap<String, Value>) -> Statement {
    _stmt.clone()
}
