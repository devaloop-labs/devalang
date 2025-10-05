/// Expression evaluator - evaluates conditions and expressions at runtime
use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Evaluate a condition expression
pub fn evaluate_condition(condition: &Value, _context: &HashMap<String, Value>) -> bool {
    match condition {
        Value::Boolean(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::String(s) | Value::Identifier(s) => !s.is_empty(),
        Value::Null => false,
        _ => true,
    }
}

/// Evaluate a numeric expression
pub fn evaluate_number(expr: &Value, context: &HashMap<String, Value>) -> f32 {
    match expr {
        Value::Number(n) => *n,
        Value::Identifier(name) => {
            if let Some(val) = context.get(name) {
                evaluate_number(val, context)
            } else {
                0.0
            }
        }
        Value::String(s) => s.parse::<f32>().unwrap_or(0.0),
        Value::Boolean(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_condition_boolean() {
        let ctx = HashMap::new();
        assert!(evaluate_condition(&Value::Boolean(true), &ctx));
        assert!(!evaluate_condition(&Value::Boolean(false), &ctx));
    }

    #[test]
    fn test_evaluate_number() {
        let mut ctx = HashMap::new();
        ctx.insert("x".to_string(), Value::Number(42.0));

        let result = evaluate_number(&Value::Identifier("x".to_string()), &ctx);
        assert!((result - 42.0).abs() < f32::EPSILON);
    }
}
