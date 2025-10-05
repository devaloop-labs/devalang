/// Keys synth - piano/keyboard-like sound
/// Characteristics:
/// - Very fast attack (like hitting a key)
/// - Natural decay
/// - Moderate sustain
/// - Natural release
/// - Clean, bell-like tone
use super::SynthType;
use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;

pub struct KeysSynth;

impl SynthType for KeysSynth {
    fn name(&self) -> &str {
        "keys"
    }

    fn modify_params(&self, params: &mut SynthParams) {
        // Keys envelope: instant attack, natural decay/sustain/release
        params.attack = 0.001; // 1ms - instant attack for hammer strike
        params.decay = 0.15; // 150ms - quick initial decay
        params.sustain = 0.4; // Lower sustain
        params.release = 0.25; // 250ms - natural release

        // Use triangle wave for mellow, bell-like tone
        if params.waveform == "sine" {
            params.waveform = "triangle".to_string();
        }
    }

    fn post_process(
        &self,
        samples: &mut [f32],
        sample_rate: u32,
        options: &HashMap<String, f32>,
    ) -> Result<()> {
        // Add "click" transient at the beginning (piano hammer hitting string)
        // Configurable option: click_amount (default: 0.4 = 40%)
        let click_amount = options.get("click_amount").copied().unwrap_or(0.4);

        let click_duration = (sample_rate as f32 * 0.003) as usize; // 3ms of click
        let click_duration = click_duration.min(samples.len());

        // Generate pseudo-random noise for click (using deterministic LFSR-like approach)
        let mut noise_state = 0x1234u32; // Seed for pseudo-random

        for i in 0..click_duration {
            // Generate pseudo-random noise using a simple LCG
            noise_state = noise_state.wrapping_mul(1103515245).wrapping_add(12345);
            let noise = ((noise_state >> 16) as f32 / 32768.0) - 1.0; // -1.0 to 1.0

            // Envelope for the click (very short, decays quickly)
            let click_envelope = 1.0 - (i as f32 / click_duration as f32);
            let click_envelope = click_envelope * click_envelope * click_envelope; // Cubic decay for sharp attack

            // Mix click (controlled by click_amount) with original signal
            let click_vol = click_amount * click_envelope;
            samples[i] = samples[i] * (1.0 - click_vol * 0.3) + noise * click_vol;
        }

        // Add subtle harmonic complexity (like piano strings)
        // Mix in a slightly detuned version for natural chorus
        let detune_samples = (sample_rate as f32 * 0.002) as usize; // 2ms detune

        if samples.len() > detune_samples * 2 {
            for i in detune_samples..samples.len() - detune_samples {
                let detuned = samples[i - detune_samples] * 0.12;
                samples[i] = samples[i] * 0.88 + detuned;
            }
        }

        // Add gentle low-pass for warmth (simulate wooden resonance)
        let mut prev1 = 0.0f32;
        let mut prev2 = 0.0f32;

        for sample in samples.iter_mut() {
            let filtered = *sample * 0.5 + prev1 * 0.35 + prev2 * 0.15;
            prev2 = prev1;
            prev1 = filtered;
            *sample = filtered;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_params() {
        let keys = KeysSynth;
        let mut params = SynthParams::default();

        keys.modify_params(&mut params);

        assert!(params.attack < 0.01);
        assert_eq!(params.waveform, "triangle");
    }
}
