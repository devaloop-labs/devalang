use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct EqProcessor {
    low_gain: f32,
    mid_gain: f32,
    high_gain: f32,
    low_l: f32,
    low_r: f32,
    high_prev_x_l: f32,
    high_prev_x_r: f32,
    high_prev_y_l: f32,
    high_prev_y_r: f32,
}

impl EqProcessor {
    pub fn new(low_gain: f32, mid_gain: f32, high_gain: f32) -> Self {
        Self {
            low_gain: low_gain.clamp(0.0, 2.0),
            mid_gain: mid_gain.clamp(0.0, 2.0),
            high_gain: high_gain.clamp(0.0, 2.0),
            low_l: 0.0,
            low_r: 0.0,
            high_prev_x_l: 0.0,
            high_prev_x_r: 0.0,
            high_prev_y_l: 0.0,
            high_prev_y_r: 0.0,
        }
    }
}

impl Default for EqProcessor {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }
}

impl EffectProcessor for EqProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let fs = sample_rate as f32;

        let low_fc = 320.0_f32.min(fs * 0.49);
        let low_omega = 2.0 * std::f32::consts::PI * low_fc / fs;
        let low_alpha = low_omega / (low_omega + 1.0);

        let high_fc = 3200.0_f32.min(fs * 0.49);
        let high_omega = 2.0 * std::f32::consts::PI * high_fc / fs;
        let high_alpha = 1.0 / (high_omega + 1.0);

        for i in (0..samples.len()).step_by(2) {
            let input_l = samples[i];
            self.low_l += low_alpha * (input_l - self.low_l);
            let high_l = high_alpha * (self.high_prev_y_l + input_l - self.high_prev_x_l);
            self.high_prev_x_l = input_l;
            self.high_prev_y_l = high_l;
            let mid_l = input_l - self.low_l - high_l;
            samples[i] =
                self.low_l * self.low_gain + mid_l * self.mid_gain + high_l * self.high_gain;

            if i + 1 < samples.len() {
                let input_r = samples[i + 1];
                self.low_r += low_alpha * (input_r - self.low_r);
                let high_r = high_alpha * (self.high_prev_y_r + input_r - self.high_prev_x_r);
                self.high_prev_x_r = input_r;
                self.high_prev_y_r = high_r;
                let mid_r = input_r - self.low_r - high_r;
                samples[i + 1] =
                    self.low_r * self.low_gain + mid_r * self.mid_gain + high_r * self.high_gain;
            }
        }
    }

    fn reset(&mut self) {
        self.low_l = 0.0;
        self.low_r = 0.0;
        self.high_prev_x_l = 0.0;
        self.high_prev_x_r = 0.0;
        self.high_prev_y_l = 0.0;
        self.high_prev_y_r = 0.0;
    }

    fn name(&self) -> &str {
        "EQ"
    }
}
