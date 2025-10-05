/// Lead synth - bright, cutting, melodic sound
/// Characteristics:
/// - Fast attack
/// - Short decay
/// - Moderate-high sustain
/// - Moderate release
/// - Bright, present in the mix, with slight vibrato
use super::SynthType;
use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;
use std::f32::consts::PI;

pub struct LeadSynth;

impl SynthType for LeadSynth {
    fn name(&self) -> &str {
        "lead"
    }

    fn modify_params(&self, params: &mut SynthParams) {
        // Lead envelope: fast attack, moderate sustain
        params.attack = 0.008; // 8ms - fast attack
        params.decay = 0.08; // 80ms - quick decay
        params.sustain = 0.7; // Moderate sustain
        params.release = 0.15; // 150ms - moderate release

        // Prefer saw wave for lead (bright harmonics)
        if params.waveform == "sine" {
            params.waveform = "saw".to_string();
        }
    }

    fn post_process(
        &self,
        samples: &mut [f32],
        sample_rate: u32,
        _options: &HashMap<String, f32>,
    ) -> Result<()> {
        // Add subtle vibrato (pitch modulation) for expressiveness
        let vibrato_rate = 5.0; // 5 Hz vibrato
        let vibrato_depth = 0.005; // Small pitch variation

        let mut modulated = vec![0.0f32; samples.len()];

        for i in 0..samples.len() {
            let time = i as f32 / sample_rate as f32;
            let vibrato = (2.0 * PI * vibrato_rate * time).sin() * vibrato_depth;

            // Calculate source index with vibrato offset
            let source_idx = (i as f32 * (1.0 + vibrato)) as usize;

            if source_idx < samples.len() {
                modulated[i] = samples[source_idx];
            } else {
                modulated[i] = samples[i];
            }
        }

        // Copy modulated samples back
        samples.copy_from_slice(&modulated);

        // Add brightness with slight high-pass filtering
        let mut prev = 0.0f32;
        for sample in samples.iter_mut() {
            let current = *sample;
            *sample = current - 0.2 * prev;
            prev = current;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_params() {
        let lead = LeadSynth;
        let mut params = SynthParams::default();

        lead.modify_params(&mut params);

        assert!(params.attack < 0.01);
        assert_eq!(params.waveform, "saw");
    }
}
