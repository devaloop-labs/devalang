use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct RollProcessor {
    pub duration_ms: f32,
    pub sync: bool,
    pub repeats: i32,
    pub fade: f32,
}

impl RollProcessor {
    pub fn new(duration_ms: f32, sync: bool, repeats: i32, fade: f32) -> Self {
        Self {
            duration_ms: duration_ms.max(1.0),
            sync,
            repeats: repeats.clamp(1, 16),
            fade: fade.clamp(0.0, 1.0),
        }
    }
}

impl Default for RollProcessor {
    fn default() -> Self {
        Self::new(100.0, false, 4, 0.02)
    }
}

impl EffectProcessor for RollProcessor {
    fn process(&mut self, samples: &mut [f32], sr: u32) {
        let frames = samples.len() / 2;
        let ms2frames = |ms: f32| ((ms / 1000.0) * sr as f32) as usize;
        let len = ms2frames(self.duration_ms).max(1).min(frames);
        // take first 'len' frames and repeat
        let mut segment = vec![0.0f32; len * 2];
        for i in 0..len {
            let si = i * 2;
            segment[si] = samples[si];
            if si + 1 < samples.len() {
                segment[si + 1] = samples[si + 1];
            }
        }
        let mut out = vec![0.0f32; samples.len()];
        let mut pos = 0usize;
        for _r in 0..self.repeats {
            for i in 0..len {
                let di = pos * 2;
                if di + 1 < out.len() {
                    out[di] = segment[i * 2];
                    out[di + 1] = segment[i * 2 + 1];
                }
                pos += 1;
                if pos >= frames {
                    break;
                }
            }
            if pos >= frames {
                break;
            }
        }
        // fill rest with original tail
        let mut src_pos = len * 2;
        while pos < frames {
            let di = pos * 2;
            let si = src_pos.min(samples.len() - 2);
            out[di] = samples[si];
            out[di + 1] = samples[si + 1];
            pos += 1;
            src_pos += 2;
        }
        samples.copy_from_slice(&out);
    }
    fn reset(&mut self) {}
    fn name(&self) -> &str {
        "Roll"
    }
}
