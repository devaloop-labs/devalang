/// Audio effects module - parameter extraction and normalization
pub mod chain;
pub mod processors;

use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Effect parameters extracted from a map
#[derive(Debug, Clone)]
pub struct EffectParams {
    pub gain: f32,
    pub pan: f32,
    pub fade_in: f32,
    pub fade_out: f32,
    pub pitch: f32,
    pub drive: f32,
    pub reverb: f32,
    pub delay: f32,
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            fade_in: 0.0,
            fade_out: 0.0,
            pitch: 1.0,
            drive: 0.0,
            reverb: 0.0,
            delay: 0.0,
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
        params.drive = param_as_f32(map, &["drive", "distortion"], 0.0);
        params.reverb = param_as_f32(map, &["reverb", "verb"], 0.0);
        params.delay = param_as_f32(map, &["delay", "echo"], 0.0);
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

/// Normalize effects map into per-effect parameter structure
pub fn normalize_effects(effects: &Option<Value>) -> HashMap<String, HashMap<String, Value>> {
    let mut out: HashMap<String, HashMap<String, Value>> = HashMap::new();

    if let Some(Value::Map(map)) = effects {
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
