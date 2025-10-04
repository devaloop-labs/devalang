/// Arp (Arpeggio) synth - rhythmically gated, staccato notes
/// Characteristics:
/// - Short, chopped notes
/// - Quick attack and release
/// - Moderate sustain (but short overall duration)
/// - Bright, punchy sound
use super::SynthType;
use crate::engine::audio::generator::SynthParams;
use anyhow::Result;
use std::collections::HashMap;

pub struct ArpSynth;

impl SynthType for ArpSynth {
    fn name(&self) -> &str {
        "arp"
    }

    fn modify_params(&self, params: &mut SynthParams) {
        // Arp envelope: instant attack, quick decay, MAINTAIN sustain for gating
        // The sustain must be high so the post_process gating can work
        params.attack = 0.001; // 1ms - instant attack
        params.decay = 0.05; // 50ms - quick initial decay
        params.sustain = 0.85; // HIGH sustain - gating will create staccato effect
        params.release = 0.01; // 10ms - very quick release
    }

    fn post_process(
        &self,
        samples: &mut [f32],
        sample_rate: u32,
        options: &HashMap<String, f32>,
    ) -> Result<()> {
        // Create rhythmic gating pattern - repeating on/off pulses
        // Configurable options:
        // - gate: pulse on ratio (default: 0.30 = 30% ON)
        // - rate: pulses per second (default: 16th notes at 120 BPM = ~8 pulses/sec)

        let gate = options.get("gate").copied().unwrap_or(0.30);
        let rate = options.get("rate").copied().unwrap_or(8.0); // pulses per second

        let pulse_duration_samples = (sample_rate as f32 / rate) as usize;
        let pulse_on_samples = (pulse_duration_samples as f32 * gate) as usize;
        let fade_samples = (sample_rate as f32 * 0.005) as usize; // 5ms fade

        let _num_pulses = samples.len() / pulse_duration_samples;

        // Apply gating pattern
        for i in 0..samples.len() {
            let position_in_pulse = i % pulse_duration_samples;

            if position_in_pulse < pulse_on_samples {
                // Keep the sample (pulse ON)
            } else if position_in_pulse < pulse_on_samples + fade_samples {
                // Fade out quickly
                let fade_progress =
                    (position_in_pulse - pulse_on_samples) as f32 / fade_samples as f32;
                samples[i] *= 1.0 - fade_progress;
            } else {
                // Silence (pulse OFF)
                samples[i] = 0.0;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arp_params() {
        let arp = ArpSynth;
        let mut params = SynthParams::default();

        arp.modify_params(&mut params);

        assert!(params.attack < 0.01);
        assert!(params.release < 0.05);
    }
}
