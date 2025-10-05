/// Bass synth - deep, powerful low-frequency sound
/// Characteristics:
/// - Quick attack (punchy)
/// - Moderate decay
/// - High sustain
/// - Short release
/// - Low-end emphasis, powerful sub frequencies
use super::SynthType;
use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;

pub struct BassSynth;

impl SynthType for BassSynth {
    fn name(&self) -> &str {
        "bass"
    }

    fn modify_params(&self, params: &mut SynthParams) {
        // Bass envelope: quick attack for punch, sustained for power
        params.attack = 0.01; // 10ms - punchy attack
        params.decay = 0.15; // 150ms - moderate decay
        params.sustain = 0.75; // High sustain - powerful
        params.release = 0.1; // 100ms - short release

        // Prefer square wave for bass (more harmonics)
        if params.waveform == "sine" {
            params.waveform = "square".to_string();
        }
    }

    fn post_process(
        &self,
        samples: &mut [f32],
        _sample_rate: u32,
        _options: &HashMap<String, f32>,
    ) -> Result<()> {
        // Add sub-harmonic and saturation for bass thickness
        for i in 0..samples.len() {
            let original = samples[i];

            // Soft saturation/compression for punch
            let compressed = if original > 0.0 {
                original.min(0.9)
            } else {
                original.max(-0.9)
            };

            // Add subtle harmonic distortion for thickness
            let distorted = compressed + (compressed * compressed * compressed) * 0.2;

            samples[i] = distorted * 1.1; // Slight boost
        }

        // Low-pass filter to emphasize lows
        let mut prev = 0.0f32;
        for sample in samples.iter_mut() {
            let filtered = *sample * 0.6 + prev * 0.4;
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
    fn test_bass_params() {
        let bass = BassSynth;
        let mut params = SynthParams::default();

        bass.modify_params(&mut params);

        assert!(params.attack < 0.02); // Quick attack
        assert!(params.sustain > 0.7); // High sustain
    }
}
