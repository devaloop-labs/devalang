use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct StereoProcessor {
    pub width: f32, // 0.0..2.0
}

impl StereoProcessor {
    pub fn new(width: f32) -> Self {
        Self {
            width: width.clamp(0.0, 2.0),
        }
    }
}

impl Default for StereoProcessor {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl EffectProcessor for StereoProcessor {
    fn process(&mut self, samples: &mut [f32], _sr: u32) {
        for i in (0..samples.len()).step_by(2) {
            let l = samples[i];
            let r = if i + 1 < samples.len() {
                samples[i + 1]
            } else {
                l
            };
            let mid = (l + r) * 0.5;
            let side = (l - r) * 0.5 * self.width;
            samples[i] = mid + side;
            if i + 1 < samples.len() {
                samples[i + 1] = mid - side;
            }
        }
    }
    fn reset(&mut self) {}
    fn name(&self) -> &str {
        "Stereo"
    }
}
