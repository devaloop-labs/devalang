use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct CompressorProcessor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    envelope: f32,
}

impl CompressorProcessor {
    pub fn new(threshold: f32, ratio: f32, attack: f32, release: f32) -> Self {
        Self {
            threshold,
            ratio: ratio.max(1.0),
            attack: attack.max(0.001),
            release: release.max(0.001),
            envelope: 0.0,
        }
    }
}

impl Default for CompressorProcessor {
    fn default() -> Self {
        Self::new(-20.0, 4.0, 0.005, 0.1)
    }
}

impl EffectProcessor for CompressorProcessor {
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

            // Update envelope
            let target = if db > self.threshold {
                self.threshold + (db - self.threshold) / self.ratio
            } else {
                db
            };

            let coeff = if target > self.envelope {
                attack_coeff
            } else {
                release_coeff
            };

            self.envelope = target + coeff * (self.envelope - target);

            // Calculate gain reduction
            let gain_db = self.envelope - db;
            let gain = 10.0_f32.powf(gain_db / 20.0);

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
        "Compressor"
    }
}
