use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct LowpassProcessor {
    pub cutoff: f32,
    pub resonance: f32,
    prev_l: f32,
    prev_r: f32,
}

impl LowpassProcessor {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff: cutoff.clamp(20.0, 20000.0),
            resonance: resonance.clamp(0.0, 1.0),
            prev_l: 0.0,
            prev_r: 0.0,
        }
    }
}

impl Default for LowpassProcessor {
    fn default() -> Self {
        Self::new(5000.0, 0.1)
    }
}

impl EffectProcessor for LowpassProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        let fs = sr as f32;
        let fc = self.cutoff.max(20.0).min(fs * 0.49);
        let omega = 2.0 * std::f32::consts::PI * fc / fs;
        let alpha = omega / (omega + 1.0);

        for i in (0..samples.len()).step_by(2) {
            let x_l = samples[i];
            self.prev_l = self.prev_l + alpha * (x_l - self.prev_l);
            samples[i] = self.prev_l;
            if i + 1 < samples.len() {
                let x_r = samples[i + 1];
                self.prev_r = self.prev_r + alpha * (x_r - self.prev_r);
                samples[i + 1] = self.prev_r;
            }
        }
    }

    fn reset(&mut self) {
        self.prev_l = 0.0;
        self.prev_r = 0.0;
    }

    fn name(&self) -> &str {
        "Lowpass"
    }
}
