use crate::core::audio::evaluator::numeric::evaluate_numeric_expression;
use devalang_types::Value;
use devalang_types::VariableTable;

// Evaluate a simple string concatenation expression like: "hello " + name + "!" + $env.beat
// - Splits on + outside quotes
// - Terms can be string literals (double quotes), variables (Number/String/Boolean), or numeric env/math expressions
// Returns None if parsing fails (fallback to raw print)
pub fn evaluate_string_expression(
    expr: &str,
    vars: &VariableTable,
    env_bpm: f32,
    env_beat: f32,
) -> Option<String> {
    // Quick reject if no '+' present
    if !expr.contains('+') {
        return None;
    }

    // Split by '+' outside of quotes
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut escape = false;
    for ch in expr.chars() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            // escape next char
            escape = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            cur.push(ch);
            continue;
        }
        if ch == '+' && !in_quotes {
            parts.push(cur.to_string());
            cur.clear();
            continue;
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        parts.push(cur.to_string());
    }
    if parts.is_empty() {
        return None;
    }

    // Resolve each part into a string
    fn strip_quotes(s: &str) -> Option<String> {
        let st = s.trim();
        if st.len() >= 2 && st.starts_with('"') && st.ends_with('"') {
            Some(st[1..st.len() - 1].to_string())
        } else {
            None
        }
    }

    let mut out = String::new();
    for p in parts {
        if p.is_empty() {
            continue;
        }
        if let Some(lit) = strip_quotes(&p) {
            out.push_str(&lit);
            continue;
        }
        // Try variables first
        if let Some(val) = vars.get(&p) {
            match val {
                Value::String(s) => out.push_str(s),
                Value::Number(n) => out.push_str(&format!("{}", n)),
                Value::Boolean(b) => out.push_str(&format!("{}", b)),
                other => out.push_str(&format!("{:?}", other)),
            }
            continue;
        }
        // Try env/math/numeric expression for this term
        if let Some(n) = evaluate_numeric_expression(&p, vars, env_bpm, env_beat) {
            out.push_str(&format!("{}", n));
            continue;
        }
        // Bareword not resolved: include as-is (safe fallback)
        out.push_str(&p);
    }

    Some(out)
}
