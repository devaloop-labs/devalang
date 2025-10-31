use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct StretchProcessor {
    pub factor: f32, // 0.25 .. 4.0
    pub pitch: f32,  // semitones
    pub formant: bool,
}

impl StretchProcessor {
    pub fn new(factor: f32, pitch: f32, formant: bool) -> Self {
        Self {
            factor: factor.clamp(0.25, 4.0),
            pitch: pitch.clamp(-48.0, 48.0),
            formant,
        }
    }
}

impl Default for StretchProcessor {
    fn default() -> Self {
        Self::new(1.0, 0.0, false)
    }
}

impl EffectProcessor for StretchProcessor {
    fn process(&mut self, samples: &mut [f32], _sr: u32) {
        // naive time-stretch by resampling: when factor>1 we speed up (shorter), <1 we stretch
        let frames = samples.len() / 2;
        if self.factor == 1.0 {
            return;
        }
        let mut out = vec![0.0f32; samples.len()];
        for i in 0..frames {
            let src_f = (i as f32) / self.factor;
            let idx = src_f.floor() as usize;
            let frac = src_f - idx as f32;

            let a_l = samples.get(idx * 2).copied().unwrap_or(0.0);
            let b_l = samples.get((idx + 1) * 2).copied().unwrap_or(a_l);
            let val_l = a_l * (1.0 - frac) + b_l * frac;
            out[i * 2] = val_l;
            let a_r = samples.get(idx * 2 + 1).copied().unwrap_or(a_l);
            let b_r = samples.get((idx + 1) * 2 + 1).copied().unwrap_or(a_r);
            let val_r = a_r * (1.0 - frac) + b_r * frac;
            out[i * 2 + 1] = val_r;
        }
        samples.copy_from_slice(&out);
    }
    fn reset(&mut self) {}
    fn name(&self) -> &str {
        "Stretch"
    }
}
