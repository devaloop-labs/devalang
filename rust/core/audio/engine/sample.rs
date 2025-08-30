use crate::core::{store::variable::VariableTable, utils::path::normalize_path};
use devalang_types::Value;
use rodio::{Decoder, Source};
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

impl super::synth::AudioEngine {
    pub fn insert_sample(
        &mut self,
        filepath: &str,
        time_secs: f32,
        dur_sec: f32,
        effects: Option<HashMap<String, Value>>,
        variable_table: &VariableTable,
    ) {
        if filepath.is_empty() {
            eprintln!("❌ Empty file path provided for audio sample.");
            return;
        }

        let module_root = Path::new(&self.module_name);
        let root = match devalang_utils::path::get_project_root() {
            Ok(p) => p,
            Err(_) => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        };
        let resolved_path: String;

        let mut var_path = filepath.to_string();
        if let Some(Value::String(variable_path)) = variable_table.variables.get(filepath) {
            var_path = variable_path.clone();
        } else if let Some(Value::Sample(sample_path)) = variable_table.variables.get(filepath) {
            var_path = sample_path.clone();
        }

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
            // Rejoin the remainder as the entity path so bank entries can contain
            // nested paths like "subdir/sample.wav" or plain names.
            let entity_name = parts[2..].join("/");

            let deva_dir = match devalang_utils::path::get_deva_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    eprintln!("❌ {}", e);
                    return;
                }
            };
            let subdir = match obj_type {
                "bank" => "banks",
                "plugin" => "plugins",
                "preset" => "presets",
                "template" => "templates",
                other => other,
            };

            // Determine the bank audio base directory. Prefer an optional
            // `audioPath` declared in the bank's bank.toml (supports keys
            // `audioPath` or `audio_path`). If absent, fall back to `audio/`.
            let mut audio_dir = deva_dir.join(subdir).join(bank_name).join("audio");
            // Try to read bank.toml to get audioPath
            let bank_toml = deva_dir.join(subdir).join(bank_name).join("bank.toml");
            if bank_toml.exists() {
                if let Ok(content) = std::fs::read_to_string(&bank_toml) {
                    if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                        if let Some(ap) = parsed
                            .get("audioPath")
                            .or_else(|| parsed.get("audio_path"))
                            .and_then(|v| v.as_str())
                        {
                            // normalize separators
                            let ap_norm = ap.replace("\\", "/");
                            audio_dir = deva_dir.join(subdir).join(bank_name).join(ap_norm);
                        }
                    }
                }
            }
            // Force looking into the computed audio_dir. If the entity_name
            // already contains an extension (e.g. .wav/.mp3) or a nested path,
            // preserve it as-is. Otherwise, try with a .wav extension.
            let bank_base = audio_dir;
            let candidate = bank_base.join(&entity_name);

            if candidate.exists() {
                resolved_path = candidate.to_string_lossy().to_string();
            } else {
                // Detect whether the provided entity already includes an extension.
                let has_extension = std::path::Path::new(&entity_name).extension().is_some();

                if !has_extension {
                    // Try appending .wav as a fallback for shorthand names without extension
                    let wav_candidate = bank_base.join(format!("{}.wav", entity_name));
                    if wav_candidate.exists() {
                        resolved_path = wav_candidate.to_string_lossy().to_string();
                    } else {
                        // Last resort: use the legacy location (no audio/), also with .wav
                        resolved_path = deva_dir
                            .join(subdir)
                            .join(bank_name)
                            .join(format!("{}.wav", entity_name))
                            .to_string_lossy()
                            .to_string();
                    }
                } else {
                    // If an extension was specified, don't append .wav; try legacy location
                    let legacy_candidate = deva_dir.join(subdir).join(bank_name).join(&entity_name);

                    if legacy_candidate.exists() {
                        resolved_path = legacy_candidate.to_string_lossy().to_string();
                    } else {
                        // No file found; fall back to the audio candidate path (even if missing)
                        resolved_path = candidate.to_string_lossy().to_string();
                    }
                }
            }
        } else {
            let entry_dir = module_root.parent().unwrap_or(&root);
            let absolute_path = root.join(entry_dir).join(&var_path);

            resolved_path = normalize_path(absolute_path.to_string_lossy().to_string());
        }

        if !Path::new(&resolved_path).exists() {
            eprintln!("❌ Unknown trigger or missing audio file: {}", filepath);
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

        let offset = (time_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;
        let required_len = offset + samples.len() * (CHANNELS as usize);
        if self.buffer.len() < required_len {
            self.buffer.resize(required_len, 0);
        }

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
        effects_map: Option<HashMap<String, Value>>,
    ) {
        let offset = (time_secs * (SAMPLE_RATE as f32) * (CHANNELS as f32)) as usize;
        let total_samples = samples.len();

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

        let delay_samples = if delay > 0.0 {
            (delay * (SAMPLE_RATE as f32)) as usize
        } else {
            0
        };
        let mut delay_buffer: Vec<f32> = vec![0.0; total_samples + delay_samples];

        for i in 0..total_samples {
            let pitch_index = if pitch != 1.0 {
                ((i as f32) / pitch) as usize
            } else {
                i
            };

            let mut adjusted = if pitch_index < total_samples {
                samples[pitch_index] as f32
            } else {
                0.0
            };

            adjusted *= gain;

            if fade_in_samples > 0 && i < fade_in_samples {
                adjusted *= (i as f32) / (fade_in_samples as f32);
            }
            if fade_out_samples > 0 && i >= total_samples.saturating_sub(fade_out_samples) {
                adjusted *= ((total_samples - i) as f32) / (fade_out_samples as f32);
            }

            if drive > 0.0 {
                let normalized = adjusted / (i16::MAX as f32);
                let pre_gain = (10f32).powf(drive / 20.0);
                let driven = (normalized * pre_gain).tanh();
                adjusted = driven * (i16::MAX as f32);
            }

            if delay_samples > 0 && i >= delay_samples {
                let echo = delay_buffer[i - delay_samples] * delay_feedback;
                adjusted += echo;
            }
            if delay_samples > 0 {
                delay_buffer[i] = adjusted;
            }

            if reverb > 0.0 {
                let reverb_delay = (0.03 * (SAMPLE_RATE as f32)) as usize;
                if i >= reverb_delay {
                    adjusted += (self.buffer[offset + i - reverb_delay] as f32) * reverb;
                }
            }

            let adjusted_sample = adjusted.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;

            let (left_gain, right_gain) = crate::core::audio::engine::helpers::pan_gains(pan);

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
