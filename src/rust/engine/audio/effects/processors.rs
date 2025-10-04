/// Effect processors - individual effect implementations
use std::f32::consts::PI;

/// Trait for all effect processors
pub trait EffectProcessor: Send + std::fmt::Debug {
    /// Process audio samples (stereo interleaved)
    fn process(&mut self, samples: &mut [f32], sample_rate: u32);

    /// Reset effect state
    fn reset(&mut self);

    /// Get effect name
    fn name(&self) -> &str;
}

/// Chorus effect - multiple detuned voices
#[derive(Debug)]
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

/// Flanger effect - sweeping comb filter
#[derive(Debug)]
pub struct FlangerProcessor {
    depth: f32,
    rate: f32,
    feedback: f32,
    mix: f32,
    phase: f32,
    delay_buffer: Vec<f32>,
    buffer_pos: usize,
}

impl FlangerProcessor {
    pub fn new(depth: f32, rate: f32, feedback: f32, mix: f32) -> Self {
        Self {
            depth: depth.clamp(0.0, 1.0),
            rate: rate.clamp(0.1, 10.0),
            feedback: feedback.clamp(0.0, 0.95),
            mix: mix.clamp(0.0, 1.0),
            phase: 0.0,
            delay_buffer: vec![0.0; 882], // ~20ms at 44.1kHz
            buffer_pos: 0,
        }
    }
}

impl Default for FlangerProcessor {
    fn default() -> Self {
        Self::new(0.7, 0.5, 0.5, 0.5)
    }
}

impl EffectProcessor for FlangerProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let max_delay_samples = (0.010 * sample_rate as f32) as usize; // 10ms max delay

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

                    // Read from delayed position
                    let read_pos = (self.buffer_pos + self.delay_buffer.len() - delay_samples)
                        % self.delay_buffer.len();
                    let delayed = self.delay_buffer[read_pos];

                    // Apply feedback
                    let output = input + delayed * self.feedback;

                    // Write to delay buffer
                    self.delay_buffer[self.buffer_pos] = output;

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
        "Flanger"
    }
}

/// Phaser effect - allpass filter cascade
#[derive(Debug)]
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

/// Compressor effect - dynamic range compression
#[derive(Debug)]
pub struct CompressorProcessor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    envelope: f32,
}

impl CompressorProcessor {
    pub fn new(threshold: f32, ratio: f32, attack: f32, release: f32) -> Self {
        Self {
            threshold,
            ratio: ratio.max(1.0),
            attack: attack.max(0.001),
            release: release.max(0.001),
            envelope: 0.0,
        }
    }
}

impl Default for CompressorProcessor {
    fn default() -> Self {
        Self::new(-20.0, 4.0, 0.005, 0.1)
    }
}

impl EffectProcessor for CompressorProcessor {
    fn process(&mut self, samples: &mut [f32], sample_rate: u32) {
        let attack_coeff = (-1.0 / (self.attack * sample_rate as f32)).exp();
        let release_coeff = (-1.0 / (self.release * sample_rate as f32)).exp();

        for i in (0..samples.len()).step_by(2) {
            // Get stereo sample RMS
            let left = samples[i];
            let right = if i + 1 < samples.len() {
                samples[i + 1]
            } else {
                left
            };
            let rms = ((left * left + right * right) / 2.0).sqrt();

            // Convert to dB
            let db = if rms > 0.0001 {
                20.0 * rms.log10()
            } else {
                -100.0
            };

            // Update envelope
            let target = if db > self.threshold {
                self.threshold + (db - self.threshold) / self.ratio
            } else {
                db
            };

            let coeff = if target > self.envelope {
                attack_coeff
            } else {
                release_coeff
            };

            self.envelope = target + coeff * (self.envelope - target);

            // Calculate gain reduction
            let gain_db = self.envelope - db;
            let gain = 10.0_f32.powf(gain_db / 20.0);

            // Apply gain
            samples[i] *= gain;
            if i + 1 < samples.len() {
                samples[i + 1] *= gain;
            }
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }

    fn name(&self) -> &str {
        "Compressor"
    }
}

/// Distortion effect - waveshaping
#[derive(Debug)]
pub struct DistortionProcessor {
    drive: f32,
    mix: f32,
}

impl DistortionProcessor {
    pub fn new(drive: f32, mix: f32) -> Self {
        Self {
            drive: drive.max(1.0),
            mix: mix.clamp(0.0, 1.0),
        }
    }
}

impl Default for DistortionProcessor {
    fn default() -> Self {
        Self::new(10.0, 0.5)
    }
}

impl EffectProcessor for DistortionProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        for sample in samples.iter_mut() {
            let input = *sample;

            // Apply drive
            let driven = input * self.drive;

            // Soft clipping using tanh
            let distorted = driven.tanh();

            // Mix wet and dry
            *sample = input * (1.0 - self.mix) + distorted * self.mix;
        }
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn name(&self) -> &str {
        "Distortion"
    }
}

/// Delay effect - echo with feedback
#[derive(Debug)]
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

/// Reverb effect - algorithmic reverb using multiple comb and allpass filters
#[derive(Debug)]
pub struct ReverbProcessor {
    room_size: f32,
    damping: f32,
    mix: f32,
    // Comb filters (parallel)
    comb_buffers: Vec<Vec<f32>>,
    comb_positions: Vec<usize>,
    comb_feedback: Vec<f32>,
    // Allpass filters (series)
    allpass_buffers: Vec<Vec<f32>>,
    allpass_positions: Vec<usize>,
}

impl ReverbProcessor {
    pub fn new(room_size: f32, damping: f32, mix: f32) -> Self {
        let room_size = room_size.clamp(0.0, 1.0);
        let damping = damping.clamp(0.0, 1.0);

        // Comb filter delays (in samples at 44.1kHz)
        let comb_delays = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        let mut comb_buffers = Vec::new();
        let mut comb_feedback = Vec::new();

        for &delay in &comb_delays {
            comb_buffers.push(vec![0.0; delay]);
            comb_feedback.push(0.84 + room_size * 0.12);
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
        Self::new(0.5, 0.5, 0.3)
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

                // Apply feedback with damping
                let filtered = delayed * (1.0 - self.damping) + buffer[pos] * self.damping;
                buffer[pos] = input + filtered * self.comb_feedback[j];

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

/// Drive effect - tube-style saturation
#[derive(Debug)]
pub struct DriveProcessor {
    amount: f32,
    tone: f32,
    mix: f32,
}

impl DriveProcessor {
    pub fn new(amount: f32, tone: f32, mix: f32) -> Self {
        Self {
            amount: amount.clamp(0.0, 1.0),
            tone: tone.clamp(0.0, 1.0),
            mix: mix.clamp(0.0, 1.0),
        }
    }
}

impl Default for DriveProcessor {
    fn default() -> Self {
        Self::new(0.5, 0.5, 0.7)
    }
}

impl EffectProcessor for DriveProcessor {
    fn process(&mut self, samples: &mut [f32], _sample_rate: u32) {
        let gain = 1.0 + self.amount * 9.0; // 1x to 10x gain

        for sample in samples.iter_mut() {
            let input = *sample;

            // Apply gain
            let driven = input * gain;

            // Asymmetric soft clipping (tube-like)
            let distorted = if driven > 0.0 {
                driven / (1.0 + driven.abs())
            } else {
                driven / (1.0 + driven.abs() * 1.5) // More compression on negative
            };

            // Simple tone control (lowpass)
            let toned = distorted * self.tone + input * (1.0 - self.tone);

            // Mix wet and dry
            *sample = input * (1.0 - self.mix) + toned * self.mix;
        }
    }

    fn reset(&mut self) {
        // No state to reset
    }

    fn name(&self) -> &str {
        "Drive"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chorus_processor() {
        let mut chorus = ChorusProcessor::default();
        let mut samples = vec![0.5, 0.5, 0.3, 0.3];
        chorus.process(&mut samples, 44100);

        // Should modify samples
        assert!(
            samples
                .iter()
                .any(|&s| (s - 0.5).abs() > 0.001 || (s - 0.3).abs() > 0.001)
        );
    }

    #[test]
    fn test_distortion_processor() {
        let mut distortion = DistortionProcessor::new(5.0, 1.0);
        let mut samples = vec![0.5, 0.5];
        distortion.process(&mut samples, 44100);

        // Should compress/saturate values with tanh
        // tanh(0.5 * 5.0) = tanh(2.5) ≈ 0.987
        // With mix=1.0, output should be close to tanh value
        assert!(samples[0] > 0.5); // Should increase low values
        assert!(samples[0] < 1.0); // But stay below 1.0
        assert!(samples[1] > 0.5);
        assert!(samples[1] < 1.0);
    }

    #[test]
    fn test_compressor_processor() {
        let mut compressor = CompressorProcessor::default();
        let mut samples = vec![0.9, 0.9, 0.1, 0.1]; // High and low levels
        compressor.process(&mut samples, 44100);

        // Compressor should reduce dynamic range
        let range_before = 0.9 - 0.1;
        let range_after = (samples[0] - samples[2]).abs();
        assert!(range_after < range_before);
    }
}
