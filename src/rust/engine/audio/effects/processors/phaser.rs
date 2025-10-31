use crate::engine::audio::effects::processors::super_trait::EffectProcessor;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct PhaserProcessor {
    stages: usize,
    rate: f32,
    depth: f32,
    feedback: f32,
    mix: f32,
    phase: f32,
    allpass_states: Vec<[f32; 2]>, // [left, right] per stage
}

impl PhaserProcessor {
    pub fn new(stages: usize, rate: f32, depth: f32, feedback: f32, mix: f32) -> Self {
        let stages = stages.clamp(2, 12);
        Self {
            stages,
            rate: rate.clamp(0.1, 10.0),
            depth: depth.clamp(0.0, 1.0),
            feedback: feedback.clamp(0.0, 0.95),
            mix: mix.clamp(0.0, 1.0),
            phase: 0.0,
            allpass_states: vec![[0.0, 0.0]; stages],
        }
    }
}

impl Default for PhaserProcessor {
    fn default() -> Self {
        Self::new(4, 0.5, 0.7, 0.5, 0.5)
    }
}

impl EffectProcessor for PhaserProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        for i in (0..samples.len()).step_by(2) {
            // Update LFO phase
            self.phase += self.rate / sample_rate as f32;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }

            // Calculate allpass coefficient using sine LFO
            let lfo = (2.0 * PI * self.phase).sin();
            let coeff = self.depth * lfo * 0.95; // Range: -0.95 to 0.95

            // Process left and right channels
            for ch in 0..2 {
                if i + ch < samples.len() {
                    let mut signal = samples[i + ch];

                    // Cascade allpass filters
                    for stage in 0..self.stages {
                        let state = self.allpass_states[stage][ch];
                        let output = -signal + coeff * (signal - state);
                        self.allpass_states[stage][ch] = signal;
                        signal = output + state;
                    }

                    // Apply feedback
                    signal = signal * self.feedback;

                    // Mix wet and dry
                    samples[i + ch] = samples[i + ch] * (1.0 - self.mix) + signal * self.mix;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        for state in &mut self.allpass_states {
            state[0] = 0.0;
            state[1] = 0.0;
        }
    }

    fn name(&self) -> &str {
        "Phaser"
    }
}
