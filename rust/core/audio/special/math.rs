use crate::core::store::variable::VariableTable;
use devalang_utils::logger::{LogLevel, Logger};

// Parse comma-separated arguments at top level (no nested parentheses split)
fn parse_top_level_args(s: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                args.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = s[start..].trim();
    if !last.is_empty() {
        args.push(last);
    }
    args
}

fn eval_math_func(func: &str, args: &[f32], fallback_seed: f32) -> Option<f32> {
    match func {
        "sin" => args.first().copied().map(f32::sin),
        "cos" => args.first().copied().map(f32::cos),
        "random" => {
            // deterministic pseudo-random based on provided seed or a fallback session seed
            let seed = args.first().copied().unwrap_or(fallback_seed);
            let x = (seed * 12.9898).sin() * 43_758.547;
            Some((x.fract() * 2.0 - 1.0).clamp(-1.0, 1.0))
        }
        "lerp" => {
            if args.len() >= 3 {
                Some(args[0] + (args[1] - args[0]) * args[2])
            } else {
                None
            }
        }
        _ => None,
    }
}

// Find and evaluate the first $math.<fn>(...) occurrence in the string, replacing it with a number.
// Supports multi-argument functions by splitting on top-level commas.
pub fn find_and_eval_first_math_call<EvalFn>(
    s: &str,
    eval: EvalFn,
    vars: &VariableTable,
    bpm: f32,
    beat: f32,
) -> Option<String>
where
    EvalFn: Fn(&str, &VariableTable, f32, f32) -> Option<f32>,
{
    let logger = Logger::new();

    let start = s.find("$math.")?;
    let open_rel = match s[start..].find('(') {
        Some(i) => i,
        None => {
            logger.log_message(
                LogLevel::Error,
                &format!("Malformed $math call: missing '(' in '{}'", s),
            );
            return None;
        }
    };
    let open = start + open_rel;
    if open <= start + 6 {
        logger.log_message(
            LogLevel::Error,
            &format!("Malformed $math call: missing function name in '{}'", s),
        );
        return None;
    }
    let func = &s[start + 6..open];

    // Find matching close parenthesis, handling nesting
    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (i, ch) in s[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close_abs = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = match close_abs {
        Some(c) => c,
        None => {
            logger.log_message(
                LogLevel::Error,
                &format!("Malformed $math call: missing closing ')' in '{}'", s),
            );
            return None;
        }
    };

    let inner = &s[open + 1..close];
    let raw_args = parse_top_level_args(inner);
    let mut args: Vec<f32> = Vec::with_capacity(raw_args.len());
    for a in raw_args {
        if let Some(v) = eval(a, vars, bpm, beat) {
            args.push(v);
        } else {
            logger.log_message(
                LogLevel::Error,
                &format!("Failed to evaluate argument '{}' for $math.{}", a, func),
            );
            return None;
        }
    }

    // If no explicit seed is provided, use $env.seed via fallback
    let fallback_seed = eval("$env.seed", vars, bpm, beat).unwrap_or(0.0);
    let result = eval_math_func(func, &args, fallback_seed)?;

    let mut replaced = String::new();
    replaced.push_str(&s[..start]);
    replaced.push_str(&result.to_string());
    replaced.push_str(&s[close + 1..]);
    Some(replaced)
}
