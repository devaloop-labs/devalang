use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct FreezeProcessor {
    pub enabled: bool,
    pub fade: f32,
    pub hold: f32,
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    captured: bool,
}

impl FreezeProcessor {
    pub fn new(enabled: bool, fade: f32, hold: f32) -> Self {
        let max = 44100usize * 5;
        Self {
            enabled,
            fade: fade.clamp(0.0, 1.0),
            hold: hold.clamp(0.05, 5.0),
            buffer_l: vec![0.0; max],
            buffer_r: vec![0.0; max],
            captured: false,
        }
    }
}

impl Default for FreezeProcessor {
    fn default() -> Self {
        Self::new(false, 0.2, 0.5)
    }
}

impl EffectProcessor for FreezeProcessor {
    fn process(&mut self, samples: &mut [f32], _sr: u32) {
        if !self.enabled {
            return;
        }
        // capture the buffer once then replay it
        if !self.captured {
            let len = samples.len() / 2;
            for i in 0..len {
                let idx = i * 2;
                self.buffer_l[i] = samples[idx];
                self.buffer_r[i] = if idx + 1 < samples.len() {
                    samples[idx + 1]
                } else {
                    samples[idx]
                };
            }
            self.captured = true;
        }
        let len = samples.len() / 2;
        for i in 0..len {
            let idx = i * 2;
            let frozen_l = self.buffer_l[i];
            let frozen_r = self.buffer_r[i];
            samples[idx] = samples[idx] * (1.0 - self.fade) + frozen_l * self.fade;
            if idx + 1 < samples.len() {
                samples[idx + 1] = samples[idx + 1] * (1.0 - self.fade) + frozen_r * self.fade;
            }
        }
    }
    fn reset(&mut self) {
        self.captured = false;
    }
    fn name(&self) -> &str {
        "Freeze"
    }
}
