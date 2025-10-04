/// Effect chain module - sequential processing of multiple effects
use super::processors::{
    ChorusProcessor, CompressorProcessor, DistortionProcessor, EffectProcessor, FlangerProcessor,
    PhaserProcessor,
};
use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Effect chain - processes audio through multiple effects in sequence
#[derive(Debug)]
pub struct EffectChain {
    effects: Vec<Box<dyn EffectProcessor>>,
}

impl EffectChain {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add an effect to the chain
    pub fn add_effect(&mut self, effect: Box<dyn EffectProcessor>) {
        self.effects.push(effect);
    }

    /// Process audio samples through all effects in the chain
    pub fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        for effect in &mut self.effects {
            effect.process(samples, sample_rate);
        }
    }

    /// Reset all effects in the chain
    pub fn reset(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }

    /// Get number of effects in the chain
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Build an effect chain from a Value::Array of effect definitions
pub fn build_effect_chain(effects_array: &[Value]) -> EffectChain {
    let mut chain = EffectChain::new();

    for effect_value in effects_array {
        if let Some(processor) = build_effect_processor(effect_value) {
            chain.add_effect(processor);
        }
    }

    chain
}

/// Build a single effect processor from a Value definition
fn build_effect_processor(value: &Value) -> Option<Box<dyn EffectProcessor>> {
    match value {
        Value::Map(map) => {
            // Get effect type
            let effect_type =
                map.get("type")
                    .or_else(|| map.get("effect"))
                    .and_then(|v| match v {
                        Value::String(s) | Value::Identifier(s) => Some(s.clone()),
                        _ => None,
                    })?;

            match effect_type.to_lowercase().as_str() {
                "chorus" => {
                    let depth = get_f32_param(map, "depth", 0.7);
                    let rate = get_f32_param(map, "rate", 0.5);
                    let mix = get_f32_param(map, "mix", 0.5);
                    Some(Box::new(ChorusProcessor::new(depth, rate, mix)))
                }
                "flanger" => {
                    let depth = get_f32_param(map, "depth", 0.7);
                    let rate = get_f32_param(map, "rate", 0.5);
                    let feedback = get_f32_param(map, "feedback", 0.5);
                    let mix = get_f32_param(map, "mix", 0.5);
                    Some(Box::new(FlangerProcessor::new(depth, rate, feedback, mix)))
                }
                "phaser" => {
                    let stages = get_f32_param(map, "stages", 4.0) as usize;
                    let rate = get_f32_param(map, "rate", 0.5);
                    let depth = get_f32_param(map, "depth", 0.7);
                    let feedback = get_f32_param(map, "feedback", 0.5);
                    let mix = get_f32_param(map, "mix", 0.5);
                    Some(Box::new(PhaserProcessor::new(
                        stages, rate, depth, feedback, mix,
                    )))
                }
                "compressor" => {
                    let threshold = get_f32_param(map, "threshold", -20.0);
                    let ratio = get_f32_param(map, "ratio", 4.0);
                    let attack = get_f32_param(map, "attack", 0.005);
                    let release = get_f32_param(map, "release", 0.1);
                    Some(Box::new(CompressorProcessor::new(
                        threshold, ratio, attack, release,
                    )))
                }
                "distortion" => {
                    let drive = get_f32_param(map, "drive", 10.0);
                    let mix = get_f32_param(map, "mix", 0.5);
                    Some(Box::new(DistortionProcessor::new(drive, mix)))
                }
                _ => None,
            }
        }
        Value::String(s) | Value::Identifier(s) => {
            // Simple effect name without parameters - use defaults
            match s.to_lowercase().as_str() {
                "chorus" => Some(Box::new(ChorusProcessor::default())),
                "flanger" => Some(Box::new(FlangerProcessor::default())),
                "phaser" => Some(Box::new(PhaserProcessor::default())),
                "compressor" => Some(Box::new(CompressorProcessor::default())),
                "distortion" => Some(Box::new(DistortionProcessor::default())),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Helper to extract f32 parameter from map
fn get_f32_param(map: &HashMap<String, Value>, key: &str, default: f32) -> f32 {
    map.get(key)
        .and_then(|v| match v {
            Value::Number(n) => Some(*n),
            Value::String(s) => s.parse::<f32>().ok(),
            _ => None,
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_chain_creation() {
        let mut chain = EffectChain::new();
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());

        chain.add_effect(Box::new(ChorusProcessor::default()));
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_build_effect_from_string() {
        let value = Value::String("chorus".to_string());
        let processor = build_effect_processor(&value);
        assert!(processor.is_some());
    }

    #[test]
    fn test_build_effect_from_map() {
        let mut map = HashMap::new();
        map.insert("type".to_string(), Value::String("chorus".to_string()));
        map.insert("depth".to_string(), Value::Number(0.8));
        map.insert("rate".to_string(), Value::Number(1.0));

        let value = Value::Map(map);
        let processor = build_effect_processor(&value);
        assert!(processor.is_some());
    }
}
