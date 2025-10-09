//! Common oscillator implementations for plugins.

use super::types::{BufferParams, Waveform};
use std::f32::consts::PI;

/// A simple oscillator for generating waveforms.
pub struct Oscillator {
    waveform: Waveform,
    frequency: f32,
    amplitude: f32,
    phase: f32,
}

impl Oscillator {
    /// Create a new oscillator with the given waveform.
    pub fn new(waveform: Waveform) -> Self {
        Self {
            waveform,
            frequency: 440.0,
            amplitude: 1.0,
            phase: 0.0,
        }
    }

    /// Set the oscillator frequency in Hz.
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
    }

    /// Set the oscillator amplitude (0.0 to 1.0).
    pub fn set_amplitude(&mut self, amp: f32) {
        self.amplitude = amp.clamp(0.0, 1.0);
    }

    /// Set the waveform type.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Reset the oscillator phase to 0.
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Render audio into the output buffer.
    pub fn render(&mut self, out: &mut [f32], params: BufferParams) {
        let phase_inc = self.frequency * 2.0 * PI / params.sample_rate as f32;

        for frame in 0..params.frames {
            let sample = self.generate_sample() * self.amplitude;

            // Write to all channels (interleaved)
            for ch in 0..params.channels {
                let idx = (frame * params.channels + ch) as usize;
                if idx < out.len() {
                    out[idx] = sample.clamp(-1.0, 1.0);
                }
            }

            // Advance phase
            self.phase += phase_inc;
            if self.phase > 2.0 * PI {
                self.phase -= 2.0 * PI;
            }
        }
    }

    /// Generate a single sample based on the current waveform and phase.
    fn generate_sample(&self) -> f32 {
        match self.waveform {
            Waveform::Sine => self.phase.sin(),
            Waveform::Saw => {
                // Sawtooth: -1 to 1 linear ramp
                (self.phase / PI) - 1.0
            }
            Waveform::Square => {
                // Square: -1 or 1
                if self.phase < PI { 1.0 } else { -1.0 }
            }
            Waveform::Triangle => {
                // Triangle: -1 to 1 to -1
                let normalized = self.phase / (2.0 * PI);
                if normalized < 0.5 {
                    4.0 * normalized - 1.0
                } else {
                    3.0 - 4.0 * normalized
                }
            }
        }
    }
}

/// ADSR envelope generator.
pub struct ADSREnvelope {
    attack_samples: u32,
    decay_samples: u32,
    sustain_level: f32,
    release_samples: u32,
    current_sample: u32,
}

impl ADSREnvelope {
    /// Create a new ADSR envelope with times in seconds.
    pub fn new(
        attack_sec: f32,
        decay_sec: f32,
        sustain_level: f32,
        release_sec: f32,
        sample_rate: u32,
    ) -> Self {
        Self {
            attack_samples: (attack_sec * sample_rate as f32) as u32,
            decay_samples: (decay_sec * sample_rate as f32) as u32,
            sustain_level: sustain_level.clamp(0.0, 1.0),
            release_samples: (release_sec * sample_rate as f32) as u32,
            current_sample: 0,
        }
    }

    /// Get the envelope value for the current sample.
    /// Call this for each sample and increment current_sample.
    pub fn get_value(&self, sustain_samples: u32) -> f32 {
        let sample = self.current_sample;

        if sample < self.attack_samples {
            // Attack phase: 0.0 to 1.0
            sample as f32 / self.attack_samples as f32
        } else if sample < self.attack_samples + self.decay_samples {
            // Decay phase: 1.0 to sustain_level
            let decay_progress = (sample - self.attack_samples) as f32 / self.decay_samples as f32;
            1.0 - decay_progress * (1.0 - self.sustain_level)
        } else if sample < self.attack_samples + self.decay_samples + sustain_samples {
            // Sustain phase: sustain_level
            self.sustain_level
        } else {
            // Release phase: sustain_level to 0.0
            let release_start = self.attack_samples + self.decay_samples + sustain_samples;
            let release_progress = (sample - release_start) as f32 / self.release_samples as f32;
            self.sustain_level * (1.0 - release_progress).max(0.0)
        }
    }

    /// Advance to the next sample.
    pub fn next(&mut self) {
        self.current_sample += 1;
    }

    /// Reset the envelope to the beginning.
    pub fn reset(&mut self) {
        self.current_sample = 0;
    }
}

/// Moog Ladder 4-pole low-pass filter (24dB/octave).
/// Classic TB-303 / minimoog filter with self-oscillation.
pub struct LowPassFilter {
    cutoff: f32,
    resonance: f32,
    // 4 cascaded one-pole stages
    stage1: f32,
    stage2: f32,
    stage3: f32,
    stage4: f32,
}

impl LowPassFilter {
    /// Create a new Moog ladder filter.
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self {
            cutoff: cutoff.clamp(20.0, 20000.0),
            resonance: resonance.clamp(0.0, 0.99), // 0.99 max to avoid instability
            stage1: 0.0,
            stage2: 0.0,
            stage3: 0.0,
            stage4: 0.0,
        }
    }

    /// Set the cutoff frequency in Hz.
    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.cutoff = cutoff.clamp(20.0, 20000.0);
    }

    /// Set the resonance (0.0 to 0.99).
    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 0.99);
    }

    /// Process a single sample through the 4-pole ladder.
    pub fn process(&mut self, input: f32, sample_rate: f32) -> f32 {
        // Calculate filter coefficient (cutoff normalized to sample rate)
        let f = 2.0 * self.cutoff / sample_rate;
        let f_clamped = f.clamp(0.0, 1.0); // Prevent aliasing

        // Resonance feedback with compensation
        // At high resonance, the filter self-oscillates
        let fb = self.resonance * 4.0; // 4.0 for strong resonance
        let feedback_signal = self.stage4 * fb;

        // Input with feedback subtracted (negative feedback loop)
        let input_compensated = input - feedback_signal;

        // 4 cascaded one-pole lowpass stages (Moog ladder topology)
        // Each stage: output = output + f * (input - output)
        self.stage1 += f_clamped * (input_compensated - self.stage1);
        self.stage2 += f_clamped * (self.stage1 - self.stage2);
        self.stage3 += f_clamped * (self.stage2 - self.stage3);
        self.stage4 += f_clamped * (self.stage3 - self.stage4);

        // Output is the 4th stage (24dB/octave rolloff)
        self.stage4.clamp(-2.0, 2.0) // Allow some headroom for resonance peaks
    }

    /// Reset the filter state (important between notes!).
    pub fn reset(&mut self) {
        self.stage1 = 0.0;
        self.stage2 = 0.0;
        self.stage3 = 0.0;
        self.stage4 = 0.0;
    }
}
