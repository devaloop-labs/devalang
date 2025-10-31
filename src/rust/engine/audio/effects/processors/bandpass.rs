use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use crate::engine::audio::effects::processors::{HighpassProcessor, LowpassProcessor};

#[derive(Debug, Clone)]
pub struct BandpassProcessor {
    pub cutoff: f32,
    pub resonance: f32,
    lp: LowpassProcessor,
    hp: HighpassProcessor,
}

impl BandpassProcessor {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff: cutoff.clamp(20.0, 20000.0),
            resonance: resonance.clamp(0.0, 1.0),
            lp: LowpassProcessor::new(cutoff * 1.5, resonance),
            hp: HighpassProcessor::new((cutoff * 0.5).max(20.0), resonance),
        }
    }
}

impl Default for BandpassProcessor {
    fn default() -> Self {
        Self::new(1000.0, 0.2)
    }
}

impl EffectProcessor for BandpassProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        // apply highpass then lowpass to get a band
        self.hp.process(samples, sr);
        self.lp.process(samples, sr);
    }

    fn reset(&mut self) {
        self.lp.reset();
        self.hp.reset();
    }

    fn name(&self) -> &str {
        "Bandpass"
    }
}
