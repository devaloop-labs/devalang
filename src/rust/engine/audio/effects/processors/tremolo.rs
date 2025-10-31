use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct TremoloProcessor {
    pub rate: f32,
    pub depth: f32,
    pub sync: bool,
    phase: f32,
}

impl TremoloProcessor {
    pub fn new(rate: f32, depth: f32, sync: bool) -> Self {
        Self {
            rate: rate.clamp(0.1, 20.0),
            depth: depth.clamp(0.0, 1.0),
            sync,
            phase: 0.0,
        }
    }
}

impl Default for TremoloProcessor {
    fn default() -> Self {
        Self::new(5.0, 0.5, false)
    }
}

impl EffectProcessor for TremoloProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        let sr_f = sr as f32;
        for i in (0..samples.len()).step_by(2) {
            let lfo = (2.0 * std::f32::consts::PI * self.phase).sin();
            let mod_amp = 1.0 - self.depth + self.depth * ((lfo + 1.0) * 0.5);
            samples[i] *= mod_amp;
            if i + 1 < samples.len() {
                samples[i + 1] *= mod_amp;
            }
            self.phase += self.rate / sr_f;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn name(&self) -> &str {
        "Tremolo"
    }
}
