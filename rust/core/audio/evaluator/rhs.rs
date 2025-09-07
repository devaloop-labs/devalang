use crate::core::audio::evaluator::numeric::evaluate_numeric_expression;
use devalang_types::Value;
use devalang_types::VariableTable;

pub fn evaluate_rhs_into_value(
    raw: &str,
    vars: &VariableTable,
    env_bpm: f32,
    env_beat: f32,
) -> Value {
    if let Some(num) = evaluate_numeric_expression(raw, vars, env_bpm, env_beat) {
        Value::Number(num)
    } else {
        Value::String(raw.to_string())
    }
}
