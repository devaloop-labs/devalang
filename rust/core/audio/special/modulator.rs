use crate::core::store::variable::VariableTable;

fn lfo_sine(rate_per_beat: f32, beat: f32) -> f32 {
    // Output in [-1,1]
    (2.0 * std::f32::consts::PI * rate_per_beat * beat).sin()
}

fn lfo_triangle(rate_per_beat: f32, beat: f32) -> f32 {
    // Triangle in [-1,1]
    let phase = (rate_per_beat * beat).fract();
    // Map [0,1]->[-1,1] tri
    4.0 * (phase - 0.5).abs() - 1.0
}

fn adsr_envelope_value_t(attack: f32, decay: f32, sustain: f32, release: f32, t: f32) -> f32 {
    let a = attack.max(0.0);
    let d = decay.max(0.0);
    let r = release.max(0.0);
    let s = sustain.clamp(0.0, 1.0);

    // Normalize phases so that the whole ADSR spans t in [0,1]
    let total = (a + d + r).max(1e-6);
    let ap = a / total;
    let dp = d / total;
    let rp = r / total;

    if t < ap {
        // attack (0->1)
        if ap > 0.0 { t / ap } else { 1.0 }
    } else if t < ap + dp {
        // decay (1->sustain)
        let u = (t - ap) / dp.max(1e-6);
        1.0 - (1.0 - s) * u
    } else if t < 1.0 - rp {
        // sustain
        s
    } else {
        // release (sustain->0)
        let u = (t - (1.0 - rp)) / rp.max(1e-6);
        s * (1.0 - u)
    }
}

fn eval_mod_func(func: &str, args: &[f32], beat: f32) -> Option<f32> {
    match func {
        "lfo.sine" => {
            let rate = args.get(0).copied().unwrap_or(1.0);
            Some(lfo_sine(rate, beat))
        }
        "lfo.tri" | "lfo.triangle" => {
            let rate = args.get(0).copied().unwrap_or(1.0);
            Some(lfo_triangle(rate, beat))
        }
        // ADSR envelope normalized over t in [0,1]
        // $mod.envelope(attack, decay, sustain, release, t)
        "envelope" | "mod.envelope" => {
            if args.len() >= 5 {
                Some(adsr_envelope_value_t(
                    args[0],
                    args[1],
                    args[2],
                    args[3],
                    args[4].clamp(0.0, 1.0),
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

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

// Find and evaluate the first $mod.<fn>(...) occurrence in the string.
pub fn find_and_eval_first_mod_call<EvalFn>(
    s: &str,
    eval: EvalFn,
    vars: &VariableTable,
    bpm: f32,
    beat: f32,
) -> Option<String>
where
    EvalFn: Fn(&str, &VariableTable, f32, f32) -> Option<f32>,
{
    let start = s.find("$mod.")?;
    let open_rel = s[start..].find('(')?;
    let open = start + open_rel;
    let func = &s[start + 5..open];

    // matching close
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
    let close = close_abs?;

    let inner = &s[open + 1..close];
    let raw_args = parse_top_level_args(inner);
    let mut args: Vec<f32> = Vec::with_capacity(raw_args.len());
    for a in raw_args {
        args.push(eval(a, vars, bpm, beat)?);
    }

    let result = eval_mod_func(func, &args, beat)?;

    let mut replaced = String::new();
    replaced.push_str(&s[..start]);
    replaced.push_str(&result.to_string());
    replaced.push_str(&s[close + 1..]);
    Some(replaced)
}
