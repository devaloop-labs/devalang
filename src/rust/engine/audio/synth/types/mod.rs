pub mod arp;
pub mod bass;
pub mod keys;
pub mod lead;
pub mod pad;
/// Synth types module - different synth presets and behaviors
pub mod pluck;

use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;

/// Trait for synth type behavior
pub trait SynthType {
    /// Get the name of this synth type
    fn name(&self) -> &str;

    /// Modify synth parameters based on type
    fn modify_params(&self, params: &mut SynthParams);

    /// Apply post-processing to generated samples
    /// options: configurable parameters from synth definition
    fn post_process(
        &self,
        samples: &mut [f32],
        sample_rate: u32,
        options: &HashMap<String, f32>,
    ) -> Result<()>;
}

/// Get synth type by name
pub fn get_synth_type(type_name: &str) -> Option<Box<dyn SynthType>> {
    match type_name.to_lowercase().as_str() {
        "pluck" => Some(Box::new(pluck::PluckSynth)),
        "arp" => Some(Box::new(arp::ArpSynth)),
        "pad" => Some(Box::new(pad::PadSynth)),
        "bass" => Some(Box::new(bass::BassSynth)),
        "lead" => Some(Box::new(lead::LeadSynth)),
        "keys" => Some(Box::new(keys::KeysSynth)),
        _ => None,
    }
}
