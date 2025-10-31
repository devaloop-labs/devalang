use crate::engine::audio::effects::processors::super_trait::EffectProcessor;

#[derive(Debug, Clone)]
pub struct DelayProcessor {
    time_ms: f32,
    feedback: f32,
    mix: f32,
    delay_buffer_l: Vec<f32>,
    delay_buffer_r: Vec<f32>,
    buffer_pos: usize,
}

impl DelayProcessor {
    pub fn new(time_ms: f32, feedback: f32, mix: f32) -> Self {
        // Allocate buffer for up to 2 seconds of delay
        let max_samples = 88200; // 2 seconds at 44.1kHz
        Self {
            time_ms: time_ms.clamp(1.0, 2000.0),
            feedback: feedback.clamp(0.0, 0.95),
            mix: mix.clamp(0.0, 1.0),
            delay_buffer_l: vec![0.0; max_samples],
            delay_buffer_r: vec![0.0; max_samples],
            buffer_pos: 0,
        }
    }
}

impl Default for DelayProcessor {
    fn default() -> Self {
        Self::new(250.0, 0.4, 0.3)
    }
}

impl EffectProcessor for DelayProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let delay_samples = ((self.time_ms / 1000.0) * sample_rate as f32) as usize;
        let delay_samples = delay_samples.min(self.delay_buffer_l.len() - 1);

        for i in (0..samples.len()).step_by(2) {
            let input_l = samples[i];
            let input_r = if i + 1 < samples.len() {
                samples[i + 1]
            } else {
                input_l
            };

            // Read from delayed position
            let read_pos = (self.buffer_pos + self.delay_buffer_l.len() - delay_samples)
                % self.delay_buffer_l.len();
            let delayed_l = self.delay_buffer_l[read_pos];
            let delayed_r = self.delay_buffer_r[read_pos];

            // Write to delay buffer with feedback
            self.delay_buffer_l[self.buffer_pos] = input_l + delayed_l * self.feedback;
            self.delay_buffer_r[self.buffer_pos] = input_r + delayed_r * self.feedback;

            // Mix wet and dry
            samples[i] = input_l * (1.0 - self.mix) + delayed_l * self.mix;
            if i + 1 < samples.len() {
                samples[i + 1] = input_r * (1.0 - self.mix) + delayed_r * self.mix;
            }

            self.buffer_pos = (self.buffer_pos + 1) % self.delay_buffer_l.len();
        }
    }

    fn reset(&mut self) {
        self.delay_buffer_l.fill(0.0);
        self.delay_buffer_r.fill(0.0);
        self.buffer_pos = 0;
    }

    fn name(&self) -> &str {
        "Delay"
    }
}
