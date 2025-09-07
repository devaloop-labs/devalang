use crate::core::audio::special::{
    find_and_eval_first_easing_call, find_and_eval_first_math_call, find_and_eval_first_mod_call,
    resolve_env_atom,
};
use devalang_types::Value;
use devalang_types::VariableTable;

// Very small expression evaluator for `$env.*`, `$math.*` and variables.
// Supports: +, -, *, / and simple parentheses, left-to-right (no precedence), and $math.(sin|cos)(expr)
pub fn evaluate_numeric_expression(
    expr: &str,
    vars: &VariableTable,
    env_bpm: f32,
    env_beat: f32,
) -> Option<f32> {
    let expr = expr.replace(" ", "");

    // Helper to resolve an atom to a number
    fn resolve_atom(atom: &str, vars: &VariableTable, bpm: f32, beat: f32) -> Option<f32> {
        if let Some(v) = resolve_env_atom(atom, bpm, beat) {
            return Some(v);
        }
        if let Ok(n) = atom.parse::<f32>() {
            return Some(n);
        }
        if let Some(Value::Number(n)) = vars.get(atom) {
            return Some(*n);
        }
        None
    }

    // Shunting-like, simplified: first evaluate any $math.func(...) calls anywhere in the expression,
    // then fold remaining parentheses and evaluate left-to-right.
    fn eval(expr: &str, vars: &VariableTable, bpm: f32, beat: f32) -> Option<f32> {
        // 1) Replace $math/$easing/$mod calls progressively with a max iteration guard
        let mut s = expr.to_string();
        let mut iterations = 0u32;
        const MAX_ITER: u32 = 64;

        // Evaluate modulators first (they may feed easing/math)
        while iterations < MAX_ITER {
            if let Some(next) =
                find_and_eval_first_mod_call(&s, evaluate_numeric_expression, vars, bpm, beat)
            {
                s = next;
                iterations += 1;
                continue;
            }
            break;
        }

        iterations = 0;
        while iterations < MAX_ITER {
            if let Some(next) =
                find_and_eval_first_easing_call(&s, evaluate_numeric_expression, vars, bpm, beat)
            {
                s = next;
                iterations += 1;
                continue;
            }
            break;
        }

        iterations = 0;
        while iterations < MAX_ITER {
            if let Some(next) =
                find_and_eval_first_math_call(&s, evaluate_numeric_expression, vars, bpm, beat)
            {
                s = next;
                iterations += 1;
                continue;
            }
            break;
        }

        // 2) Evaluate remaining (pure) parentheses starting from innermost
        if let Some(open) = s.rfind('(') {
            if let Some(close_rel) = s[open..].find(')') {
                // index relatif
                let close = open + close_rel;
                let inner = &s[open + 1..close];
                let val = eval(inner, vars, bpm, beat)?;
                let mut replaced = String::new();
                replaced.push_str(&s[..open]);
                replaced.push_str(&val.to_string());
                replaced.push_str(&s[close + 1..]);
                return eval(&replaced, vars, bpm, beat);
            }
        }

        // Tokenize by operators left-to-right
        let mut parts: Vec<String> = Vec::new();
        let mut cur = String::new();
        for ch in s.chars() {
            if "+-*/".contains(ch) {
                if !cur.is_empty() {
                    parts.push(cur.clone());
                    cur.clear();
                }
                parts.push(ch.to_string());
            } else {
                cur.push(ch);
            }
        }
        if !cur.is_empty() {
            parts.push(cur);
        }
        if parts.is_empty() {
            return None;
        }

        // Resolve atoms and compute
        let mut acc: Option<f32> = None;
        let mut op: Option<char> = None;
        for part in parts {
            if part.len() == 1 && "+-*/".contains(part.chars().next().unwrap()) {
                op = part.chars().next();
                continue;
            }
            let val = if let Some(v) = resolve_atom(&part, vars, bpm, beat) {
                v
            } else if part.starts_with("$env.") {
                // $env atom not handled by resolve_atom (when composed), try recursive eval
                eval(&part, vars, bpm, beat)?
            } else {
                return None;
            };

            acc = Some(match (acc, op) {
                (None, _) => val,
                (Some(a), Some('+')) => a + val,
                (Some(a), Some('-')) => a - val,
                (Some(a), Some('*')) => a * val,
                (Some(a), Some('/')) => {
                    if val != 0.0 {
                        a / val
                    } else {
                        return Some(f32::INFINITY);
                    }
                }
                (Some(_), None) => val,
                _ => {
                    return None;
                }
            });
        }

        acc
    }

    eval(&expr, vars, env_bpm, env_beat)
}
