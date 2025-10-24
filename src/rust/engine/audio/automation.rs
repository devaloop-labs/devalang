use crate::language::syntax::ast::Value;
/// Automation system - parameter automation over time
/// Supports linear, exponential, and custom curves
use std::collections::HashMap;

/// Automation curve type (legacy simple curves)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationCurve {
    Linear,
    Exponential,
    Logarithmic,
    Smooth, // Smooth interpolation (ease-in-out)
}

impl AutomationCurve {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "linear" | "lin" => AutomationCurve::Linear,
            "exponential" | "exp" => AutomationCurve::Exponential,
            "logarithmic" | "log" => AutomationCurve::Logarithmic,
            "smooth" | "ease" => AutomationCurve::Smooth,
            _ => AutomationCurve::Linear,
        }
    }
}

/// Automation parameter
#[derive(Debug, Clone)]
pub struct AutomationParam {
    pub param_name: String,
    pub from_value: f32,
    pub to_value: f32,
    pub start_time: f32, // seconds
    pub duration: f32,   // seconds
    pub curve: AutomationCurve,
}

/// Lightweight template for per-note automation (percent-based points)
#[derive(Debug, Clone)]
pub struct AutomationParamTemplate {
    pub param_name: String,
    /// Points as (progress_fraction 0.0-1.0, value)
    pub points: Vec<(f32, f32)>,
    pub curve: AutomationCurve,
    /// Advanced curve (if specified)
    pub advanced_curve: Option<crate::engine::curves::CurveType>,
}

/// Automation envelope - collection of automation parameters
#[derive(Debug, Clone)]
pub struct AutomationEnvelope {
    pub target: String, // Target entity (synth name, "global", etc.)
    pub params: Vec<AutomationParam>,
}

impl AutomationEnvelope {
    pub fn new(target: String) -> Self {
        Self {
            target,
            params: Vec::new(),
        }
    }

    /// Add automation parameter
    pub fn add_param(&mut self, param: AutomationParam) {
        self.params.push(param);
    }

    /// Get automated value for a parameter at a specific time
    pub fn get_value(&self, param_name: &str, time_seconds: f32) -> Option<f32> {
        // Find all automation params for this parameter name
        let matching: Vec<&AutomationParam> = self
            .params
            .iter()
            .filter(|p| p.param_name == param_name)
            .collect();

        if matching.is_empty() {
            return None;
        }

        // Find the active automation (most recent one that affects current time)
        for param in matching.iter().rev() {
            let end_time = param.start_time + param.duration;

            if time_seconds >= param.start_time && time_seconds <= end_time {
                // Currently in automation range
                let progress = (time_seconds - param.start_time) / param.duration;
                let value =
                    interpolate_value(param.from_value, param.to_value, progress, param.curve);
                return Some(value);
            } else if time_seconds > end_time {
                // Past automation - return end value
                return Some(param.to_value);
            }
        }

        // Before any automation - return first start value
        Some(matching[0].from_value)
    }
}

/// Interpolate between two values based on progress and curve type
fn interpolate_value(from: f32, to: f32, progress: f32, curve: AutomationCurve) -> f32 {
    let t = progress.clamp(0.0, 1.0);

    let interpolated = match curve {
        AutomationCurve::Linear => t,
        AutomationCurve::Exponential => {
            // Exponential curve (ease-out)
            if (to - from).abs() < 0.0001 { t } else { t * t }
        }
        AutomationCurve::Logarithmic => {
            // Logarithmic curve (ease-in)
            1.0 - (1.0 - t) * (1.0 - t)
        }
        AutomationCurve::Smooth => {
            // Smooth ease-in-out (cubic)
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
    };

    from + (to - from) * interpolated
}

/// Parse automation from Value::Map
pub fn parse_automation_from_value(value: &Value) -> Option<AutomationEnvelope> {
    if let Value::Map(map) = value {
        let target = map.get("target").and_then(|v| match v {
            Value::String(s) | Value::Identifier(s) => Some(s.clone()),
            _ => None,
        })?;

        let mut envelope = AutomationEnvelope::new(target);

        // Parse params array
        if let Some(Value::Array(params_array)) = map.get("params") {
            for param_value in params_array {
                if let Some(param) = parse_automation_param(param_value) {
                    envelope.add_param(param);
                }
            }
        }

        Some(envelope)
    } else {
        None
    }
}

/// Parse per-param templates from a raw automate body string.
/// Expects blocks like: param <name> { 0% = 0.0 100% = 1.0 }
pub fn parse_param_templates_from_raw(raw: &str) -> Vec<AutomationParamTemplate> {
    use regex::Regex;
    let mut templates = Vec::new();

    // Find param blocks: param <name> [curve <curveName>] { ... }
    let re_block =
        Regex::new(r"param\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:curve\s+([^\s{]+)\s*)?\{([^}]*)\}")
            .unwrap();
    let re_point = Regex::new(r"([0-9]+(?:\.[0-9]+)?)%?\s*=\s*([\-0-9\.eE]+)").unwrap();

    for cap in re_block.captures_iter(raw) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let curve_str = cap.get(2).map(|m| m.as_str());
        let body = cap.get(3).unwrap().as_str();

        let mut points: Vec<(f32, f32)> = Vec::new();
        for pcap in re_point.captures_iter(body) {
            if let (Some(p_str), Some(v_str)) = (pcap.get(1), pcap.get(2)) {
                if let (Ok(pv), Ok(vv)) =
                    (p_str.as_str().parse::<f32>(), v_str.as_str().parse::<f32>())
                {
                    let frac = (pv / 100.0).clamp(0.0, 1.0);
                    points.push((frac, vv));
                }
            }
        }

        // Sort by progress fraction
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        if !points.is_empty() {
            // Parse advanced curve if specified
            let advanced_curve = curve_str.and_then(|s| crate::engine::curves::parse_curve(s));

            templates.push(AutomationParamTemplate {
                param_name: name,
                points,
                curve: AutomationCurve::Linear,
                advanced_curve,
            });
        }
    }

    templates
}

/// Evaluate a template at a given progress fraction (0.0..1.0)
pub fn evaluate_template_at(tpl: &AutomationParamTemplate, progress: f32) -> f32 {
    let p = progress.clamp(0.0, 1.0);
    if tpl.points.is_empty() {
        return 0.0;
    }
    // Before first point
    if p <= tpl.points[0].0 {
        return tpl.points[0].1;
    }
    // After last point
    if p >= tpl.points.last().unwrap().0 {
        return tpl.points.last().unwrap().1;
    }

    // Find segment
    for w in tpl.points.windows(2) {
        let (p0, v0) = w[0];
        let (p1, v1) = w[1];
        if p >= p0 && p <= p1 {
            let local = if (p1 - p0).abs() < f32::EPSILON {
                0.0
            } else {
                (p - p0) / (p1 - p0)
            };

            // Apply advanced curve if specified
            let eased_local = if let Some(curve) = &tpl.advanced_curve {
                crate::engine::curves::evaluate_curve(*curve, local)
            } else {
                local // Standard linear interpolation
            };

            return v0 + (v1 - v0) * eased_local;
        }
    }

    // Fallback
    tpl.points.last().unwrap().1
}

/// Parse single automation parameter from Value
fn parse_automation_param(value: &Value) -> Option<AutomationParam> {
    if let Value::Map(map) = value {
        let param_name = map.get("name").and_then(|v| match v {
            Value::String(s) | Value::Identifier(s) => Some(s.clone()),
            _ => None,
        })?;

        let from_value = map.get("from").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })?;

        let to_value = map.get("to").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })?;

        let start_time = map
            .get("start")
            .and_then(|v| match v {
                Value::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);

        let duration = map.get("duration").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })?;

        let curve = map
            .get("curve")
            .and_then(|v| match v {
                Value::String(s) | Value::Identifier(s) => Some(AutomationCurve::from_str(s)),
                _ => None,
            })
            .unwrap_or(AutomationCurve::Linear);

        Some(AutomationParam {
            param_name,
            from_value,
            to_value,
            start_time,
            duration,
            curve,
        })
    } else {
        None
    }
}

/// Automation registry - stores all active automations
#[derive(Debug, Clone, Default)]
pub struct AutomationRegistry {
    envelopes: HashMap<String, AutomationEnvelope>,
}

impl AutomationRegistry {
    pub fn new() -> Self {
        Self {
            envelopes: HashMap::new(),
        }
    }

    /// Register an automation envelope
    pub fn register(&mut self, envelope: AutomationEnvelope) {
        self.envelopes.insert(envelope.target.clone(), envelope);
    }

    /// Get automated value for a target and parameter at a specific time
    pub fn get_value(&self, target: &str, param_name: &str, time_seconds: f32) -> Option<f32> {
        self.envelopes
            .get(target)
            .and_then(|env| env.get_value(param_name, time_seconds))
    }

    /// Check if a target has any active automations
    pub fn has_automation(&self, target: &str) -> bool {
        self.envelopes.contains_key(target)
    }

    /// Get all automation targets
    pub fn targets(&self) -> Vec<String> {
        self.envelopes.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let param = AutomationParam {
            param_name: "volume".to_string(),
            from_value: 0.0,
            to_value: 1.0,
            start_time: 0.0,
            duration: 2.0,
            curve: AutomationCurve::Linear,
        };

        let mut envelope = AutomationEnvelope::new("synth1".to_string());
        envelope.add_param(param);

        // At t=0, should be 0.0
        assert_eq!(envelope.get_value("volume", 0.0), Some(0.0));

        // At t=1 (50%), should be 0.5
        assert!((envelope.get_value("volume", 1.0).unwrap() - 0.5).abs() < 0.001);

        // At t=2, should be 1.0
        assert_eq!(envelope.get_value("volume", 2.0), Some(1.0));

        // At t=3 (past), should stay at 1.0
        assert_eq!(envelope.get_value("volume", 3.0), Some(1.0));
    }

    #[test]
    fn test_smooth_curve() {
        let value_start = interpolate_value(0.0, 1.0, 0.0, AutomationCurve::Smooth);
        let value_mid = interpolate_value(0.0, 1.0, 0.5, AutomationCurve::Smooth);
        let value_end = interpolate_value(0.0, 1.0, 1.0, AutomationCurve::Smooth);

        assert_eq!(value_start, 0.0);
        assert_eq!(value_end, 1.0);
        assert!(value_mid > 0.4 && value_mid < 0.6); // Should be around 0.5
    }

    #[test]
    fn test_automation_registry() {
        let mut registry = AutomationRegistry::new();

        let mut envelope = AutomationEnvelope::new("synth1".to_string());
        envelope.add_param(AutomationParam {
            param_name: "volume".to_string(),
            from_value: 0.0,
            to_value: 1.0,
            start_time: 0.0,
            duration: 2.0,
            curve: AutomationCurve::Linear,
        });

        registry.register(envelope);

        assert!(registry.has_automation("synth1"));
        assert!(!registry.has_automation("synth2"));

        let value = registry.get_value("synth1", "volume", 1.0);
        assert!(value.is_some());
        assert!((value.unwrap() - 0.5).abs() < 0.001);
    }
}
