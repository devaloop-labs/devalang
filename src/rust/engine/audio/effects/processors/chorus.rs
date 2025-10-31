use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct ChorusProcessor {
    depth: f32,
    rate: f32,
    mix: f32,
    phase: f32,
    delay_buffer: Vec<f32>,
    buffer_pos: usize,
}

impl ChorusProcessor {
    pub fn new(depth: f32, rate: f32, mix: f32) -> Self {
        Self {
            depth: depth.clamp(0.0, 1.0),
            rate: rate.clamp(0.1, 10.0),
            mix: mix.clamp(0.0, 1.0),
            phase: 0.0,
            delay_buffer: vec![0.0; 8820], // ~200ms at 44.1kHz
            buffer_pos: 0,
        }
    }
}

impl Default for ChorusProcessor {
    fn default() -> Self {
        Self::new(0.7, 0.5, 0.5)
    }
}

impl EffectProcessor for ChorusProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let max_delay_samples = (0.020 * sample_rate as f32) as usize; // 20ms max delay

        for i in (0..samples.len()).step_by(2) {
            // Update LFO phase
            self.phase += self.rate / sample_rate as f32;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }

            // Calculate delay offset using sine LFO
            let lfo = (2.0 * PI * self.phase).sin();
            let delay_samples =
                (self.depth * max_delay_samples as f32 * (lfo + 1.0) / 2.0) as usize;
            let delay_samples = delay_samples.min(self.delay_buffer.len() - 1);

            // Process left and right channels
            for ch in 0..2 {
                if i + ch < samples.len() {
                    let input = samples[i + ch];

                    // Write to delay buffer
                    self.delay_buffer[self.buffer_pos] = input;

                    // Read from delayed position
                    let read_pos = (self.buffer_pos + self.delay_buffer.len() - delay_samples)
                        % self.delay_buffer.len();
                    let delayed = self.delay_buffer[read_pos];

                    // Mix wet and dry
                    samples[i + ch] = input * (1.0 - self.mix) + delayed * self.mix;
                }
            }

            self.buffer_pos = (self.buffer_pos + 1) % self.delay_buffer.len();
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.delay_buffer.fill(0.0);
        self.buffer_pos = 0;
    }

    fn name(&self) -> &str {
        "Chorus"
    }
}
