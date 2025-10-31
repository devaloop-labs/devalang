use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct MonoizerProcessor {
    pub enabled: bool,
    pub mix: f32,
}

impl MonoizerProcessor {
    pub fn new(enabled: bool, mix: f32) -> Self {
        Self {
            enabled,
            mix: mix.clamp(0.0, 1.0),
        }
    }
}

impl Default for MonoizerProcessor {
    fn default() -> Self {
        Self::new(true, 1.0)
    }
}

impl EffectProcessor for MonoizerProcessor {
    fn process(&mut self, samples: &mut [f32], _sr: u32) {
        if !self.enabled {
            return;
        }
        for i in (0..samples.len()).step_by(2) {
            let l = samples[i];
            let r = if i + 1 < samples.len() {
                samples[i + 1]
            } else {
                l
            };
            let mid = (l + r) * 0.5;
            samples[i] = l * (1.0 - self.mix) + mid * self.mix;
            if i + 1 < samples.len() {
                samples[i + 1] = r * (1.0 - self.mix) + mid * self.mix;
            }
        }
    }
    fn reset(&mut self) {}
    fn name(&self) -> &str {
        "Monoizer"
    }
}
