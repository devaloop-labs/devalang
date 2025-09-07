use devalang_types::Value;
use devalang_types::VariableTable;
use devalang_utils::path::normalize_path;
use rodio::{Decoder, Source};
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

pub fn insert_sample_impl(
    engine: &mut crate::core::audio::engine::driver::AudioEngine,
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

    let module_root = Path::new(&engine.module_name);
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

    // Read frames from decoder and convert to mono if needed.
    let sample_rate = engine.sample_rate as f32;
    let channels = engine.channels as usize;

    let max_frames = (dur_sec * sample_rate) as usize;
    let dec_channels = decoder.channels() as usize;
    let max_raw_samples = max_frames.saturating_mul(dec_channels.max(1));
    let raw_samples: Vec<i16> = decoder.convert_samples().take(max_raw_samples).collect();

    // Convert interleaved channels to mono by averaging channels per frame.
    // Apply a small RMS-preserving scale so mono level is similar to mixed stereo.
    let actual_frames = if dec_channels > 0 {
        raw_samples.len() / dec_channels
    } else {
        0
    };
    let mut samples: Vec<i16> = Vec::with_capacity(actual_frames);
    let rms_scale = (dec_channels as f32).sqrt();
    for frame in 0..actual_frames {
        let mut sum: i32 = 0;
        for ch in 0..dec_channels {
            sum += raw_samples[frame * dec_channels + ch] as i32;
        }
        if dec_channels > 0 {
            let avg = (sum / (dec_channels as i32)) as f32;
            let scaled = (avg * rms_scale).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            samples.push(scaled);
        } else {
            samples.push(0);
        }
    }

    if samples.is_empty() {
        eprintln!("❌ No samples read from {}", resolved_path);
        return;
    }

    let offset = (time_secs * sample_rate * (channels as f32)) as usize;
    let required_len = offset + samples.len() * (channels as usize);
    if engine.buffer.len() < required_len {
        engine.buffer.resize(required_len, 0);
    }

    crate::core::audio::engine::sample::padding::pad_samples_impl(
        engine, &samples, time_secs, effects,
    );
}
