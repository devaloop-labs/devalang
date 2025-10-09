//! Core types for safe plugin development.
//!
//! These types provide safe, idiomatic Rust abstractions for writing plugins.

use serde::{Deserialize, Serialize};

/// Lightweight representation of a musical note.
///
/// Contains all the information needed to render a note in a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// MIDI pitch (0-127), where 60 is middle C (C4)
    pub pitch: u8,
    /// Note velocity (0-127), representing how hard the note was played
    pub velocity: u8,
    /// Duration of the note in milliseconds
    pub duration_ms: u32,
}

impl Default for Note {
    fn default() -> Self {
        Self {
            pitch: 60,     // Middle C
            velocity: 100, // Strong velocity
            duration_ms: 500,
        }
    }
}

impl Note {
    /// Create a new note with the given parameters.
    pub fn new(pitch: u8, velocity: u8, duration_ms: u32) -> Self {
        Self {
            pitch: pitch.min(127),
            velocity: velocity.min(127),
            duration_ms,
        }
    }

    /// Convert MIDI pitch to frequency in Hz.
    ///
    /// Uses the standard formula: f = 440 * 2^((pitch - 69) / 12)
    pub fn frequency(&self) -> f32 {
        440.0 * 2.0_f32.powf((self.pitch as f32 - 69.0) / 12.0)
    }

    /// Get normalized velocity (0.0 to 1.0).
    pub fn velocity_normalized(&self) -> f32 {
        self.velocity as f32 / 127.0
    }

    /// Get duration in seconds.
    pub fn duration_seconds(&self) -> f32 {
        self.duration_ms as f32 / 1000.0
    }
}

/// Parameters describing the audio buffer and rendering context.
///
/// This struct contains all the information needed to render audio correctly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BufferParams {
    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo)
    pub channels: u32,
    /// Number of frames (samples per channel) in the buffer
    pub frames: u32,
}

impl Default for BufferParams {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            frames: 0,
        }
    }
}

impl BufferParams {
    /// Create new buffer parameters.
    pub fn new(sample_rate: u32, channels: u32, frames: u32) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            channels: channels.max(1),
            frames,
        }
    }

    /// Calculate the total buffer length (frames * channels).
    ///
    /// This is the expected length of the interleaved audio buffer.
    pub fn buffer_len(&self) -> usize {
        (self.frames as usize).saturating_mul(self.channels as usize)
    }

    /// Validate that a buffer has the correct length for these parameters.
    pub fn validate_buffer(&self, buffer: &[f32]) -> Result<(), &'static str> {
        let expected = self.buffer_len();
        if buffer.len() < expected {
            Err("output buffer too small for given parameters")
        } else {
            Ok(())
        }
    }

    /// Get the duration of the buffer in seconds.
    pub fn duration_seconds(&self) -> f32 {
        self.frames as f32 / self.sample_rate as f32
    }

    /// Get the duration of the buffer in milliseconds.
    pub fn duration_ms(&self) -> f32 {
        self.duration_seconds() * 1000.0
    }
}

/// Common signature for plugin render functions.
///
/// Plugins implement this signature to generate audio samples.
///
/// # Parameters
///
/// - `out`: Mutable slice to write audio samples (interleaved if stereo)
/// - `params`: Buffer and audio context parameters
/// - `note`: The note being played
/// - `freq`: Frequency in Hz (derived from note pitch)
/// - `amp`: Amplitude (0.0 to 1.0)
pub type RenderFn = fn(out: &mut [f32], params: BufferParams, note: Note, freq: f32, amp: f32);

/// Extended render function signature with additional context.
///
/// This signature provides extra information for complex synthesis scenarios.
///
/// # Additional Parameters
///
/// - `voice_index`: Index of the voice (for polyphonic synths)
/// - `time_ms`: Current playback time in milliseconds
pub type RenderFnExt = fn(
    out: &mut [f32],
    params: BufferParams,
    note: Note,
    freq: f32,
    amp: f32,
    voice_index: u32,
    time_ms: u64,
);

/// Waveform types for oscillators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

impl Waveform {
    /// Parse waveform from numeric value (for plugin parameters)
    pub fn from_f32(value: f32) -> Self {
        match value as i32 {
            0 => Waveform::Sine,
            1 => Waveform::Saw,
            2 => Waveform::Square,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        }
    }
}
