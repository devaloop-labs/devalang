/// Effect chain module - sequential processing of multiple effects
use super::registry::{CloneableEffect, EffectRegistry};
use crate::engine::audio::lfo::{LfoParams, LfoRate, LfoTarget, LfoWaveform};
use crate::language::syntax::ast::Value;
use std::collections::HashMap;

/// Effect chain - processes audio through multiple effects in sequence
#[derive(Debug)]
pub struct EffectChain {
    effects: Vec<Box<dyn CloneableEffect>>,
    registry: EffectRegistry,
    synth_context: bool,
}

impl EffectChain {
    /// Create a new effect chain with context
    pub fn new(synth_context: bool) -> Self {
        Self {
            effects: Vec::new(),
            registry: EffectRegistry::new(),
            synth_context,
        }
    }

    /// Add an effect to the chain if available in current context
    pub fn add_effect(&mut self, name: &str, params: Option<Value>) -> bool {
        if !self.registry.is_effect_available(name, self.synth_context) {
            return false;
        }

        if let Some(processor) =
            build_effect_processor(&self.registry, name, params, self.synth_context)
        {
            self.effects.push(processor);
            true
        } else {
            false
        }
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

    /// Get all available effects for the current context
    pub fn available_effects(&self) -> Vec<&'static str> {
        self.registry.list_available_effects(self.synth_context)
    }

    /// Check if an effect is available in current context
    pub fn is_effect_available(&self, name: &str) -> bool {
        self.registry.is_effect_available(name, self.synth_context)
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new(true) // Default to synth context
    }
}

/// Build an effect chain from a Value::Array of effect definitions
pub fn build_effect_chain(effects_array: &[Value], synth_context: bool) -> EffectChain {
    let mut chain = EffectChain::new(synth_context);

    for effect_value in effects_array {
        match effect_value {
            Value::Map(map) => {
                // Get effect type
                if let Some(effect_type) = map.get("type").or_else(|| map.get("effect")) {
                    if let Value::String(s) | Value::Identifier(s) = effect_type {
                        chain.add_effect(s, Some(Value::Map(map.clone())));
                    }
                } else {
                    // Fallback: map may be of the form { "reverb": { ... } } (no explicit "type" key)
                    // or { "reverb": value } - in that case, expand each key as an effect entry.
                    for (k, v) in map.iter() {
                        match v {
                            Value::Map(sm) => {
                                chain.add_effect(k, Some(Value::Map(sm.clone())));
                            }
                            _ => {
                                // non-map value, wrap as { "value": v }
                                let mut sub = std::collections::HashMap::new();
                                sub.insert("value".to_string(), v.clone());
                                chain.add_effect(k, Some(Value::Map(sub.clone())));
                            }
                        }
                    }
                }
            }
            Value::String(s) | Value::Identifier(s) => {
                chain.add_effect(s, None);
            }
            _ => {}
        }
    }

    chain
}

/// Build a single effect processor from name and optional parameters
fn build_effect_processor(
    registry: &EffectRegistry,
    name: &str,
    params: Option<Value>,
    synth_context: bool,
) -> Option<Box<dyn CloneableEffect>> {
    let base_processor = registry.get_effect(name, synth_context)?;

    if let Some(Value::Map(params_map)) = params {
        match name {
            "chorus" => {
                let depth = get_f32_param(&params_map, "depth", 0.7);
                let rate = get_f32_param(&params_map, "rate", 0.5);
                let mix = get_f32_param(&params_map, "mix", 0.5);
                Some(Box::new(super::processors::ChorusProcessor::new(
                    depth, rate, mix,
                )))
            }
            "flanger" => {
                let depth = get_f32_param(&params_map, "depth", 0.7);
                let rate = get_f32_param(&params_map, "rate", 0.5);
                let feedback = get_f32_param(&params_map, "feedback", 0.5);
                let mix = get_f32_param(&params_map, "mix", 0.5);
                Some(Box::new(super::processors::FlangerProcessor::new(
                    depth, rate, feedback, mix,
                )))
            }
            "phaser" => {
                let stages = get_f32_param(&params_map, "stages", 4.0) as usize;
                let rate = get_f32_param(&params_map, "rate", 0.5);
                let depth = get_f32_param(&params_map, "depth", 0.7);
                let feedback = get_f32_param(&params_map, "feedback", 0.5);
                let mix = get_f32_param(&params_map, "mix", 0.5);
                Some(Box::new(super::processors::PhaserProcessor::new(
                    stages, rate, depth, feedback, mix,
                )))
            }
            "compressor" => {
                let threshold = get_f32_param(&params_map, "threshold", -20.0);
                let ratio = get_f32_param(&params_map, "ratio", 4.0);
                let attack = get_f32_param(&params_map, "attack", 0.005);
                let release = get_f32_param(&params_map, "release", 0.1);
                Some(Box::new(super::processors::CompressorProcessor::new(
                    threshold, ratio, attack, release,
                )))
            }
            "drive" => {
                let amount = get_f32_param(&params_map, "amount", 0.7);
                let mix = get_f32_param(&params_map, "mix", 0.5);
                let tone = get_f32_param(&params_map, "tone", 0.5);
                let color = get_f32_param(&params_map, "color", 0.5);

                Some(Box::new(super::processors::DriveProcessor::new(
                    amount, tone, color, mix,
                )))
            }
            "reverb" => {
                let size = get_f32_param(&params_map, "size", 0.5);
                let damping = get_f32_param(&params_map, "damping", 0.5);
                let decay = get_f32_param(&params_map, "decay", 0.5);
                let mix = get_f32_param(&params_map, "mix", 0.3);

                Some(Box::new(super::processors::ReverbProcessor::new(
                    size, damping, decay, mix,
                )))
            }
            "delay" => {
                let time = get_f32_param(&params_map, "time", 250.0);
                let feedback = get_f32_param(&params_map, "feedback", 0.4);
                let mix = get_f32_param(&params_map, "mix", 0.3);
                Some(Box::new(super::processors::DelayProcessor::new(
                    time, feedback, mix,
                )))
            }
            "speed" => {
                // support both {'speed': 2.0} and {'value': 2.0} normalized forms
                let speed = get_f32_param(
                    &params_map,
                    "speed",
                    get_f32_param(&params_map, "value", 1.0),
                );

                Some(Box::new(super::processors::SpeedProcessor::new(speed)))
            }
            "lfo" => {
                // parse LFO params: rate, depth, waveform, target, phase
                // rate may be a number or string like "1/8"
                let bpm = get_f32_param(&params_map, "bpm", 120.0);
                let depth = get_f32_param(&params_map, "depth", 0.5).clamp(0.0, 1.0);
                let phase = get_f32_param(&params_map, "phase", 0.0).fract();

                // waveform
                let waveform = params_map
                    .get("waveform")
                    .and_then(|v| match v {
                        Value::String(s) | Value::Identifier(s) => Some(LfoWaveform::from_str(s)),
                        _ => None,
                    })
                    .unwrap_or(LfoWaveform::Sine);

                // rate
                let rate_value_opt = params_map.get("rate");
                let rate = match rate_value_opt {
                    Some(Value::String(s)) | Some(Value::Identifier(s)) => LfoRate::from_value(s),
                    Some(Value::Number(n)) => LfoRate::Hz(*n),
                    _ => LfoRate::Hz(1.0),
                };

                // target
                let target = params_map
                    .get("target")
                    .and_then(|v| match v {
                        Value::String(s) | Value::Identifier(s) => LfoTarget::from_str(s),
                        _ => None,
                    })
                    .unwrap_or(LfoTarget::Volume);

                let params = LfoParams {
                    rate,
                    depth,
                    waveform,
                    target,
                    phase,
                };
                // parse optional ranges
                let semitones = get_f32_param(&params_map, "semitones", 2.0);
                let base_cutoff = get_f32_param(&params_map, "cutoff", 1000.0);
                let cutoff_range = get_f32_param(&params_map, "cutoff_range", 1000.0);
                Some(Box::new(super::processors::LfoProcessor::new(
                    params,
                    bpm,
                    semitones,
                    base_cutoff,
                    cutoff_range,
                )))
            }
            "reverse" => {
                let reverse = get_bool_param(
                    &params_map,
                    "reverse",
                    get_bool_param(&params_map, "value", true),
                );
                Some(Box::new(super::processors::ReverseProcessor::new(reverse)))
            }
            _ => Some(base_processor),
        }
    } else {
        Some(base_processor)
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

/// Helper to extract bool parameter from map
fn get_bool_param(map: &HashMap<String, Value>, key: &str, default: bool) -> bool {
    map.get(key)
        .and_then(|v| match v {
            Value::Boolean(b) => Some(*b),
            Value::Number(n) => Some(*n != 0.0),
            Value::String(s) => s.parse::<bool>().ok(),
            _ => None,
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_chain_creation() {
        let chain = EffectChain::new(true); // Synth context
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());

        // Test available effects in synth context
        let synth_effects = chain.available_effects();
        assert!(synth_effects.contains(&"reverb"));
        assert!(!synth_effects.contains(&"speed")); // Trigger-only effect
    }

    #[test]
    fn test_effect_chain_trigger_context() {
        let mut chain = EffectChain::new(false); // Trigger context

        // Test trigger-specific effects
        assert!(chain.is_effect_available("speed"));
        assert!(chain.is_effect_available("reverse"));

        // Test adding trigger-specific effect
        assert!(chain.add_effect("speed", None));
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_effect_parameter_parsing() {
        let mut chain = EffectChain::new(true);

        // Test adding effect with parameters
        let mut params = HashMap::new();
        params.insert("depth".to_string(), Value::Number(0.8));
        params.insert("rate".to_string(), Value::Number(1.0));
        params.insert("mix".to_string(), Value::Number(0.6));

        assert!(chain.add_effect("chorus", Some(Value::Map(params))));
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_effect_context_restrictions() {
        let mut synth_chain = EffectChain::new(true);
        let mut trigger_chain = EffectChain::new(false);

        // Speed effect should only work in trigger context
        assert!(!synth_chain.add_effect("speed", None));
        assert!(trigger_chain.add_effect("speed", None));

        // Reverb effect should work in both contexts
        assert!(synth_chain.add_effect("reverb", None));
        assert!(trigger_chain.add_effect("reverb", None));
    }
}
