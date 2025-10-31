use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct VibratoProcessor {
    pub rate: f32,
    pub depth: f32,
    pub sync: bool,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
    pos: usize,
    phase: f32,
}

impl VibratoProcessor {
    pub fn new(rate: f32, depth: f32, sync: bool) -> Self {
        let max_delay = 2048usize;
        Self {
            rate: rate.clamp(0.1, 10.0),
            depth: depth.clamp(0.0, 0.02),
            sync,
            buf_l: vec![0.0; max_delay],
            buf_r: vec![0.0; max_delay],
            pos: 0,
            phase: 0.0,
        }
    }
}

impl Default for VibratoProcessor {
    fn default() -> Self {
        Self::new(5.0, 0.003, false)
    }
}

impl EffectProcessor for VibratoProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        let sr_f = sr as f32;
        let max_delay = (self.buf_l.len() - 2) as f32;
        for i in (0..samples.len()).step_by(2) {
            let in_l = samples[i];
            let in_r = if i + 1 < samples.len() {
                samples[i + 1]
            } else {
                in_l
            };
            // compute read position first (read from previous samples), then write current input into buffer
            let lfo = (2.0 * std::f32::consts::PI * self.phase).sin();
            let delay = ((self.depth * max_delay) * (lfo + 1.0) * 0.5).max(0.0);
            let read_pos =
                (self.pos as f32 - delay + self.buf_l.len() as f32) % self.buf_l.len() as f32;
            let idx = read_pos.floor() as usize % self.buf_l.len();
            let frac = read_pos - read_pos.floor();
            let a = self.buf_l[idx];
            let b = self.buf_l[(idx + 1) % self.buf_l.len()];
            samples[i] = a * (1.0 - frac) + b * frac;

            if i + 1 < samples.len() {
                let a2 = self.buf_r[idx];
                let b2 = self.buf_r[(idx + 1) % self.buf_r.len()];
                samples[i + 1] = a2 * (1.0 - frac) + b2 * frac;
            }

            // now write current input into buffer (overwrite oldest)
            let write_idx = self.pos % self.buf_l.len();
            self.buf_l[write_idx] = in_l;
            self.buf_r[write_idx] = in_r;
            self.pos = (self.pos + 1) % self.buf_l.len();
            self.phase += self.rate / sr_f;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
    }

    fn reset(&mut self) {
        self.pos = 0;
        self.phase = 0.0;
    }

    fn name(&self) -> &str {
        "Vibrato"
    }
}
