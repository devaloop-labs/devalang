use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

/// Gate effect - silences audio below a threshold
#[derive(Debug, Clone)]
pub struct GateProcessor {
    threshold: f32,
    attack: f32,
    release: f32,
    envelope: f32,
}

impl GateProcessor {
    pub fn new(threshold: f32, attack: f32, release: f32) -> Self {
        Self {
            threshold,
            attack: attack.max(0.001),
            release: release.max(0.001),
            envelope: 0.0,
        }
    }
}

impl Default for GateProcessor {
    fn default() -> Self {
        Self::new(-30.0, 0.001, 0.05)
    }
}

impl EffectProcessor for GateProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let attack_coeff = (-1.0 / (self.attack * sample_rate as f32)).exp();
        let release_coeff = (-1.0 / (self.release * sample_rate as f32)).exp();

        for i in (0..samples.len()).step_by(2) {
            // Get stereo sample RMS
            let left = samples[i];
            let right = if i + 1 < samples.len() {
                samples[i + 1]
            } else {
                left
            };
            let rms = ((left * left + right * right) / 2.0).sqrt();

            // Convert to dB
            let db = if rms > 0.0001 {
                20.0 * rms.log10()
            } else {
                -100.0
            };

            // Gate logic: fully open if above threshold, fully closed if below
            let target = if db > self.threshold {
                0.0 // Full gain (0 dB)
            } else {
                -100.0 // Silent
            };

            let coeff = if target > self.envelope {
                attack_coeff
            } else {
                release_coeff
            };

            self.envelope = target + coeff * (self.envelope - target);

            // Calculate gain
            let gain = 10.0_f32.powf(self.envelope / 20.0);

            // Apply gain
            samples[i] *= gain;
            if i + 1 < samples.len() {
                samples[i + 1] *= gain;
            }
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn name(&self) -> &str {
        "Gate"
    }
}
