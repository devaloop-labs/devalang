/// Low-Frequency Oscillator (LFO) module
/// Provides modulation for various parameters (volume, pitch, filter cutoff, pan)
use std::f32::consts::PI;

/// LFO waveform types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoWaveform {
    Sine,
    Triangle,
    Square,
    Saw,
}

impl LfoWaveform {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sine" | "sin" => LfoWaveform::Sine,
            "triangle" | "tri" => LfoWaveform::Triangle,
            "square" | "sq" => LfoWaveform::Square,
            "saw" | "sawtooth" => LfoWaveform::Saw,
            _ => LfoWaveform::Sine,
        }
    }
}

/// LFO rate specification - Hz or tempo-synced
#[derive(Debug, Clone, PartialEq)]
pub enum LfoRate {
    Hz(f32),        // Rate in Hz
    TempoSync(f32), // Rate as fraction of beat (e.g., 1.0 = 1 beat, 0.25 = 1/4 beat)
}

impl LfoRate {
    /// Parse rate from string or number
    /// "4.0" or 4.0 => 4.0 Hz
    /// "1/4" => 1/4 beat (tempo-synced)
    /// "1/8" => 1/8 beat
    pub fn from_value(s: &str) -> Self {
        if s.contains('/') {
            // Parse fraction like "1/4"
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                if let (Ok(num), Ok(denom)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    if denom != 0.0 {
                        return LfoRate::TempoSync(num / denom);
                    }
                }
            }
        }

        // Try to parse as Hz
        s.parse::<f32>()
            .map(LfoRate::Hz)
            .unwrap_or(LfoRate::Hz(1.0))
    }

    /// Convert to Hz given current BPM
    pub fn to_hz(&self, bpm: f32) -> f32 {
        match self {
            LfoRate::Hz(hz) => *hz,
            LfoRate::TempoSync(beats) => {
                // Convert beat fraction to Hz
                // e.g., 1/4 beat at 120 BPM = 120/60 * 4 = 8 Hz
                let beat_hz = bpm / 60.0;
                beat_hz / beats
            }
        }
    }
}

/// LFO parameters
#[derive(Debug, Clone)]
pub struct LfoParams {
    pub rate: LfoRate, // Frequency (Hz or tempo-synced)
    pub depth: f32,    // Modulation depth (0.0-1.0)
    pub waveform: LfoWaveform,
    pub target: LfoTarget, // What parameter to modulate
    pub phase: f32,        // Initial phase (0.0-1.0)
}

/// LFO modulation target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoTarget {
    Volume,
    Pitch,
    FilterCutoff,
    Pan,
}

impl LfoTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "volume" | "vol" | "amp" | "amplitude" => Some(LfoTarget::Volume),
            "pitch" | "frequency" | "freq" => Some(LfoTarget::Pitch),
            "filter" | "cutoff" | "filter_cutoff" => Some(LfoTarget::FilterCutoff),
            "pan" | "panning" => Some(LfoTarget::Pan),
            _ => None,
        }
    }
}

impl Default for LfoParams {
    fn default() -> Self {
        Self {
            rate: LfoRate::Hz(1.0),
            depth: 0.5,
            waveform: LfoWaveform::Sine,
            target: LfoTarget::Volume,
            phase: 0.0,
        }
    }
}

/// Generate LFO value at a specific time
/// Returns a value in the range [-1.0, 1.0]
pub fn generate_lfo_value(params: &LfoParams, time_seconds: f32, bpm: f32) -> f32 {
    let rate_hz = params.rate.to_hz(bpm);
    let phase = (time_seconds * rate_hz + params.phase).fract();

    let raw_value = match params.waveform {
        LfoWaveform::Sine => lfo_sine(phase),
        LfoWaveform::Triangle => lfo_triangle(phase),
        LfoWaveform::Square => lfo_square(phase),
        LfoWaveform::Saw => lfo_saw(phase),
    };

    // Scale by depth
    raw_value * params.depth
}

/// Apply LFO modulation to a base value
/// center_value: the base value to modulate around
/// range: the maximum deviation from center
pub fn apply_lfo_modulation(
    params: &LfoParams,
    time_seconds: f32,
    bpm: f32,
    center_value: f32,
    range: f32,
) -> f32 {
    let lfo_value = generate_lfo_value(params, time_seconds, bpm);
    center_value + (lfo_value * range)
}

// Waveform generators (phase is 0.0-1.0)

fn lfo_sine(phase: f32) -> f32 {
    (2.0 * PI * phase).sin()
}

fn lfo_triangle(phase: f32) -> f32 {
    // Triangle wave: rises from -1 to 1 and back
    4.0 * (phase - 0.5).abs() - 1.0
}

fn lfo_square(phase: f32) -> f32 {
    if phase < 0.5 { 1.0 } else { -1.0 }
}

fn lfo_saw(phase: f32) -> f32 {
    // Sawtooth: rises linearly from -1 to 1
    2.0 * phase - 1.0
}

#[cfg(test)]
#[path = "test_lfo.rs"]
mod tests;
