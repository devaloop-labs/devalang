use crate::core::{ shared::value::Value, store::variable::VariableTable };

pub fn evaluate_condition_string(expr: &str, vars: &VariableTable) -> bool {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.len() != 3 {
        return false;
    }

    let left = tokens[0];
    let op = tokens[1];
    let right = tokens[2];

    let left_val = match vars.get(left) {
        Some(Value::Number(n)) => *n,
        _ => {
            return false;
        }
    };

    let right_val: f32 = right.parse().unwrap_or(0.0);

    match op {
        ">" => left_val > right_val,
        "<" => left_val < right_val,
        ">=" => left_val >= right_val,
        "<=" => left_val <= right_val,
        "==" => (left_val - right_val).abs() < f32::EPSILON,
        "!=" => (left_val - right_val).abs() > f32::EPSILON,
        _ => false,
    }
}
