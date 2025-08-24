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
        duration_ms: f32,
        synth_params: HashMap<String, Value>,
        note_params: HashMap<String, Value>
    ) {
        let valid_synth_params = vec!["attack", "decay", "sustain", "release"];
        let valid_note_params = vec![
            "duration",
            "velocity",
            "glide",
            "slide",
            "amp",
            "target_freq",
            "target_amp",
            "modulation",
            "expression"
        ];

        // Synth params validation
        for key in synth_params.keys() {
            if !valid_synth_params.contains(&key.as_str()) {
                eprintln!("⚠️ Unknown synth parameter: '{}'", key);
            }
        }

        // Note params validation
        for key in note_params.keys() {
            if !valid_note_params.contains(&key.as_str()) {
                eprintln!("⚠️ Unknown note parameter: '{}'", key);
            }
        }

    // Synth parameters
    let attack = self.extract_f32(&synth_params, "attack").unwrap_or(0.0);
    let decay = self.extract_f32(&synth_params, "decay").unwrap_or(0.0);
    let sustain = self.extract_f32(&synth_params, "sustain").unwrap_or(0.0);
    let release = self.extract_f32(&synth_params, "release").unwrap_or(0.0);
    let attack_s = if attack > 10.0 { attack / 1000.0 } else { attack };
    let decay_s = if decay > 10.0 { decay / 1000.0 } else { decay };
    let release_s = if release > 10.0 { release / 1000.0 } else { release };
    let sustain_level = if sustain > 1.0 { (sustain / 100.0).clamp(0.0, 1.0) } else { sustain.clamp(0.0, 1.0) };

        // Note parameters
    let duration_ms = self.extract_f32(&note_params, "duration").unwrap_or(duration_ms);
    let velocity = self.extract_f32(&note_params, "velocity").unwrap_or(1.0);
        let glide = self.extract_boolean(&note_params, "glide").unwrap_or(false);
        let slide = self.extract_boolean(&note_params, "slide").unwrap_or(false);

    let _amplitude = (i16::MAX as f32) * amp.clamp(0.0, 1.0) * velocity.clamp(0.0, 1.0);

        // Logic for glide and slide
    let freq_start = freq;
        let mut freq_end = freq;
    let amp_start = amp * velocity.clamp(0.0, 1.0);
        let mut amp_end = amp_start;

        if glide {
            if let Some(Value::Number(target_freq)) = note_params.get("target_freq") {
                freq_end = *target_freq;
            } else {
                freq_end = freq * 1.5; // Par défaut, glide vers une quinte
            }
        }

        if slide {
            if let Some(Value::Number(target_amp)) = note_params.get("target_amp") {
                amp_end = *target_amp * velocity.clamp(0.0, 1.0);
            } else {
                amp_end = amp_start * 0.5; // Par défaut, slide vers la moitié
            }
        }

        let sample_rate = SAMPLE_RATE as f32;
        let channels = CHANNELS as usize;

        let total_samples = ((duration_ms / 1000.0) * sample_rate) as usize;
        let start_sample = ((start_time_ms / 1000.0) * sample_rate) as usize;

        let mut samples = Vec::with_capacity(total_samples);
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

            // Slide
            let current_amp = if slide {
                amp_start + ((amp_end - amp_start) * (i as f32)) / (total_samples as f32)
            } else {
                amp_start
            };

            let phase = 2.0 * std::f32::consts::PI * current_freq * t;

            let mut value = match waveform.as_str() {
                "sine" => phase.sin(),
                "square" => if phase.sin() >= 0.0 { 1.0 } else { -1.0 },
                "saw" => 2.0 * (current_freq * t - (current_freq * t + 0.5).floor()),
                "triangle" => (2.0 * (2.0 * (current_freq * t).fract() - 1.0)).abs() * 2.0 - 1.0,
                _ => 0.0,
            };

            // ADSR envelope
            let envelope = if i < attack_samples {
                (i as f32) / (attack_samples as f32)
            } else if i < attack_samples + decay_samples {
                1.0 -
                    (1.0 - sustain_level) * (((i - attack_samples) as f32) / (decay_samples as f32))
            } else if i < attack_samples + decay_samples + sustain_samples {
                sustain_level
            } else {
                if release_samples > 0 {
                    sustain_level *
                        (1.0 -
                            ((i - attack_samples - decay_samples - sustain_samples) as f32) /
                                (release_samples as f32))
                } else {
                    0.0
                }
            };

            // Fade in/out
            if i < fade_len {
                value *= (i as f32) / (fade_len as f32);
            } else if i >= total_samples - fade_len {
                value *= ((total_samples - i) as f32) / (fade_len as f32);
            }

            value *= envelope;
            // Application de l'amplitude dynamique (slide + velocity)
            samples.push((value * (i16::MAX as f32) * current_amp) as i16);
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
            // Debug: note si on rencontre des samples non nuls
            // (pour traquer les buffers silencieux)
            // if i == 0 { eprintln!("[debug] first stereo sample: {}", sample); }
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
    let resolved_path: String;

        // Get the variable path from the variable table
        let mut var_path = filepath.to_string();
        if let Some(Value::String(variable_path)) = variable_table.variables.get(filepath) {
            var_path = variable_path.clone();
        } else if let Some(Value::Sample(sample_path)) = variable_table.variables.get(filepath) {
            var_path = sample_path.clone();
        }

        // Handle devalang:// protocol
        if var_path.starts_with("devalang://") {
            let path_after_protocol = var_path.replace("devalang://", "");
            let parts: Vec<&str> = path_after_protocol.split('/').collect();

            if parts.len() < 3 {
                eprintln!(
                    "❌ Invalid devalang:// path format. Expected devalang://<type>/<author>.<bank>/<entity>"
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
                if s == "true" { Some(true) } else if s == "false" { Some(false) } else { None }
            }
            Some(Value::String(s)) => {
                if s == "true" { Some(true) } else if s == "false" { Some(false) } else { None }
            }
            _ => None,
        }
    }
}
