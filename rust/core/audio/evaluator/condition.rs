use crate::core::audio::special::resolve_env_atom;
use devalang_types::Value;
use devalang_types::VariableTable;

pub fn evaluate_condition_string(expr: &str, vars: &VariableTable) -> bool {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.len() != 3 {
        return false;
    }

    let left = tokens[0];
    let op = tokens[1];
    let right = tokens[2];

    // Resolve left and right to numeric values where possible. Accept numbers, variables or env atoms.
    fn resolve_for_cond(s: &str, vars: &VariableTable) -> Option<f32> {
        if let Ok(n) = s.parse::<f32>() {
            return Some(n);
        }
        if let Some(Value::Number(n)) = vars.get(s) {
            return Some(*n);
        }
        if let Some(v) = resolve_env_atom(s, 120.0, 1.0) {
            return Some(v);
        }
        None
    }

    let left_val = match resolve_for_cond(left, vars) {
        Some(v) => v,
        None => {
            return false;
        }
    };

    let right_val = match resolve_for_cond(right, vars) {
        Some(v) => v,
        None => {
            return false;
        }
    };

    match op {
        ">" => left_val > right_val,
        "<" => left_val < right_val,
        ">=" => left_val >= right_val,
        "<=" => left_val <= right_val,
        "==" => {
            // relative epsilon for floating comparisons
            let diff = (left_val - right_val).abs();
            let largest = left_val.abs().max(right_val.abs()).max(1.0);
            diff <= f32::EPSILON * largest
        }
        "!=" => {
            let diff = (left_val - right_val).abs();
            let largest = left_val.abs().max(right_val.abs()).max(1.0);
            diff > f32::EPSILON * largest
        }
        _ => false,
    }
}
