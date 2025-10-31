use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use std::fmt::Debug;

/// Drive effect - tube-style saturation
#[derive(Debug, Clone)]
pub struct DriveProcessor {
    amount: f32,
    tone: f32,
    mix: f32,
    color: f32,
    prev_l: f32,
    prev_r: f32,
}

impl DriveProcessor {
    pub fn new(amount: f32, tone: f32, color: f32, mix: f32) -> Self {
        Self {
            amount: amount.clamp(0.0, 1.0),
            tone: tone.clamp(0.0, 1.0),
            mix: mix.clamp(0.0, 1.0),
            color: color.clamp(0.0, 1.0),
            prev_l: 0.0,
            prev_r: 0.0,
        }
    }
}

impl Default for DriveProcessor {
    fn default() -> Self {
        Self::new(0.5, 0.5, 0.5, 0.7)
    }
}

impl EffectProcessor for DriveProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        // Map amount (0.0..1.0) to a perceptible drive/gain range (1x .. 20x)
        let gain = 1.0 + self.amount * 19.0; // 1x to 20x

        // Process interleaved stereo by frames
        for i in (0..samples.len()).step_by(2) {
            // Left
            let input_l = samples[i];
            let driven_l = input_l * gain;

            // Use tanh waveshaper for more audible saturation behaviour
            let distorted_l = driven_l.tanh();

            // Tone mixes the distorted vs original to shape brightness
            let mut toned_l = distorted_l * self.tone + input_l * (1.0 - self.tone);
            let alpha = 0.05 + self.color * 0.95; // 0.05..1.0 smoothing
            toned_l = alpha * toned_l + (1.0 - alpha) * self.prev_l;
            self.prev_l = toned_l;
            samples[i] = input_l * (1.0 - self.mix) + toned_l * self.mix;

            // Right
            if i + 1 < samples.len() {
                let input_r = samples[i + 1];
                let driven_r = input_r * gain;
                let distorted_r = driven_r.tanh();
                let mut toned_r = distorted_r * self.tone + input_r * (1.0 - self.tone);
                let alpha_r = 0.05 + self.color * 0.95;
                toned_r = alpha_r * toned_r + (1.0 - alpha_r) * self.prev_r;
                self.prev_r = toned_r;
                samples[i + 1] = input_r * (1.0 - self.mix) + toned_r * self.mix;
            }
        }
    }

    fn reset(&mut self) {
        self.prev_l = 0.0;
        self.prev_r = 0.0;
    }

    fn name(&self) -> &str {
        "Drive"
    }
}
