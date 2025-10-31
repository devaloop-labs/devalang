/// Audio effects module - Processor and effect type management
pub mod chain;
pub mod processors;
pub mod registry;

use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Effect availability enum - defines where an effect can be used
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectAvailability {
    SynthOnly,
    TriggerOnly,
    Both,
}

/// Effect parameters extracted from a map
#[derive(Debug, Clone)]
pub struct EffectParams {
    pub gain: f32,
    pub pan: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub pitch: f32,
    pub drive: DriveParams,
    pub reverb: ReverbParams,
    pub delay: DelayParams,
    pub speed: f32,
    pub reverse: bool,
}

#[derive(Debug, Clone)]
pub struct DriveParams {
    pub amount: f32,
    pub mix: f32,
    pub tone: f32,
    pub color: f32,
}

#[derive(Debug, Clone)]
pub struct ReverbParams {
    pub size: f32,
    pub mix: f32,
    pub decay: f32,
    pub damping: f32,
}

#[derive(Debug, Clone)]
pub struct DelayParams {
    pub time: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            fade_in: 0.0,
            fade_out: 0.0,
            pitch: 1.0,
            drive: DriveParams {
                amount: 0.7,
                mix: 0.5,
                tone: 0.5,
                color: 0.5,
            },
            reverb: ReverbParams {
                size: 0.5,
                mix: 0.3,
                decay: 0.5,
                damping: 0.5,
            },
            delay: DelayParams {
                time: 250.0,
                feedback: 0.4,
                mix: 0.3,
            },
            speed: 1.0,
            reverse: false,
        }
    }
}

/// Extract effect parameters from a Value map
pub fn extract_effect_params(effects: &Option<Value>) -> EffectParams {
    let mut params = EffectParams::default();

    if let Some(Value::Map(map)) = effects {
        params.gain = param_as_f32(map, &["gain", "volume", "vol"], 1.0);
        params.pan = param_as_f32(map, &["pan", "panning"], 0.0);
        params.fade_in = param_as_f32(map, &["fadeIn", "fadein", "fade_in"], 0.0);
        params.fade_out = param_as_f32(map, &["fadeOut", "fadeout", "fade_out"], 0.0);
        params.pitch = param_as_f32(map, &["pitch", "tune"], 1.0);
        params.speed = param_as_f32(map, &["speed", "rate"], 1.0);
        params.reverse = param_as_bool(map, &["reverse", "rev"], false);

        // Extract drive parameters
        if let Some(Value::Map(drive_map)) = map.get("drive") {
            params.drive.amount = param_as_f32(drive_map, &["amount", "drive"], 0.7);
            params.drive.mix = param_as_f32(drive_map, &["mix", "wet"], 0.5);
            params.drive.tone = param_as_f32(drive_map, &["tone"], 0.5);
            params.drive.color = param_as_f32(drive_map, &["color", "tone"], 0.5);
        }

        // Extract reverb parameters
        if let Some(Value::Map(reverb_map)) = map.get("reverb") {
            params.reverb.size = param_as_f32(reverb_map, &["size", "room"], 0.5);
            params.reverb.mix = param_as_f32(reverb_map, &["mix", "wet"], 0.3);
            params.reverb.damping = param_as_f32(reverb_map, &["damping", "damp"], 0.5);
            params.reverb.decay = param_as_f32(reverb_map, &["decay", "tail"], 0.5);
        }

        // Extract delay parameters
        if let Some(Value::Map(delay_map)) = map.get("delay") {
            params.delay.time = param_as_f32(delay_map, &["time", "length"], 250.0);
            params.delay.feedback = param_as_f32(delay_map, &["feedback", "fb"], 0.4);
            params.delay.mix = param_as_f32(delay_map, &["mix", "wet"], 0.3);
        }
    }

    params
}

/// Read a numeric parameter from a map with multiple possible key names
pub fn param_as_f32(params: &HashMap<String, Value>, names: &[&str], default: f32) -> f32 {
    for name in names.iter() {
        if let Some(v) = params.get(*name) {
            match v {
                Value::Number(n) => return *n,
                Value::String(s) | Value::Identifier(s) => {
                    if let Ok(parsed) = s.parse::<f32>() {
                        return parsed;
                    }
                }
                Value::Boolean(b) => return if *b { 1.0 } else { 0.0 },
                _ => {}
            }
        }
    }
    default
}

/// Read a boolean parameter from a map with multiple possible key names
pub fn param_as_bool(params: &HashMap<String, Value>, names: &[&str], default: bool) -> bool {
    for name in names.iter() {
        if let Some(v) = params.get(*name) {
            match v {
                Value::Boolean(b) => return *b,
                Value::Number(n) => return *n != 0.0,
                Value::String(s) | Value::Identifier(s) => {
                    if let Ok(parsed) = s.parse::<bool>() {
                        return parsed;
                    }
                }
                _ => {}
            }
        }
    }
    default
}

/// Normalize effects map into per-effect parameter structure
pub fn normalize_effects(effects: &Option<Value>) -> HashMap<String, HashMap<String, Value>> {
    let mut out: HashMap<String, HashMap<String, Value>> = HashMap::new();

    if let Some(Value::Map(map)) = effects {
        // DEPRECATION: legacy map-style effect parameters are deprecated.
        // Prefer chained params (-> effect(...)) in scripts. Keep compatibility for now but warn.
        eprintln!(
            "DEPRECATION: effect param map support is deprecated — use chained params instead. This will be removed in a future version."
        );
        for (k, v) in map.iter() {
            match v {
                Value::Number(_) | Value::Boolean(_) | Value::String(_) | Value::Identifier(_) => {
                    let mut sub = HashMap::new();
                    sub.insert("value".to_string(), v.clone());
                    out.insert(k.clone(), sub);
                }
                Value::Map(sm) => {
                    out.insert(k.clone(), sm.clone());
                }
                _ => {}
            }
        }
    }

    out
}

/// Merge an effect entry into a flat effects map
pub fn merge_effect_entry(map: &mut HashMap<String, Value>, key: &str, val: &Value) {
    match val {
        Value::Number(_) | Value::Boolean(_) | Value::String(_) | Value::Identifier(_) => {
            map.insert(key.to_string(), val.clone());
        }
        Value::Map(m) => {
            map.insert(key.to_string(), Value::Map(m.clone()));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_effect_params_defaults() {
        let params = extract_effect_params(&None);
        assert!((params.gain - 1.0).abs() < f32::EPSILON);
        assert!((params.pan - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_extract_effect_params_with_values() {
        let mut map = HashMap::new();
        map.insert("gain".to_string(), Value::Number(0.5));
        map.insert("pan".to_string(), Value::Number(-0.3));

        let params = extract_effect_params(&Some(Value::Map(map)));
        assert!((params.gain - 0.5).abs() < f32::EPSILON);
        assert!((params.pan + 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn test_param_as_f32_with_aliases() {
        let mut map = HashMap::new();
        map.insert("volume".to_string(), Value::Number(0.8));

        let result = param_as_f32(&map, &["gain", "volume", "vol"], 1.0);
        assert!((result - 0.8).abs() < f32::EPSILON);
    }
}
