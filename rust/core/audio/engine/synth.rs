use devalang_types::Value;
use std::collections::HashMap;

// Sample rate and channel constants used throughout the engine.
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

/// AudioEngine holds the generated interleaved stereo buffer and
/// provides simple utilities to mix/merge buffers and export WAV files.
///
/// Notes:
/// - Buffer is interleaved stereo (L,R,L,R...).
/// - Methods are synchronous and operate on in-memory buffers.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioEngine {
    /// Master volume multiplier (not automatically applied by helpers).
    pub volume: f32,
    /// Interleaved i16 PCM buffer.
    pub buffer: Vec<i16>,
    /// Logical module name used for error traces/diagnostics.
    pub module_name: String,
    /// Simple diagnostic counter for inserted notes.
    pub note_count: usize,
}

impl AudioEngine {
    pub fn new(module_name: String) -> Self {
        AudioEngine {
            volume: 1.0,
            buffer: vec![],
            module_name,
            note_count: 0,
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
        let total_samples = (duration_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;

        if self.buffer.len() < total_samples {
            self.buffer.resize(total_samples, 0);
        }
    }

    pub fn generate_wav_file(&mut self, output_dir: &String) -> Result<(), String> {
        if self.buffer.len() % (CHANNELS as usize) != 0 {
            self.buffer.push(0);
            println!("Completed buffer to respect stereo format.");
        }

        let spec = hound::WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(output_dir, spec)
            .map_err(|e| format!("Error creating WAV file: {}", e))?;

        for sample in &self.buffer {
            writer
                .write_sample(*sample)
                .map_err(|e| format!("Error writing sample: {:?}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Error finalizing WAV: {:?}", e))?;

        Ok(())
    }

    // Insert note moved here from original engine.rs
    pub fn insert_note(
        &mut self,
        waveform: String,
        freq: f32,
        amp: f32,
        start_time_ms: f32,
        duration_ms: f32,
        synth_params: HashMap<String, Value>,
        note_params: HashMap<String, Value>,
        automation: Option<HashMap<String, Value>>,
    ) {
        // Keep internal logic; helpers called from helpers module
        let attack = self.extract_f32(&synth_params, "attack").unwrap_or(0.0);
        let decay = self.extract_f32(&synth_params, "decay").unwrap_or(0.0);
        let sustain = self.extract_f32(&synth_params, "sustain").unwrap_or(1.0);
        let release = self.extract_f32(&synth_params, "release").unwrap_or(0.0);
        let attack_s = if attack > 10.0 {
            attack / 1000.0
        } else {
            attack
        };
        let decay_s = if decay > 10.0 { decay / 1000.0 } else { decay };
        let release_s = if release > 10.0 {
            release / 1000.0
        } else {
            release
        };
        let sustain_level = if sustain > 1.0 {
            (sustain / 100.0).clamp(0.0, 1.0)
        } else {
            sustain.clamp(0.0, 1.0)
        };

        let duration_ms = self
            .extract_f32(&note_params, "duration")
            .unwrap_or(duration_ms);
        let velocity = self.extract_f32(&note_params, "velocity").unwrap_or(1.0);
        let glide = self.extract_boolean(&note_params, "glide").unwrap_or(false);
        let slide = self.extract_boolean(&note_params, "slide").unwrap_or(false);

        let _amplitude = (i16::MAX as f32) * amp.clamp(0.0, 1.0) * velocity.clamp(0.0, 1.0);

        let freq_start = freq;
        let mut freq_end = freq;
        let amp_start = amp * velocity.clamp(0.0, 1.0);
        let mut amp_end = amp_start;

        if glide {
            if let Some(Value::Number(target_freq)) = note_params.get("target_freq") {
                freq_end = *target_freq;
            } else {
                freq_end = freq * 1.5;
            }
        }

        if slide {
            if let Some(Value::Number(target_amp)) = note_params.get("target_amp") {
                amp_end = *target_amp * velocity.clamp(0.0, 1.0);
            } else {
                amp_end = amp_start * 0.5;
            }
        }

        let sample_rate = SAMPLE_RATE as f32;
        let channels = CHANNELS as usize;

        let total_samples = ((duration_ms / 1000.0) * sample_rate) as usize;
        let start_sample = ((start_time_ms / 1000.0) * sample_rate) as usize;

        let (volume_env, pan_env, pitch_env) =
            crate::core::audio::engine::helpers::env_maps_from_automation(&automation);

        let mut stereo_samples: Vec<i16> = Vec::with_capacity(total_samples * 2);
        let fade_len = (sample_rate * 0.01) as usize; // 10 ms fade

        let attack_samples = (attack_s * sample_rate) as usize;
        let decay_samples = (decay_s * sample_rate) as usize;
        let release_samples = (release_s * sample_rate) as usize;
        let sustain_samples = if total_samples > attack_samples + decay_samples + release_samples {
            total_samples - attack_samples - decay_samples - release_samples
        } else {
            0
        };

        for i in 0..total_samples {
            let t = ((start_sample + i) as f32) / sample_rate;

            // Glide
            let current_freq = if glide {
                freq_start + ((freq_end - freq_start) * (i as f32)) / (total_samples as f32)
            } else {
                freq
            };

            // Pitch automation (in semitones), applied as frequency multiplier
            let pitch_semi = crate::core::audio::engine::helpers::eval_env_map(
                &pitch_env,
                (i as f32) / (total_samples as f32),
                0.0,
            );
            let current_freq = current_freq * (2.0_f32).powf(pitch_semi / 12.0);

            // Slide
            let current_amp = if slide {
                amp_start + ((amp_end - amp_start) * (i as f32)) / (total_samples as f32)
            } else {
                amp_start
            };

            let mut value =
                crate::core::audio::engine::helpers::oscillator_sample(&waveform, current_freq, t);

            // ADSR envelope
            let envelope = crate::core::audio::engine::helpers::adsr_envelope_value(
                i,
                attack_samples,
                decay_samples,
                sustain_samples,
                release_samples,
                sustain_level,
            );

            // Fade in/out
            if fade_len > 0 && i < fade_len {
                if fade_len == 1 {
                    value *= 0.0;
                } else {
                    value *= (i as f32) / (fade_len as f32);
                }
            } else if fade_len > 0 && i >= total_samples.saturating_sub(fade_len) {
                if fade_len == 1 {
                    value *= 0.0;
                } else {
                    // ensure last sample becomes exactly zero to avoid clicks
                    value *= ((total_samples - 1 - i) as f32) / ((fade_len - 1) as f32);
                }
            }

            value *= envelope;
            let mut sample_val = value * (i16::MAX as f32) * current_amp;

            let vol_mul = crate::core::audio::engine::helpers::eval_env_map(
                &volume_env,
                (i as f32) / (total_samples as f32),
                1.0,
            )
            .clamp(0.0, 10.0);
            sample_val *= vol_mul;

            let pan_val = crate::core::audio::engine::helpers::eval_env_map(
                &pan_env,
                (i as f32) / (total_samples as f32),
                0.0,
            )
            .clamp(-1.0, 1.0);
            let (left_gain, right_gain) = crate::core::audio::engine::helpers::pan_gains(pan_val);

            let left = (sample_val * left_gain)
                .round()
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            let right = (sample_val * right_gain)
                .round()
                .clamp(i16::MIN as f32, i16::MAX as f32) as i16;

            stereo_samples.push(left);
            stereo_samples.push(right);
        }

        // Increment note counter for diagnostics
        self.note_count = self.note_count.saturating_add(1);

        crate::core::audio::engine::helpers::mix_stereo_samples_into_buffer(
            self,
            start_sample,
            channels,
            &stereo_samples,
        );
    }

    // helper extraction functions left in this struct for now
    fn extract_f32(&self, map: &HashMap<String, Value>, key: &str) -> Option<f32> {
        match map.get(key) {
            Some(Value::Number(n)) => Some(*n),
            Some(Value::String(s)) => s.parse::<f32>().ok(),
            Some(Value::Boolean(b)) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    fn extract_boolean(&self, map: &HashMap<String, Value>, key: &str) -> Option<bool> {
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
