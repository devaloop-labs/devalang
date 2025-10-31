use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct SpeedProcessor {
    speed: f32,
    buffer: Vec<f32>,
}

impl SpeedProcessor {
    pub fn new(speed: f32) -> Self {
        // Accept any speed value (positive >1 speeds speed up, negative reverses).
        // Avoid exact zero which would be invalid; clamp tiny values to a small epsilon
        let clamped = if speed.abs() < 0.0001 {
            if speed.is_sign_negative() {
                -0.0001
            } else {
                0.0001
            }
        } else {
            speed
        };

        Self {
            speed: clamped,
            buffer: Vec::new(),
        }
    }
}

impl Default for SpeedProcessor {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl EffectProcessor for SpeedProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        if (self.speed - 1.0).abs() < f32::EPSILON {
            return; // No speed change needed
        }

        let original_len = samples.len();
        let speed_abs = self.speed.abs();
        let new_len = (original_len as f32 / speed_abs) as usize;

        // Resize buffer if needed
        self.buffer.resize(new_len, 0.0);

        // Simple linear interpolation for speed change
        // For positive speeds, map dest -> src forward. For negative speeds, produce reversed playback.
        if self.speed > 0.0 {
            for i in 0..new_len {
                let src_pos = i as f32 * self.speed;
                let src_idx = src_pos.floor() as isize;
                let frac = src_pos.fract();

                if src_idx >= 0 && (src_idx as usize) + 1 < original_len {
                    let idx = src_idx as usize;
                    self.buffer[i] = samples[idx] * (1.0 - frac) + samples[idx + 1] * frac;
                } else if src_idx >= 0 && (src_idx as usize) < original_len {
                    self.buffer[i] = samples[src_idx as usize];
                } else {
                    self.buffer[i] = 0.0;
                }
            }
        } else {
            // speed < 0: reversed playback at magnitude 'speed_abs'
            for i in 0..new_len {
                // map destination index i to source position from the end
                let src_pos_from_end = i as f32 * speed_abs;
                // compute source position relative to start: original_len - 1 - src_pos_from_end
                let src_pos = (original_len as f32 - 1.0) - src_pos_from_end;
                if src_pos < 0.0 {
                    self.buffer[i] = 0.0;
                    continue;
                }
                let src_idx = src_pos.floor() as usize;
                let frac = src_pos.fract();

                if src_idx + 1 < original_len {
                    self.buffer[i] = samples[src_idx] * (1.0 - frac) + samples[src_idx + 1] * frac;
                } else if src_idx < original_len {
                    self.buffer[i] = samples[src_idx];
                } else {
                    self.buffer[i] = 0.0;
                }
            }
        }

        // Copy processed samples back into the provided slice.
        // We cannot change the slice length; copy as many as fit and zero the remainder.
        let copy_len = std::cmp::min(original_len, new_len);
        samples[..copy_len].copy_from_slice(&self.buffer[..copy_len]);
        if new_len < original_len {
            for s in &mut samples[new_len..original_len] {
                *s = 0.0;
            }
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }

    fn name(&self) -> &str {
        "Speed"
    }
}
