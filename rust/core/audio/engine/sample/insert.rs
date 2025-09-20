use crate::config::ops::load_config;
use devalang_types::Value;
use devalang_types::VariableTable;
use devalang_utils::path::normalize_path;
use rodio::{Decoder, Source};
use std::path::PathBuf;
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

        // Try both plural and singular folder names (some installs use 'bank' instead of 'banks')
        let singular = if subdir.ends_with('s') {
            &subdir[..subdir.len() - 1]
        } else {
            subdir
        };

        // Build a list of candidate addon roots to support different layouts:
        // - legacy flat: .deva/<subdir>/<publisher>.<name>
        // - nested: .deva/<subdir>/<publisher>/<name>
        // Test both plural and singular folder names.
        let mut candidate_roots: Vec<PathBuf> = Vec::new();
        for sd in &[subdir, singular] {
            let base = deva_dir.join(sd).join(bank_name);
            candidate_roots.push(base.clone());

            if bank_name.contains('.') {
                let mut it = bank_name.splitn(2, '.');
                let pubr = it.next().unwrap_or("");
                let nm = it.next().unwrap_or("");
                candidate_roots.push(deva_dir.join(sd).join(pubr).join(nm));
            }
        }

        // If none of the candidate roots yields the asset, we will also
        // try to lookup referenced addons from the project config.

        // Helper to resolve audio path for a given addon root
        let resolve_from_root = |root: &PathBuf| -> Option<String> {
            // Determine audio dir: prefer audioPath in bank.toml, else audio/
            let mut audio_dir = root.join("audio");
            let bank_toml = root.join("bank.toml");
            if bank_toml.exists() {
                if let Ok(content) = std::fs::read_to_string(&bank_toml) {
                    if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                        if let Some(ap) = parsed
                            .get("audioPath")
                            .or_else(|| parsed.get("audio_path"))
                            .and_then(|v| v.as_str())
                        {
                            let ap_norm = ap.replace("\\", "/");
                            audio_dir = root.join(ap_norm);
                        }
                    }
                }
            }

            let candidate = audio_dir.join(&entity_name);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().to_string());
            }

            let has_extension = std::path::Path::new(&entity_name).extension().is_some();
            if !has_extension {
                let wav_candidate = audio_dir.join(format!("{}.wav", entity_name));
                if wav_candidate.exists() {
                    return Some(wav_candidate.to_string_lossy().to_string());
                }

                let legacy_candidate = root.join(format!("{}.wav", entity_name));
                if legacy_candidate.exists() {
                    return Some(legacy_candidate.to_string_lossy().to_string());
                }
            } else {
                let legacy_candidate = root.join(&entity_name);
                if legacy_candidate.exists() {
                    return Some(legacy_candidate.to_string_lossy().to_string());
                }
            }

            None
        };

        let mut found: Option<String> = None;
        for root in &candidate_roots {
            if let Some(p) = resolve_from_root(root) {
                found = Some(p);
                break;
            }
        }

        // If not found in typical layouts, try to find the addon referenced in the project config
        if found.is_none() {
            if let Ok(config_path) = devalang_utils::path::get_devalang_config_path() {
                if let Some(cfg) = load_config(Some(&config_path)) {
                    // Scan banks or plugins depending on obj_type
                    if obj_type == "bank" {
                        if let Some(banks) = cfg.banks {
                            for b in banks {
                                if let Some(name_in_path) = b.path.strip_prefix("devalang://bank/")
                                {
                                    // match by exact, suffix, or dot notation
                                    if name_in_path == bank_name
                                        || name_in_path.ends_with(bank_name)
                                    {
                                        let root = deva_dir.join(subdir).join(name_in_path);
                                        if let Some(p) = resolve_from_root(&root) {
                                            found = Some(p);
                                            break;
                                        }
                                        // try nested layout
                                        if name_in_path.contains('.') {
                                            let mut it = name_in_path.splitn(2, '.');
                                            let pubr = it.next().unwrap_or("");
                                            let nm = it.next().unwrap_or("");
                                            let root2 = deva_dir.join(subdir).join(pubr).join(nm);
                                            if let Some(p) = resolve_from_root(&root2) {
                                                found = Some(p);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if obj_type == "plugin" {
                        if let Some(plugins) = cfg.plugins {
                            for p in plugins {
                                if let Some(name_in_path) =
                                    p.path.strip_prefix("devalang://plugin/")
                                {
                                    if name_in_path == bank_name
                                        || name_in_path.ends_with(bank_name)
                                    {
                                        let root = deva_dir.join(subdir).join(name_in_path);
                                        if let Some(path_found) = resolve_from_root(&root) {
                                            found = Some(path_found);
                                            break;
                                        }
                                        if name_in_path.contains('.') {
                                            let mut it = name_in_path.splitn(2, '.');
                                            let pubr = it.next().unwrap_or("");
                                            let nm = it.next().unwrap_or("");
                                            let root2 = deva_dir.join(subdir).join(pubr).join(nm);
                                            if let Some(path_found) = resolve_from_root(&root2) {
                                                found = Some(path_found);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(p) = found {
            resolved_path = p;
        } else {
            // Not found; fallback to legacy candidate for error message
            let legacy = deva_dir
                .join(subdir)
                .join(bank_name)
                .join(format!("{}.wav", entity_name));
            resolved_path = legacy.to_string_lossy().to_string();
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
