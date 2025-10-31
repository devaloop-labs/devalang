use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct ReverseProcessor {
    reversed: bool,
}

impl ReverseProcessor {
    pub fn new(reversed: bool) -> Self {
        Self { reversed }
    }
}

impl Default for ReverseProcessor {
    fn default() -> Self {
        Self::new(false)
    }
}

impl EffectProcessor for ReverseProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        if self.reversed {
            samples.reverse();
        }
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn name(&self) -> &str {
        "Reverse"
    }
}
