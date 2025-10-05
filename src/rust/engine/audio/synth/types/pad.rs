/// Pad synth - lush, continuous, ambient sound
/// Characteristics:
/// - Slow attack (gradual swell)
/// - Long decay
/// - High sustain level
/// - Long release (lingering tail)
/// - Smooth, dreamy sound with natural reverb-like quality
use super::SynthType;
use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;

pub struct PadSynth;

impl SynthType for PadSynth {
    fn name(&self) -> &str {
        "pad"
    }

    fn modify_params(&self, params: &mut SynthParams) {
        // Pad envelope: slow attack, long decay, high sustain, long release
        params.attack = 0.3; // 300ms - slow swell
        params.decay = 0.4; // 400ms - gentle decay
        params.sustain = 0.85; // High sustain - stays full
        params.release = 0.8; // 800ms - long tail
    }

    fn post_process(
        &self,
        samples: &mut [f32],
        sample_rate: u32,
        _options: &HashMap<String, f32>,
    ) -> Result<()> {
        // Add subtle chorus/widening effect for pad thickness
        // Simple delay-based chorus
        let delay_samples = (sample_rate as f32 * 0.015) as usize; // 15ms delay

        if samples.len() <= delay_samples * 2 {
            return Ok(());
        }

        // Create a delayed copy and mix it in (chorus effect)
        for i in delay_samples..samples.len() - delay_samples {
            let delayed = samples[i - delay_samples];
            samples[i] = samples[i] * 0.7 + delayed * 0.3;
        }

        // Add subtle low-pass filtering for warmth
        let mut prev = 0.0f32;
        for sample in samples.iter_mut() {
            let filtered = *sample * 0.7 + prev * 0.3;
            prev = filtered;
            *sample = filtered;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_params() {
        let pad = PadSynth;
        let mut params = SynthParams::default();

        pad.modify_params(&mut params);

        assert!(params.attack > 0.2); // Slow attack
        assert!(params.sustain > 0.8); // High sustain
        assert!(params.release > 0.5); // Long release
    }
}
