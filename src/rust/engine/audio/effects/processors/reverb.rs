use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ReverbProcessor {
    room_size: f32,
    damping: f32,
    mix: f32,
    decay: f32,
    // Comb filters (parallel)
    comb_buffers: Vec<Vec<f32>>,
    comb_positions: Vec<usize>,
    comb_feedback: Vec<f32>,
    // Allpass filters (series)
    allpass_buffers: Vec<Vec<f32>>,
    allpass_positions: Vec<usize>,
}

impl ReverbProcessor {
    pub fn new(room_size: f32, damping: f32, decay: f32, mix: f32) -> Self {
        let room_size = room_size.clamp(0.0, 1.0);
        let damping = damping.clamp(0.0, 1.0);
        let decay = decay.clamp(0.0, 2.0);

        // Comb filter delays (in samples at 44.1kHz)
        let comb_delays = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        let mut comb_buffers = Vec::new();
        let mut comb_feedback = Vec::new();

        for &delay in &comb_delays {
            comb_buffers.push(vec![0.0; delay]);
            // decay controls how long the reverb tails are. Compute a stable feedback < 1.0
            let base = 0.78 + room_size * 0.14; // base feedback per room size
            let mult = 0.6 + decay * 0.2; // decay scales length but keep <~1.0
            let mut fb = base * mult;
            if fb >= 0.995 {
                fb = 0.995;
            }
            comb_feedback.push(fb);
        }

        // Allpass filter delays
        let allpass_delays = [556, 441, 341, 225];
        let mut allpass_buffers = Vec::new();

        for &delay in &allpass_delays {
            allpass_buffers.push(vec![0.0; delay]);
        }

        Self {
            room_size,
            damping,
            mix: mix.clamp(0.0, 1.0),
            decay,
            comb_buffers,
            comb_positions: vec![0; 8],
            comb_feedback,
            allpass_buffers,
            allpass_positions: vec![0; 4],
        }
    }
}

impl Default for ReverbProcessor {
    fn default() -> Self {
        Self::new(0.5, 0.5, 0.5, 0.3)
    }
}

impl EffectProcessor for ReverbProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        for i in 0..samples.len() {
            let input = samples[i];
            let mut output = 0.0;

            // Process through parallel comb filters
            for (j, buffer) in self.comb_buffers.iter_mut().enumerate() {
                let pos = self.comb_positions[j];
                let delayed = buffer[pos];

                // Apply damping as a simple attenuation of the delayed sample
                let filtered = delayed * (1.0 - self.damping);
                buffer[pos] = input + filtered * self.comb_feedback[j];

                // accumulate delayed signal for wet output
                output += delayed;

                self.comb_positions[j] = (pos + 1) % buffer.len();
            }

            output /= self.comb_buffers.len() as f32;

            // Process through series allpass filters
            for (j, buffer) in self.allpass_buffers.iter_mut().enumerate() {
                let pos = self.allpass_positions[j];
                let delayed = buffer[pos];

                buffer[pos] = output + delayed * 0.5;
                output = delayed - output * 0.5;

                self.allpass_positions[j] = (pos + 1) % buffer.len();
            }

            // Mix wet and dry
            samples[i] = input * (1.0 - self.mix) + output * self.mix;
        }
    }

    fn reset(&mut self) {
        for buffer in &mut self.comb_buffers {
            buffer.fill(0.0);
        }
        for buffer in &mut self.allpass_buffers {
            buffer.fill(0.0);
        }
        self.comb_positions.fill(0);
        self.allpass_positions.fill(0);
    }

    fn name(&self) -> &str {
        "Reverb"
    }
}
