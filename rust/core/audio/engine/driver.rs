use devalang_types::Value;
use std::collections::HashMap;

// Minimal representation of a MIDI note event for export purposes.
#[derive(Debug, Clone, PartialEq)]
pub struct MidiNoteEvent {
    /// MIDI key number 0-127
    pub key: u8,
    /// velocity 0-127
    pub vel: u8,
    /// start time in milliseconds (absolute)
    pub start_ms: u32,
    /// duration in milliseconds
    pub duration_ms: u32,
    /// MIDI channel (0-15)
    pub channel: u8,
}

// Sample rate and channel constants used throughout the engine.
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNELS: u16 = 2;

/// AudioEngine holds the generated interleaved stereo buffer and
/// provides simple utilities to mix/merge buffers and export WAV files.
///
/// Notes:
/// - Buffer is interleaved stereo (L,R,L,R...).
/// - Methods are synchronous and operate on in-memory buffers.
///
#[derive(Debug, Clone, PartialEq)]
pub struct AudioEngine {
    /// Master volume multiplier (not automatically applied by helpers).
    pub volume: f32,
    /// Interleaved i16 PCM buffer.
    pub buffer: Vec<i16>,
    /// Collected MIDI note events for export (non-audio representation).
    pub midi_events: Vec<MidiNoteEvent>,
    /// Map target synth -> last inserted note sample ranges (start_sample, total_samples)
    pub last_notes: std::collections::HashMap<String, Vec<(usize, usize)>>,
    /// Logical module name used for error traces/diagnostics.
    pub module_name: String,
    /// Simple diagnostic counter for inserted notes.
    pub note_count: usize,
    /// Sample rate (can be overridden per-engine)
    pub sample_rate: u32,
    /// Number of channels (interleaved). Defaults to 2.
    pub channels: u16,
}

impl AudioEngine {
    pub fn new(module_name: String) -> Self {
        AudioEngine {
            volume: 1.0,
            buffer: vec![],
            midi_events: Vec::new(),
            module_name,
            note_count: 0,
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            last_notes: std::collections::HashMap::new(),
        }
    }

    pub fn get_buffer(&self) -> &[i16] {
        &self.buffer
    }

    pub fn get_normalized_buffer(&self) -> Vec<f32> {
        self.buffer.iter().map(|&s| (s as f32) / 32768.0).collect()
    }

    pub fn mix(&mut self, other: &AudioEngine) {
        let max_len = self.buffer.len().max(other.buffer.len());
        self.buffer.resize(max_len, 0);

        for (i, &sample) in other.buffer.iter().enumerate() {
            self.buffer[i] = self.buffer[i].saturating_add(sample);
        }
    }

    pub fn merge_with(&mut self, other: AudioEngine) {
        // If the other buffer is empty, simply return without warning (common for spawns that produced nothing)
        if other.buffer.is_empty() {
            return;
        }

        // If the other buffer is present but contains only zeros, warn and skip merge
        if other.buffer.iter().all(|&s| s == 0) {
            eprintln!("⚠️ Skipping merge: other buffer is silent");
            return;
        }

        if self.buffer.iter().all(|&s| s == 0) {
            self.buffer = other.buffer;
            return;
        }

        self.mix(&other);
    }

    pub fn set_duration(&mut self, duration_secs: f32) {
        let total_samples =
            (duration_secs * (self.sample_rate as f32) * (self.channels as f32)) as usize;

        if self.buffer.len() < total_samples {
            self.buffer.resize(total_samples, 0);
        }
    }

    pub fn generate_midi_file(
        &mut self,
        output_path: &String,
        bpm: Option<f32>,
        tpqn: Option<u16>,
    ) -> Result<(), String> {
        crate::core::audio::engine::export::generate_midi_file_impl(
            &self.midi_events,
            output_path,
            bpm,
            tpqn,
        )
    }

    pub fn generate_wav_file(
        &mut self,
        output_dir: &String,
        audio_format: Option<String>,
        sample_rate: Option<u32>,
    ) -> Result<(), String> {
        crate::core::audio::engine::export::generate_wav_file_impl(
            &mut self.buffer,
            output_dir,
            audio_format,
            sample_rate,
        )
    }

    pub fn insert_note(
        &mut self,
        owner: Option<String>,
        waveform: String,
        freq: f32,
        amp: f32,
        start_time_ms: f32,
        duration_ms: f32,
        synth_params: HashMap<String, Value>,
        note_params: HashMap<String, Value>,
        automation: Option<HashMap<String, Value>>,
    ) -> Vec<(usize, usize)> {
        // Delegated implementation lives in notes.rs
        crate::core::audio::engine::notes::insert_note_impl(
            self,
            owner,
            waveform,
            freq,
            amp,
            start_time_ms,
            duration_ms,
            synth_params,
            note_params,
            automation,
        )
    }

    pub fn record_last_note_range(
        &mut self,
        owner: &str,
        start_sample: usize,
        total_samples: usize,
    ) {
        self.last_notes
            .entry(owner.to_string())
            .or_default()
            .push((start_sample, total_samples));
    }

    // helper extraction functions left in this struct for now
    pub(crate) fn extract_f32(&self, map: &HashMap<String, Value>, key: &str) -> Option<f32> {
        match map.get(key) {
            Some(Value::Number(n)) => Some(*n),
            Some(Value::String(s)) => s.parse::<f32>().ok(),
            Some(Value::Boolean(b)) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub(crate) fn extract_boolean(&self, map: &HashMap<String, Value>, key: &str) -> Option<bool> {
        match map.get(key) {
            Some(Value::Boolean(b)) => Some(*b),
            Some(Value::Number(n)) => Some(*n != 0.0),
            Some(Value::Identifier(s)) => {
                if s == "true" {
                    Some(true)
                } else if s == "false" {
                    Some(false)
                } else {
                    None
                }
            }
            Some(Value::String(s)) => {
                if s == "true" {
                    Some(true)
                } else if s == "false" {
                    Some(false)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// Parse simple musical fraction strings like "1/16" into seconds using bpm
pub fn parse_fraction_to_seconds(s: &str, bpm: f32) -> Option<f32> {
    let trimmed = s.trim();
    if let Some((num, den)) = trimmed.split_once('/') {
        if let (Ok(n), Ok(d)) = (num.parse::<f32>(), den.parse::<f32>()) {
            if d != 0.0 {
                let beats = n / d; // e.g. 1/16 -> 0.0625 beats
                let secs_per_beat = 60.0 / bpm.max(1.0);
                return Some(beats * secs_per_beat);
            }
        }
    }
    None
}

// Convert a devalang_types::Duration to seconds using bpm when relevant.
pub fn duration_to_seconds(d: &devalang_types::Duration, bpm: f32) -> Option<f32> {
    use devalang_types::Duration as D;
    match d {
        D::Number(s) => Some(*s),
        D::Beat(frac) | D::Identifier(frac) => parse_fraction_to_seconds(frac, bpm),
        _ => None,
    }
}
