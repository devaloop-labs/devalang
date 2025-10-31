use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct HighpassProcessor {
    pub cutoff: f32,
    pub resonance: f32,
    prev_x_l: f32,
    prev_x_r: f32,
    prev_y_l: f32,
    prev_y_r: f32,
}

impl HighpassProcessor {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff: cutoff.clamp(20.0, 20000.0),
            resonance: resonance.clamp(0.0, 1.0),
            prev_x_l: 0.0,
            prev_x_r: 0.0,
            prev_y_l: 0.0,
            prev_y_r: 0.0,
        }
    }
}

impl Default for HighpassProcessor {
    fn default() -> Self {
        Self::new(200.0, 0.1)
    }
}

impl EffectProcessor for HighpassProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        let fs = sr as f32;
        let fc = self.cutoff.max(20.0).min(fs * 0.49);
        let omega = 2.0 * std::f32::consts::PI * fc / fs;
        let alpha = 1.0 / (omega + 1.0);

        for i in (0..samples.len()).step_by(2) {
            let x_l = samples[i];
            let y_l = alpha * (self.prev_y_l + x_l - self.prev_x_l);
            self.prev_x_l = x_l;
            self.prev_y_l = y_l;
            samples[i] = y_l;
            if i + 1 < samples.len() {
                let x_r = samples[i + 1];
                let y_r = alpha * (self.prev_y_r + x_r - self.prev_x_r);
                self.prev_x_r = x_r;
                self.prev_y_r = y_r;
                samples[i + 1] = y_r;
            }
        }
    }

    fn reset(&mut self) {
        self.prev_x_l = 0.0;
        self.prev_x_r = 0.0;
        self.prev_y_l = 0.0;
        self.prev_y_r = 0.0;
    }

    fn name(&self) -> &str {
        "Highpass"
    }
}
