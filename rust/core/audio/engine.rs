use std::{ collections::HashMap, fs::File, io::BufReader, path::Path };
use hound::{ SampleFormat, WavSpec, WavWriter };
use rodio::{ Decoder, Source };
use crate::core::{
    shared::value::Value,
    store::variable::VariableTable,
    utils::path::normalize_path,
};

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct AudioEngine {
    pub volume: f32,
    pub buffer: Vec<i16>,
    pub module_name: String,
}

impl AudioEngine {
    pub fn new(module_name: String) -> Self {
        AudioEngine {
            volume: 1.0,
            buffer: vec![],
            module_name,
        }
    }

    pub fn get_buffer(&self) -> &[i16] {
        &self.buffer
    }

    pub fn get_normalized_buffer(&self) -> Vec<f32> {
        self.buffer
            .iter()
            .map(|&s| (s as f32) / 32768.0)
            .collect()
    }

    pub fn mix(&mut self, other: &AudioEngine) {
        let max_len = self.buffer.len().max(other.buffer.len());
        self.buffer.resize(max_len, 0);

        for (i, &sample) in other.buffer.iter().enumerate() {
            self.buffer[i] = self.buffer[i].saturating_add(sample);
        }
    }

    pub fn merge_with(&mut self, other: AudioEngine) {
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

        let spec = WavSpec {
            channels: CHANNELS,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_dir, spec).map_err(|e|
            format!("Error creating WAV file: {}", e)
        )?;

        for sample in &self.buffer {
            writer.write_sample(*sample).map_err(|e| format!("Error writing sample: {:?}", e))?;
        }

        writer.finalize().map_err(|e| format!("Error finalizing WAV: {:?}", e))?;

        Ok(())
    }

    pub fn insert_note(
        &mut self,
        waveform: String,
        freq: f32,
        amp: f32,
        start_time_ms: f32,
        duration_ms: f32
    ) {
        let sample_rate = SAMPLE_RATE as f32;
        let channels = CHANNELS as usize;

        let total_samples = ((duration_ms / 1000.0) * sample_rate) as usize;
        let start_sample = ((start_time_ms / 1000.0) * sample_rate) as usize;
        let amplitude = (i16::MAX as f32) * amp.clamp(0.0, 1.0);

        let mut samples = Vec::with_capacity(total_samples);
        let fade_len = (sample_rate * 0.01) as usize; // 10 ms fade

        for i in 0..total_samples {
            let t = ((start_sample + i) as f32) / sample_rate;
            let phase = 2.0 * std::f32::consts::PI * freq * t;

            let mut value = match waveform.as_str() {
                "sine" => phase.sin(),
                "square" => if phase.sin() >= 0.0 { 1.0 } else { -1.0 }
                "saw" => 2.0 * (freq * t - (freq * t + 0.5).floor()),
                "triangle" => (2.0 * (2.0 * (freq * t).fract() - 1.0)).abs() * 2.0 - 1.0,
                _ => 0.0,
            };

            // Fade in/out
            if i < fade_len {
                value *= (i as f32) / (fade_len as f32);
            } else if i >= total_samples - fade_len {
                value *= ((total_samples - i) as f32) / (fade_len as f32);
            }

            samples.push((value * amplitude) as i16);
        }

        // Convert to stereo
        let stereo_samples: Vec<i16> = samples
            .iter()
            .flat_map(|s| vec![*s, *s])
            .collect();

        let offset = start_sample * channels;
        let required_len = offset + stereo_samples.len();

        if self.buffer.len() < required_len {
            self.buffer.resize(required_len, 0);
        }

        for (i, sample) in stereo_samples.iter().enumerate() {
            if *sample != 0 {
            }
            self.buffer[offset + i] = self.buffer[offset + i].saturating_add(*sample);
        }
    }

    pub fn insert_sample(
        &mut self,
        filepath: &str,
        time_secs: f32,
        dur_sec: f32,
        effects: Option<HashMap<String, Value>>,
        variable_table: &VariableTable
    ) {
        if filepath.is_empty() {
            eprintln!("❌ Empty file path provided for audio sample.");
            return;
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let module_root = Path::new(&self.module_name);
        let mut resolved_path = String::new();

        // Get the variable path from the variable table
        let mut var_path = filepath.to_string();
        if let Some(Value::String(variable_path)) = variable_table.variables.get(filepath) {
            var_path = variable_path.clone();
        } else if let Some(Value::Sample(sample_path)) = variable_table.variables.get(filepath) {
            var_path = sample_path.clone();
        }

        // If it's a namespace
        if var_path.contains(".") {
            let parts: Vec<&str> = var_path.trim_start_matches('.').split('.').collect();
            if parts.len() == 2 {
                let bank_name = parts[0];
                let entity_name = parts[1];

                // Verifies if the bank is declared
                if !variable_table.variables.contains_key(bank_name) {
                    eprintln!(
                        "❌ Bank '{}' not declared. Please declare it first using : 'bank {}'",
                        bank_name,
                        bank_name
                    );
                    return;
                }

                resolved_path = root
                    .join(".deva")
                    .join("bank")
                    .join(bank_name)
                    .join(format!("{}.wav", entity_name))
                    .to_string_lossy()
                    .to_string();
            } else {
                eprintln!("❌ Invalid namespace format: {}", var_path);
                return;
            }
        } else if var_path.starts_with("devalang://") {
            let path_after_protocol = var_path.replace("devalang://", "");
            let parts: Vec<&str> = path_after_protocol.split('/').collect();

            if parts.len() < 3 {
                eprintln!(
                    "❌ Invalid devalang:// path format. Expected devalang://<type>/<bank>/<entity>"
                );
                return;
            }

            let obj_type = parts[0];
            let bank_name = parts[1];
            let entity_name = parts[2];

            resolved_path = root
                .join(".deva")
                .join(obj_type)
                .join(bank_name)
                .join(format!("{}.wav", entity_name))
                .to_string_lossy()
                .to_string();
        } else {
            // Else, resolve as a relative path
            let entry_dir = module_root.parent().unwrap_or(root);
            let absolute_path = root.join(entry_dir).join(&var_path);

            resolved_path = normalize_path(absolute_path.to_string_lossy().to_string());
        }

        // Verify if the file exists
        if !Path::new(&resolved_path).exists() {
            eprintln!("❌ Audio file not found at: {}", resolved_path);
            return;
        }

        let file = match File::open(&resolved_path) {
            Ok(f) => BufReader::new(f),
            Err(e) => {
                eprintln!("❌ Failed to open audio file {}: {}", resolved_path, e);
                return;
            }
        };

        let decoder = match Decoder::new(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("❌ Failed to decode audio file {}: {}", resolved_path, e);
                return;
            }
        };

        let max_mono_samples = (dur_sec * (SAMPLE_RATE as f32)) as usize;
        let samples: Vec<i16> = decoder.convert_samples().take(max_mono_samples).collect();

        if samples.is_empty() {
            eprintln!("❌ No samples read from {}", resolved_path);
            return;
        }

        // Calculate buffer offset and size
        let offset = (time_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;
        let required_len = offset + samples.len() * (CHANNELS as usize);
        if self.buffer.len() < required_len {
            self.buffer.resize(required_len, 0);
        }

        // Apply effects and mix
        if let Some(effects_map) = effects {
            self.pad_samples(&samples, time_secs, Some(effects_map));
        } else {
            self.pad_samples(&samples, time_secs, None);
        }
    }

    fn pad_samples(
        &mut self,
        samples: &[i16],
        time_secs: f32,
        effects_map: Option<HashMap<String, Value>>
    ) {
        let offset = (time_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;
        let total_samples = samples.len();

        // Default values
        let mut gain = 1.0;
        let mut pan = 0.0;
        let mut fade_in = 0.0;
        let mut fade_out = 0.0;
        let mut pitch = 1.0;
        let mut drive = 0.0;
        let mut reverb = 0.0;
        let mut delay = 0.0; // delay time in seconds
        let delay_feedback = 0.35; // default feedback

        if let Some(map) = &effects_map {
            for (key, val) in map {
                match (key.as_str(), val) {
                    ("gain", Value::Number(v)) => {
                        gain = *v;
                    }
                    ("pan", Value::Number(v)) => {
                        pan = *v;
                    }
                    ("fadeIn", Value::Number(v)) => {
                        fade_in = *v;
                    }
                    ("fadeOut", Value::Number(v)) => {
                        fade_out = *v;
                    }
                    ("pitch", Value::Number(v)) => {
                        pitch = *v;
                    }
                    ("drive", Value::Number(v)) => {
                        drive = *v;
                    }
                    ("reverb", Value::Number(v)) => {
                        reverb = *v;
                    }
                    ("delay", Value::Number(v)) => {
                        delay = *v;
                    }
                    _ => eprintln!("⚠️ Unknown or invalid effect '{}'", key),
                }
            }
        }

        let fade_in_samples = (fade_in * (SAMPLE_RATE as f32)) as usize;
        let fade_out_samples = (fade_out * (SAMPLE_RATE as f32)) as usize;

        let delay_samples = if delay > 0.0 { (delay * (SAMPLE_RATE as f32)) as usize } else { 0 };
        let mut delay_buffer: Vec<f32> = vec![0.0; total_samples + delay_samples];

        for i in 0..total_samples {
            // PITCH FIRST
            let pitch_index = if pitch != 1.0 { ((i as f32) / pitch) as usize } else { i };

            let mut adjusted = if pitch_index < total_samples {
                samples[pitch_index] as f32
            } else {
                0.0
            };

            // GAIN
            adjusted *= gain;

            // FADE IN/OUT
            if fade_in_samples > 0 && i < fade_in_samples {
                adjusted *= (i as f32) / (fade_in_samples as f32);
            }
            if fade_out_samples > 0 && i >= total_samples.saturating_sub(fade_out_samples) {
                adjusted *= ((total_samples - i) as f32) / (fade_out_samples as f32);
            }

            // DRIVE (soft)
            if drive > 0.0 {
                let normalized = adjusted / (i16::MAX as f32);
                let pre_gain = (10f32).powf(drive / 20.0); // dB mapping
                let driven = (normalized * pre_gain).tanh();
                adjusted = driven * (i16::MAX as f32);
            }

            // DELAY
            if delay_samples > 0 && i >= delay_samples {
                let echo = delay_buffer[i - delay_samples] * delay_feedback;
                adjusted += echo;
            }
            if delay_samples > 0 {
                delay_buffer[i] = adjusted;
            }

            // REVERB
            if reverb > 0.0 {
                let reverb_delay = (0.03 * (SAMPLE_RATE as f32)) as usize;
                if i >= reverb_delay {
                    adjusted += (self.buffer[offset + i - reverb_delay] as f32) * reverb;
                }
            }

            // CLAMP
            let adjusted_sample = adjusted.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

            // PAN
            let left_gain = 1.0 - pan.max(0.0); // Pan > 0 => reduce left
            let right_gain = 1.0 + pan.min(0.0); // Pan < 0 => reduce right

            let left = ((adjusted_sample as f32) * left_gain) as i16;
            let right = ((adjusted_sample as f32) * right_gain) as i16;

            let left_pos = offset + i * 2;
            let right_pos = left_pos + 1;

            if right_pos < self.buffer.len() {
                self.buffer[left_pos] = self.buffer[left_pos].saturating_add(left);
                self.buffer[right_pos] = self.buffer[right_pos].saturating_add(right);
            }
        }
    }
}
