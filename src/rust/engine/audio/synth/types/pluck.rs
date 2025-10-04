/// Pluck synth - short percussive sound like a plucked string or water droplet
/// Characteristics:
/// - Very short attack (instant)
/// - Quick decay
/// - No sustain
/// - Short release
/// - Bright, clean sound
use super::SynthType;
use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;

pub struct PluckSynth;

impl SynthType for PluckSynth {
    fn name(&self) -> &str {
        "pluck"
    }

    fn modify_params(&self, params: &mut SynthParams) {
        // Pluck envelope: instant attack, quick decay, no sustain
        params.attack = 0.001; // 1ms - almost instant
        params.decay = 0.15; // 150ms - quick decay
        params.sustain = 0.0; // No sustain - sound dies out
        params.release = 0.05; // 50ms - short release
    }

    fn post_process(
        &self,
        samples: &mut [f32],
        _sample_rate: u32,
        _options: &HashMap<String, f32>,
    ) -> Result<()> {
        // Add brightness by emphasizing transients
        // Simple high-pass effect to make it more "plucky"
        if samples.len() < 4 {
            return Ok(());
        }

        let mut prev = 0.0f32;
        for sample in samples.iter_mut() {
            let current = *sample;
            // High-pass filter (emphasize changes)
            *sample = current - 0.3 * prev;
            prev = current;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluck_params() {
        let pluck = PluckSynth;
        let mut params = SynthParams::default();

        pluck.modify_params(&mut params);

        assert!(params.attack < 0.01); // Very short attack
        assert_eq!(params.sustain, 0.0); // No sustain
    }
}
