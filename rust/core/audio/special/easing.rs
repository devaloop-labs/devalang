use crate::core::store::variable::VariableTable;

// Basic easing functions operating on t in [0,1]
fn easing_value(func: &str, t: f32) -> Option<f32> {
    let x = t.clamp(0.0, 1.0);
    match func {
        "linear" => Some(x),
        "easeInQuad" => Some(x * x),
        "easeOutQuad" => Some(x * (2.0 - x)),
        "easeInOutQuad" => {
            if x < 0.5 { Some(2.0 * x * x) } else { Some(-1.0 + (4.0 - 2.0 * x) * x) }
        }
        // Cubic
        "easeInCubic" => Some(x * x * x),
        "easeOutCubic" => Some(1.0 - (1.0 - x).powi(3)),
        "easeInOutCubic" => {
            if x < 0.5 { Some(4.0 * x * x * x) } else { Some(1.0 - (-2.0 * x + 2.0).powi(3) / 2.0) }
        }
        // Quartic
        "easeInQuart" => Some(x.powi(4)),
        "easeOutQuart" => Some(1.0 - (1.0 - x).powi(4)),
        "easeInOutQuart" => {
            if x < 0.5 { Some(8.0 * x.powi(4)) } else { Some(1.0 - (-2.0 * x + 2.0).powi(4) / 2.0) }
        }
        // Exponential
        "easeInExpo" => Some(if x <= 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * x - 10.0) }),
        "easeOutExpo" => Some(if x >= 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * x) }),
        "easeInOutExpo" => Some(if x <= 0.0 { 0.0 } else if x >= 1.0 { 1.0 } else if x < 0.5 { 2.0_f32.powf(20.0 * x - 10.0) / 2.0 } else { (2.0 - 2.0_f32.powf(-20.0 * x + 10.0)) / 2.0 }),
        // Back (overshoot c ~ 1.70158)
        "easeInBack" => { let c = 1.70158; Some((c + 1.0) * x * x * x - c * x * x) }
        "easeOutBack" => { let c = 1.70158; let y = 1.0 - x; Some(1.0 - ((c + 1.0) * y * y * y - c * y * y)) }
        "easeInOutBack" => {
            let c1 = 1.70158; let c2 = c1 * 1.525; let x2 = x * 2.0;
            if x2 < 1.0 { Some((x2 * x2 * ((c2 + 1.0) * x2 - c2)) / 2.0) } else {
                let x2 = x2 - 2.0; Some((x2 * x2 * ((c2 + 1.0) * x2 + c2)) / 2.0 + 1.0)
            }
        }
        // Elastic
        "easeInElastic" => {
            if x == 0.0 { Some(0.0) } else if x == 1.0 { Some(1.0) } else {
                let c = 2.0 * std::f32::consts::PI / 3.0;
                Some(- (2.0_f32.powf(10.0 * x - 10.0)) * ((x * 10.0 - 10.75) * c).sin())
            }
        }
        "easeOutElastic" => {
            if x == 0.0 { Some(0.0) } else if x == 1.0 { Some(1.0) } else {
                let c = 2.0 * std::f32::consts::PI / 3.0;
                Some(2.0_f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * c).sin() + 1.0)
            }
        }
        "easeInOutElastic" => {
            if x == 0.0 { Some(0.0) } else if x == 1.0 { Some(1.0) } else {
                let c = 2.0 * std::f32::consts::PI / 4.5;
                if x < 0.5 {
                    Some(-(2.0_f32.powf(20.0 * x - 10.0)) * ((20.0 * x - 11.125) * c).sin() / 2.0)
                } else {
                    Some(2.0_f32.powf(-20.0 * x + 10.0) * ((20.0 * x - 11.125) * c).sin() / 2.0 + 1.0)
                }
            }
        }
        // Bounce helpers
        "easeInBounce" => Some(1.0 - bounce_out(1.0 - x)),
        "easeOutBounce" => Some(bounce_out(x)),
        "easeInOutBounce" => Some(if x < 0.5 { (1.0 - bounce_out(1.0 - 2.0 * x)) / 2.0 } else { (1.0 + bounce_out(2.0 * x - 1.0)) / 2.0 }),
        _ => None,
    }
}

fn bounce_out(x: f32) -> f32 {
    let n1 = 7.5625; let d1 = 2.75;
    if x < 1.0 / d1 {
        n1 * x * x
    } else if x < 2.0 / d1 {
        let x = x - 1.5 / d1; n1 * x * x + 0.75
    } else if x < 2.5 / d1 {
        let x = x - 2.25 / d1; n1 * x * x + 0.9375
    } else {
        let x = x - 2.625 / d1; n1 * x * x + 0.984375
    }
}

// Find and evaluate the first $easing.<fn>(...) occurrence in the string.
// Accepts a single argument expression producing t in [0,1].
pub fn find_and_eval_first_easing_call<EvalFn>(
    s: &str,
    eval: EvalFn,
    vars: &VariableTable,
    bpm: f32,
    beat: f32,
) -> Option<String>
where
    EvalFn: Fn(&str, &VariableTable, f32, f32) -> Option<f32>,
{
    let start = s.find("$easing.")?;
    let open_rel = s[start..].find('(')?;
    let open = start + open_rel;
    let func = &s[start + 9..open];

    // Find matching close parenthesis
    let mut depth: i32 = 0;
    let mut close_abs: Option<usize> = None;
    for (i, ch) in s[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => { depth -= 1; if depth == 0 { close_abs = Some(open + i); break; } }
            _ => {}
        }
    }
    let close = close_abs?;

    let inner = &s[open + 1..close];
    let t = eval(inner, vars, bpm, beat)?;
    let result = easing_value(func, t)?;

    let mut replaced = String::new();
    replaced.push_str(&s[..start]);
    replaced.push_str(&result.to_string());
    replaced.push_str(&s[close + 1..]);
    Some(replaced)
}
