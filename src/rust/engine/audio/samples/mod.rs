//! Sample loading and management for native builds
//!
//! This module provides functionality to load sample banks from disk (TOML manifests + WAV files)
//! and manage a registry of loaded samples for use in audio rendering.

#![cfg(feature = "cli")]

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global sample registry for native builds
static SAMPLE_REGISTRY: Lazy<Arc<Mutex<SampleRegistry>>> =
    Lazy::new(|| Arc::new(Mutex::new(SampleRegistry::new())));

/// Bank manifest structure (from bank.toml)
#[derive(Debug, Deserialize)]
struct BankManifest {
    bank: BankInfo,
    triggers: Vec<TriggerInfo>,
}

#[derive(Debug, Deserialize)]
struct BankInfo {
    name: String,
    publisher: String,
    audio_path: String,
    #[allow(dead_code)]
    description: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    access: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TriggerInfo {
    name: String,
    path: String,
}

/// Sample data (mono f32 PCM)
#[derive(Clone, Debug)]
pub struct SampleData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// Bank metadata for lazy loading
#[derive(Debug, Clone)]
pub struct BankMetadata {
    bank_id: String,
    bank_path: PathBuf,
    audio_path: String,
    triggers: HashMap<String, String>, // trigger_name -> file_path
}

/// Sample registry for managing loaded samples with lazy loading
#[derive(Debug)]
pub struct SampleRegistry {
    samples: HashMap<String, SampleData>,           // Loaded samples cache
    banks: HashMap<String, BankMetadata>,           // Bank metadata for lazy loading
    loaded_samples: HashMap<String, bool>,          // Track which samples are loaded
}

impl SampleRegistry {
    fn new() -> Self {
        Self {
            samples: HashMap::new(),
            banks: HashMap::new(),
            loaded_samples: HashMap::new(),
        }
    }

    /// Register a sample with URI and PCM data (eager loading)
    pub fn register_sample(&mut self, uri: String, data: SampleData) {
        self.samples.insert(uri.clone(), data);
        self.loaded_samples.insert(uri, true);
    }

    /// Register bank metadata for lazy loading
    pub fn register_bank_metadata(&mut self, metadata: BankMetadata) {
        self.banks.insert(metadata.bank_id.clone(), metadata);
    }

    /// Get sample data by URI (lazy load if needed)
    pub fn get_sample(&mut self, uri: &str) -> Option<SampleData> {
        // If already loaded, return from cache
        if let Some(data) = self.samples.get(uri) {
            return Some(data.clone());
        }

        // Try lazy loading
        if !self.loaded_samples.contains_key(uri) {
            if let Some(data) = self.try_lazy_load(uri) {
                self.samples.insert(uri.to_string(), data.clone());
                self.loaded_samples.insert(uri.to_string(), true);
                return Some(data);
            }
            // Mark as attempted (failed to load)
            self.loaded_samples.insert(uri.to_string(), false);
        }

        None
    }

    /// Try to lazy load a sample from bank metadata
    fn try_lazy_load(&self, uri: &str) -> Option<SampleData> {
        // Parse URI: devalang://bank/{bank_id}/{trigger_name}
        if !uri.starts_with("devalang://bank/") {
            return None;
        }

        let path = &uri["devalang://bank/".len()..];
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 2 {
            return None;
        }

        let bank_id = parts[0];
        let trigger_name = parts[1];

        // Find bank metadata
        let bank_meta = self.banks.get(bank_id)?;

        // Find trigger file path
        let file_relative_path = bank_meta.triggers.get(trigger_name)?;

        // Construct full path
        let audio_dir = bank_meta.bank_path.join(&bank_meta.audio_path);
        let wav_path = audio_dir.join(file_relative_path);

        // Load WAV file
        match load_wav_file(&wav_path) {
            Ok(data) => {
                // Lazy loaded sample
                Some(data)
            }
            Err(e) => {
                eprintln!("Failed to lazy load {:?}: {}", wav_path, e);
                None
            }
        }
    }

    /// Check if bank is registered
    pub fn has_bank(&self, bank_id: &str) -> bool {
        self.banks.contains_key(bank_id)
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        let total_banks = self.banks.len();
        let total_samples: usize = self.banks.values().map(|b| b.triggers.len()).sum();
        let loaded_samples = self.samples.len();
        (total_banks, total_samples, loaded_samples)
    }
}

/// Load a bank from a directory containing bank.toml and audio files
/// Uses lazy loading: only metadata is loaded initially, samples are loaded on demand
pub fn load_bank_from_directory(bank_path: &Path) -> Result<String> {
    let manifest_path = bank_path.join("bank.toml");
    if !manifest_path.exists() {
        anyhow::bail!("bank.toml not found in {:?}", bank_path);
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {:?}", manifest_path))?;

    let manifest: BankManifest = toml::from_str(&manifest_content)
        .with_context(|| format!("Failed to parse {:?}", manifest_path))?;

    let bank_id = format!("{}.{}", manifest.bank.publisher, manifest.bank.name);

    // Build trigger map: trigger_name -> file_path
    let mut triggers = HashMap::new();
    for trigger in &manifest.triggers {
        // Clean up trigger path (remove leading ./)
        let clean_path = trigger.path.trim_start_matches("./").to_string();
        triggers.insert(trigger.name.clone(), clean_path);
    }

    // Create bank metadata for lazy loading
    let metadata = BankMetadata {
        bank_id: bank_id.clone(),
        bank_path: bank_path.to_path_buf(),
        audio_path: manifest.bank.audio_path.clone(),
        triggers: triggers.clone(),
    };

    // Register bank metadata
    let mut registry = SAMPLE_REGISTRY.lock().unwrap();
    registry.register_bank_metadata(metadata);

    // Bank registered

    Ok(bank_id)
}

/// Load WAV file and convert to mono f32 PCM
fn load_wav_file(path: &Path) -> Result<SampleData> {
    let bytes = fs::read(path)?;
    
    // Use the common WAV parser
    let parser_result = crate::utils::wav_parser::parse_wav_generic(&bytes)
        .map_err(|e| anyhow::anyhow!("WAV parse error: {}", e))?;

    let (_channels, sample_rate, mono_i16) = parser_result;

    // Convert i16 to f32 normalized [-1.0, 1.0]
    let samples: Vec<f32> = mono_i16.iter().map(|&s| s as f32 / 32768.0).collect();

    Ok(SampleData {
        samples,
        sample_rate,
    })
}

/// Attempt to load an audio file in any supported format.
/// First try the existing WAV parser, then fall back to `rodio::Decoder` which
/// supports MP3/FLAC/OGG and other formats when the CLI feature enables `rodio`.
fn load_audio_file(path: &Path) -> Result<SampleData> {
    // Try WAV parser first (fast, native implementation)
    if let Ok(data) = load_wav_file(path) {
        return Ok(data);
    }

    // Fallback: use rodio decoder (requires the `cli` feature which enables `rodio`)
    // This handles mp3, flac, ogg, and many container formats via Symphonia/rodio.
    use std::fs::File;
    use std::io::BufReader;
    use rodio::Decoder;
    use rodio::Source; // bring trait methods (sample_rate, channels, convert_samples) into scope

    let file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let reader = BufReader::new(file);

    let decoder = Decoder::new(reader).map_err(|e| anyhow::anyhow!("rodio decode error: {}", e))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    // Convert all samples to f32 then to mono if needed.
    let samples_f32: Vec<f32> = decoder.convert_samples::<f32>().collect();

    let mono_f32 = if channels > 1 {
        let ch = channels as usize;
        let frames = samples_f32.len() / ch;
        let mut mono = Vec::with_capacity(frames);
        for f in 0..frames {
            let mut acc = 0.0f32;
            for c in 0..ch {
                acc += samples_f32[f * ch + c];
            }
            mono.push(acc / ch as f32);
        }
        mono
    } else {
        samples_f32
    };

    // Keep samples as f32 (normalized) to match SampleData type
    Ok(SampleData {
        samples: mono_f32,
        sample_rate,
    })
}

/// Get sample from global registry (with lazy loading)
pub fn get_sample(uri: &str) -> Option<SampleData> {
    let mut registry = SAMPLE_REGISTRY.lock().unwrap();
    registry.get_sample(uri)
}

/// Register a sample into the global registry with the given URI.
pub fn register_sample(uri: &str, data: SampleData) {
    let mut registry = SAMPLE_REGISTRY.lock().unwrap();
    registry.register_sample(uri.to_string(), data);
}

/// Convenience: load a WAV file at `path` and register it under an absolute path string URI.
/// Returns the URI used (absolute path) on success.
pub fn register_sample_from_path(path: &std::path::Path) -> Result<String, anyhow::Error> {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let abs_norm = abs.canonicalize().unwrap_or(abs);
    let uri = abs_norm.to_string_lossy().to_string();

    // Load audio file using generic loader (WAV parser first, then fall back to rodio)
    match load_audio_file(&abs_norm) {
        Ok(data) => {
            register_sample(&uri, data);
            Ok(uri)
        }
        Err(e) => Err(e),
    }
}

/// Get registry statistics (banks, total samples, loaded samples)
pub fn get_stats() -> (usize, usize, usize) {
    let registry = SAMPLE_REGISTRY.lock().unwrap();
    registry.stats()
}

/// Auto-discover and load banks from standard locations
pub fn auto_load_banks() -> Result<()> {
    let mut possible_paths = Vec::new();
    
    // 1. Current directory + addons/banks
    if let Ok(cwd) = std::env::current_dir() {
        possible_paths.push(cwd.join("addons").join("banks"));
        possible_paths.push(cwd.join(".deva").join("banks"));
    }
    
    // 2. Home directory + .deva/banks
    if let Some(home_dir) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home_path = PathBuf::from(home_dir);
        possible_paths.push(home_path.join(".deva").join("banks"));
    }
    
    // 3. Parent directories (useful for monorepo structures)
    if let Ok(cwd) = std::env::current_dir() {
        let mut current = cwd.as_path();
        for _ in 0..3 {
            if let Some(parent) = current.parent() {
                possible_paths.push(parent.join("addons").join("banks"));
                possible_paths.push(parent.join("static").join("addons").join("banks"));
                current = parent;
            }
        }
    }

    for base_path in possible_paths {
        if !base_path.exists() {
            continue;
        }

        // Look for bank directories (publisher/bankname)
        if let Ok(publishers) = fs::read_dir(&base_path) {
            for publisher_entry in publishers.filter_map(Result::ok) {
                let publisher_path = publisher_entry.path();
                if !publisher_path.is_dir() {
                    continue;
                }

                if let Ok(banks) = fs::read_dir(&publisher_path) {
                    for bank_entry in banks.filter_map(Result::ok) {
                        let bank_path = bank_entry.path();
                        if !bank_path.is_dir() {
                            continue;
                        }

                        // Try to load this bank
                        if let Err(e) = load_bank_from_directory(&bank_path) {
                            eprintln!("Failed to load bank from {:?}: {}", bank_path, e);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
