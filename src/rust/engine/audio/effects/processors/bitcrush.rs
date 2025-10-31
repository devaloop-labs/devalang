use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct BitcrushProcessor {
    pub depth: f32,       // bits nominal (1.0..16.0)
    pub sample_rate: f32, // downsample rate in Hz (e.g. 8000.0)
    pub mix: f32,
    sample_pos: usize,
    hold_step: usize,
}

impl BitcrushProcessor {
    pub fn new(depth: f32, sample_rate: f32, mix: f32) -> Self {
        Self {
            depth: depth.clamp(1.0, 16.0),
            sample_rate: sample_rate.max(100.0),
            mix: mix.clamp(0.0, 1.0),
            sample_pos: 0,
            hold_step: 1,
        }
    }
}

impl Default for BitcrushProcessor {
    fn default() -> Self {
        Self::new(8.0, 8000.0, 0.5)
    }
}

impl EffectProcessor for BitcrushProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        let sr_f = sr as f32;
        // compute hold_step in samples
        let target_rate = self.sample_rate.clamp(100.0, sr_f);
        let step = (sr_f / target_rate).max(1.0) as usize;
        self.hold_step = step;

        let levels = (2u32.pow(self.depth as u32)) as f32;
        let mut hold_l = 0.0f32;
        let mut hold_r = 0.0f32;
        for i in (0..samples.len()).step_by(2) {
            if self.sample_pos % self.hold_step == 0 {
                // quantize
                let in_l = samples[i];
                let ql =
                    (((in_l + 1.0) * 0.5 * (levels - 1.0)).round() / (levels - 1.0)) * 2.0 - 1.0;
                hold_l = ql;
                if i + 1 < samples.len() {
                    let in_r = samples[i + 1];
                    let qr = (((in_r + 1.0) * 0.5 * (levels - 1.0)).round() / (levels - 1.0)) * 2.0
                        - 1.0;
                    hold_r = qr;
                }
            }
            samples[i] = samples[i] * (1.0 - self.mix) + hold_l * self.mix;
            if i + 1 < samples.len() {
                samples[i + 1] = samples[i + 1] * (1.0 - self.mix) + hold_r * self.mix;
            }
            self.sample_pos = self.sample_pos.wrapping_add(1);
        }
    }

    fn reset(&mut self) {
        self.sample_pos = 0;
    }

    fn name(&self) -> &str {
        "Bitcrush"
    }
}
