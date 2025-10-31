/// Curves system for Devalang
/// Provides easing functions and animation curves for automations
use crate::language::syntax::ast::Value;
use std::f32::consts::PI;

/// Easing curve types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    // Basic easing
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,

    // Advanced easing
    Swing(f32),                 // Swing with optional intensity (0.0-1.0)
    Bounce(f32),                // Bounce with optional height (0.0-1.0)
    Elastic(f32),               // Elastic with optional intensity
    Bezier(f32, f32, f32, f32), // Custom bezier with control points

    // Noise-based
    Random,
    Perlin, // Perlin noise-based curve

    // Special
    StepFunction(f32), // Number of steps
}

/// Evaluate a curve at progress (0.0 to 1.0)
pub fn evaluate_curve(curve: CurveType, progress: f32) -> f32 {
    let p = progress.clamp(0.0, 1.0);

    match curve {
        CurveType::Linear => p,

        CurveType::EaseIn => p * p,
        CurveType::EaseOut => 1.0 - (1.0 - p) * (1.0 - p),
        CurveType::EaseInOut => {
            if p < 0.5 {
                2.0 * p * p
            } else {
                1.0 - (-2.0 * p + 2.0).powi(2) / 2.0
            }
        }

        CurveType::Swing(intensity) => {
            let i = intensity.clamp(0.0, 1.0);
            let swing_amount = 0.5 * i;
            if p < 0.5 {
                let local_p = p * 2.0;
                (1.0 + swing_amount) * local_p - swing_amount * local_p * local_p
            } else {
                let local_p = (p - 0.5) * 2.0;
                1.0 - ((1.0 + swing_amount) * (1.0 - local_p)
                    - swing_amount * (1.0 - local_p) * (1.0 - local_p))
            }
        }

        CurveType::Bounce(height) => {
            let h = height.clamp(0.0, 1.0);
            let bounce_height = 0.5 * h;

            // Simplified bounce curve
            if p < 0.5 {
                let local_p = p * 2.0;
                local_p / 2.0 + bounce_height * (PI * local_p).sin()
            } else {
                let local_p = (p - 0.5) * 2.0;
                0.5 + local_p / 2.0 + bounce_height * (PI * (1.0 - local_p)).sin()
            }
        }

        CurveType::Elastic(intensity) => {
            let i = intensity.clamp(0.0, 1.0);
            let n = 5.0 * i; // Number of bounces
            p * ((n * PI * p).sin() * 0.5 + 1.0)
        }

        CurveType::Bezier(x1, y1, x2, y2) => {
            // Simplified cubic bezier evaluation
            bezier(p, x1, y1, x2, y2)
        }

        CurveType::Random => {
            // Deterministic "random" based on input (using sine for pseudo-randomness)
            ((p * 12.9898).sin() * 43758.5453_f32).fract()
        }

        CurveType::Perlin => {
            // Simplified Perlin-like noise
            let t = p * 3.0;
            let i = t.floor();
            let f = t - i;
            let u = f * f * (3.0 - 2.0 * f); // Smooth step

            // Pseudo-random gradients
            let g0 = ((i * 12.9898).sin() * 43758.5453_f32).fract();
            let g1 = (((i + 1.0) * 12.9898).sin() * 43758.5453_f32).fract();

            g0.lerp(g1, u)
        }

        CurveType::StepFunction(steps) => {
            let n = steps.max(1.0);
            (p * n).floor() / n
        }
    }
}

/// Cubic bezier evaluation using Newton's method
fn bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Find X value for given t
    let mut u = t;
    for _ in 0..4 {
        let cu = (1.0 - u).powi(3)
            + 3.0 * (1.0 - u).powi(2) * u * x1
            + 3.0 * (1.0 - u) * u.powi(2) * x2
            + u.powi(3);
        let der = 3.0 * (1.0 - u).powi(2) * (x1 - 1.0)
            + 6.0 * (1.0 - u) * u * (x2 - x1)
            + 3.0 * u.powi(2) * (1.0 - x2);

        u -= (cu - t) / der;
    }

    // Calculate Y from U
    (1.0 - u).powi(3)
        + 3.0 * (1.0 - u).powi(2) * u * y1
        + 3.0 * (1.0 - u) * u.powi(2) * y2
        + u.powi(3)
}

/// Parse a curve definition from special variable syntax
/// Examples:
/// - $curve.linear
/// - $curve.easeIn
/// - $curve.swing(0.5)
/// - $ease.bezier(0.25, 0.1, 0.25, 1.0)
pub fn parse_curve(name: &str) -> Option<CurveType> {
    if name.starts_with("$curve.") {
        let curve_name = &name[7..]; // Remove "$curve."
        parse_curve_name(curve_name)
    } else if name.starts_with("$ease.") {
        let ease_name = &name[6..]; // Remove "$ease."
        parse_ease_name(ease_name)
    } else {
        None
    }
}

fn parse_curve_name(name: &str) -> Option<CurveType> {
    match name {
        "linear" => Some(CurveType::Linear),
        "in" => Some(CurveType::EaseIn),
        "out" => Some(CurveType::EaseOut),
        "inOut" => Some(CurveType::EaseInOut),
        "random" => Some(CurveType::Random),
        "perlin" => Some(CurveType::Perlin),

        // Parameterized curves
        _ if name.starts_with("swing(") => {
            let intensity = extract_param(name, "swing")?;
            Some(CurveType::Swing(intensity))
        }
        _ if name.starts_with("bounce(") => {
            let height = extract_param(name, "bounce")?;
            Some(CurveType::Bounce(height))
        }
        _ if name.starts_with("elastic(") => {
            let intensity = extract_param(name, "elastic")?;
            Some(CurveType::Elastic(intensity))
        }
        _ if name.starts_with("step(") => {
            let steps = extract_param(name, "step")?;
            Some(CurveType::StepFunction(steps))
        }

        _ => None,
    }
}

fn parse_ease_name(name: &str) -> Option<CurveType> {
    match name {
        "linear" => Some(CurveType::Linear),
        "in" => Some(CurveType::EaseIn),
        "out" => Some(CurveType::EaseOut),
        "inOut" => Some(CurveType::EaseInOut),

        // Bezier curve: bezier(x1, y1, x2, y2)
        _ if name.starts_with("bezier(") => extract_bezier_params(name),

        _ => None,
    }
}

/// Extract single parameter from function string
/// Example: "swing(0.5)" -> 0.5
fn extract_param(name: &str, func_name: &str) -> Option<f32> {
    let start = format!("{}(", func_name);
    let start_idx = name.find(&start)? + start.len();
    let end_idx = name.rfind(')')?;
    let content = &name[start_idx..end_idx];
    content.trim().parse::<f32>().ok()
}

/// Extract bezier parameters: bezier(x1, y1, x2, y2)
fn extract_bezier_params(name: &str) -> Option<CurveType> {
    let start_idx = name.find('(')? + 1;
    let end_idx = name.rfind(')')?;
    let content = &name[start_idx..end_idx];

    let parts: Vec<f32> = content
        .split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
        .collect();

    if parts.len() == 4 {
        Some(CurveType::Bezier(parts[0], parts[1], parts[2], parts[3]))
    } else {
        None
    }
}

/// Helper trait for lerp
trait Lerp {
    fn lerp(&self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, other: f32, t: f32) -> f32 {
        self + (other - self) * t
    }
}

/// Get a curve as a Value (for use with automation)
pub fn curve_to_value(curve: CurveType) -> Value {
    // Store curve as a string representation for now
    // Later could use a custom Value variant
    let repr = match curve {
        CurveType::Linear => "$curve.linear".to_string(),
        CurveType::EaseIn => "$curve.in".to_string(),
        CurveType::EaseOut => "$curve.out".to_string(),
        CurveType::EaseInOut => "$curve.inOut".to_string(),
        CurveType::Swing(i) => format!("$curve.swing({})", i),
        CurveType::Bounce(h) => format!("$curve.bounce({})", h),
        CurveType::Elastic(i) => format!("$curve.elastic({})", i),
        CurveType::Random => "$curve.random".to_string(),
        CurveType::Perlin => "$curve.perlin".to_string(),
        CurveType::Bezier(x1, y1, x2, y2) => {
            format!("$ease.bezier({}, {}, {}, {})", x1, y1, x2, y2)
        }
        CurveType::StepFunction(s) => format!("$curve.step({})", s),
    };

    Value::String(repr)
}

#[cfg(test)]
#[path = "test_curves.rs"]
mod tests;
