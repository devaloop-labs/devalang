use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use rand::prelude::*;

#[derive(Debug, Clone)]
pub struct SliceProcessor {
    pub segments: i32,
    pub mode: String, // "sequential" | "random"
    pub crossfade: f32,
}

impl SliceProcessor {
    pub fn new(segments: i32, mode: &str, crossfade: f32) -> Self {
        Self {
            segments: segments.clamp(1, 16),
            mode: mode.to_string(),
            crossfade: crossfade.clamp(0.0, 1.0),
        }
    }
}

impl Default for SliceProcessor {
    fn default() -> Self {
        Self::new(4, "sequential", 0.01)
    }
}

impl EffectProcessor for SliceProcessor {
    fn process(&mut self, samples: &mut [f32], _sr: u32) {
        let frames = samples.len() / 2;
        let segs = self.segments.max(1) as usize;
        let seg_len = (frames / segs).max(1);
        let mut out = vec![0.0f32; frames * 2];
        let mut order: Vec<usize> = (0..segs).collect();
        if self.mode == "random" {
            order.shuffle(&mut thread_rng());
        }
        let mut dst = 0usize;
        for &s in order.iter() {
            let start = s * seg_len;
            let end = ((s + 1) * seg_len).min(frames);
            for i in start..end {
                let si = i * 2;
                if dst < frames {
                    let di = dst * 2;
                    out[di] = samples[si];
                    out[di + 1] = samples.get(si + 1).copied().unwrap_or(samples[si]);
                    dst += 1;
                }
            }
        }
        // write back
        for i in 0..frames {
            let si = i * 2;
            samples[si] = out[si];
            if si + 1 < samples.len() {
                samples[si + 1] = out[si + 1];
            }
        }
    }
    fn reset(&mut self) {}
    fn name(&self) -> &str {
        "Slice"
    }
}
