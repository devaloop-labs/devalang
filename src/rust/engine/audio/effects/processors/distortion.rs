use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct DistortionProcessor {
    amount: f32,
    mix: f32,
}

impl DistortionProcessor {
    pub fn new(amount: f32, mix: f32) -> Self {
        Self {
            amount: amount.clamp(0.0, 1.0),
            mix: mix.clamp(0.0, 1.0),
        }
    }
}

impl Default for DistortionProcessor {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

impl EffectProcessor for DistortionProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        // Map amount -> drive factor for audible range
        let drive = 1.0 + self.amount * 29.0; // 1x..30x
        for sample in samples.iter_mut() {
            let input = *sample;
            let driven = input * drive;
            let distorted = driven.tanh();
            *sample = input * (1.0 - self.mix) + distorted * self.mix;
        }
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn name(&self) -> &str {
        "Distortion"
    }
}
