/// Audio synthesis utilities - oscillators and envelopes
pub mod types;

use std::f32::consts::PI;

/// Generate a single sample from an oscillator
pub fn oscillator_sample(waveform: &str, frequency: f32, time: f32) -> f32 {
    let phase = 2.0 * PI * frequency * time;

    match waveform {
        "sine" => phase.sin(),

        "square" => {
            if phase.sin() >= 0.0 {
                1.0
            } else {
                -1.0
            }
        }

        "saw" => {
            // Sawtooth: -1 to 1
            2.0 * (frequency * time - (frequency * time + 0.5).floor())
        }

        "triangle" => {
            // Triangle wave
            (2.0 * (2.0 * (frequency * time).fract() - 1.0)).abs() * 2.0 - 1.0
        }

        _ => 0.0, // Unknown waveform returns silence
    }
}

/// Calculate ADSR envelope value at sample position
/// Returns amplitude multiplier (0.0 to 1.0)
pub fn adsr_envelope(
    sample_index: usize,
    attack_samples: usize,
    decay_samples: usize,
    sustain_samples: usize,
    release_samples: usize,
    sustain_level: f32,
) -> f32 {
    let attack_end = attack_samples;
    let decay_end = attack_samples + decay_samples;
    let sustain_end = attack_samples + decay_samples + sustain_samples;
    let release_end = attack_samples + decay_samples + sustain_samples + release_samples;

    if sample_index < attack_end && attack_samples > 0 {
        // Attack phase: 0.0 -> 1.0
        let progress = sample_index as f32 / attack_samples.max(1) as f32;
        progress
    } else if sample_index < decay_end && decay_samples > 0 {
        // Decay phase: 1.0 -> sustain_level
        let decay_progress = (sample_index - attack_end) as f32 / decay_samples.max(1) as f32;
        1.0 - (1.0 - sustain_level) * decay_progress
    } else if sample_index < sustain_end {
        // Sustain phase: constant at sustain_level
        sustain_level
    } else if sample_index < release_end && release_samples > 0 {
        // Release phase: sustain_level -> 0.0
        let release_progress = (sample_index - sustain_end) as f32 / release_samples.max(1) as f32;
        sustain_level * (1.0 - release_progress).max(0.0)
    } else {
        // After release, silence
        0.0
    }
}

/// Convert time in seconds to samples
pub fn time_to_samples(time_seconds: f32, sample_rate: u32) -> usize {
    (time_seconds * sample_rate as f32) as usize
}

/// Convert MIDI note to frequency in Hz
pub fn midi_to_frequency(midi_note: u8) -> f32 {
    440.0 * 2.0_f32.powf((midi_note as f32 - 69.0) / 12.0)
}

#[cfg(test)]
#[path = "test_synth.rs"]
mod tests;
