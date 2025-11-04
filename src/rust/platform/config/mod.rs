#![cfg(feature = "cli")]

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use atty;
use inquire;
use serde::{Deserialize, Serialize};

use crate::engine::audio::settings::{AudioBitDepth, AudioChannels, AudioFormat, ResampleQuality};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub project: ProjectSection,
    pub paths: PathsSection,
    pub audio: AudioSection,
    pub live: LiveSection,
    pub rules: RulesSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectSection {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsSection {
    pub entry: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSection {
    #[serde(deserialize_with = "deserialize_format")]
    pub format: Vec<String>,
    pub bit_depth: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub resample_quality: String,
    pub bpm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LiveSection {
    pub crossfade_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RulesSection {
    #[serde(default)]
    pub explicit_durations: RuleLevel,
    #[serde(default)]
    pub deprecated_syntax: RuleLevel,
    #[serde(default)]
    pub var_keyword: RuleLevel,
    #[serde(default)]
    pub missing_duration: RuleLevel,
    #[serde(default)]
    pub implicit_type_conversion: RuleLevel,
    #[serde(default)]
    pub unused_variables: RuleLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleLevel {
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "off")]
    Off,
}

impl Default for RuleLevel {
    fn default() -> Self {
        RuleLevel::Warning
    }
}

impl RuleLevel {
    pub fn should_report(&self) -> bool {
        *self != RuleLevel::Off
    }

    pub fn is_error(&self) -> bool {
        *self == RuleLevel::Error
    }

    pub fn is_warning(&self) -> bool {
        *self == RuleLevel::Warning
    }

    pub fn is_info(&self) -> bool {
        *self == RuleLevel::Info
    }
}

/// Custom deserializer to handle both String and Vec<String> for format field
fn deserialize_format<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::String(s) => Ok(vec![s]),
        Value::Array(arr) => arr
            .into_iter()
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| D::Error::custom("format array must contain strings"))
            })
            .collect(),
        _ => Err(D::Error::custom(
            "format must be a string or array of strings",
        )),
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            project: ProjectSection::default(),
            paths: PathsSection::default(),
            audio: AudioSection::default(),
            live: LiveSection::default(),
            rules: RulesSection::default(),
        }
    }
}

impl Default for ProjectSection {
    fn default() -> Self {
        Self {
            name: "Devalang Project".to_string(),
        }
    }
}

impl Default for PathsSection {
    fn default() -> Self {
        Self {
            entry: PathBuf::from("examples/index.deva"),
            output: PathBuf::from("output"),
        }
    }
}

impl Default for AudioSection {
    fn default() -> Self {
        Self {
            format: vec!["wav".to_string()],
            bit_depth: 16,
            channels: 2,
            sample_rate: 44_100,
            resample_quality: "sinc24".to_string(),
            bpm: 120.0,
        }
    }
}

impl Default for LiveSection {
    fn default() -> Self {
        Self { crossfade_ms: 50 }
    }
}

impl Default for RulesSection {
    fn default() -> Self {
        Self {
            explicit_durations: RuleLevel::Warning,
            deprecated_syntax: RuleLevel::Warning,
            var_keyword: RuleLevel::Error,
            missing_duration: RuleLevel::Info,
            implicit_type_conversion: RuleLevel::Info,
            unused_variables: RuleLevel::Warning,
        }
    }
}

impl AppConfig {
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let json_path = root.join("devalang.json");
        let dot_path = root.join(".devalang");
        let toml_path = root.join("devalang.toml");

        // Collect existing candidate configs
        let mut candidates: Vec<PathBuf> = Vec::new();
        if json_path.exists() {
            candidates.push(json_path.clone());
        }
        if dot_path.exists() {
            candidates.push(dot_path.clone());
        }
        if toml_path.exists() {
            candidates.push(toml_path.clone());
        }

        match candidates.len() {
            0 => {
                let default = AppConfig::default();
                // create devalang.json with defaults for discoverability
                write_default_json(root, &json_path, &default)?;
                Ok(default)
            }
            1 => {
                let path = &candidates[0];
                load_config_by_path(path)
            }
            _ => {
                // Conflict: multiple config files present. Prompt the user to choose.
                // Use interactive prompt (inquire). If prompt fails (non-interactive), fall back to priority order.
                let selected = select_config_interactive(&candidates)
                    .unwrap_or_else(|| pick_config_priority(&candidates));
                load_config_by_path(&selected)
            }
        }
    }

    pub fn entry_path(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(&self.paths.entry)
    }

    pub fn output_path(&self, root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(&self.paths.output)
    }

    pub fn audio_format(&self) -> AudioFormat {
        // Return first format as primary (for backward compatibility)
        self.audio_formats().first().copied().unwrap_or_default()
    }

    pub fn audio_formats(&self) -> Vec<AudioFormat> {
        self.audio
            .format
            .iter()
            .filter_map(|s| AudioFormat::from_str(s))
            .collect()
    }

    pub fn audio_bit_depth(&self) -> AudioBitDepth {
        match self.audio.bit_depth {
            8 => AudioBitDepth::Bit8,
            24 => AudioBitDepth::Bit24,
            32 => AudioBitDepth::Bit32,
            _ => AudioBitDepth::Bit16,
        }
    }

    pub fn audio_channels(&self) -> AudioChannels {
        match self.audio.channels {
            1 => AudioChannels::Mono,
            _ => AudioChannels::Stereo,
        }
    }

    pub fn resample_quality(&self) -> ResampleQuality {
        match self.audio.resample_quality.to_lowercase().as_str() {
            "linear2" | "linear" | "2" => ResampleQuality::Linear2,
            "sinc12" => ResampleQuality::Sinc12,
            "sinc48" => ResampleQuality::Sinc48,
            "sinc96" => ResampleQuality::Sinc96,
            "sinc192" => ResampleQuality::Sinc192,
            "sinc512" => ResampleQuality::Sinc512,
            _ => ResampleQuality::Sinc24,
        }
    }

    pub fn crossfade_ms(&self) -> u64 {
        self.live.crossfade_ms.max(10)
    }

    pub fn sample_rate(&self) -> u32 {
        self.audio.sample_rate.max(8_000)
    }
}

fn load_json(path: &Path) -> Result<AppConfig> {
    let file = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config = serde_json::from_str(&file)
        .with_context(|| format!("invalid JSON config: {}", path.display()))?;
    Ok(config)
}

fn load_toml(path: &Path) -> Result<AppConfig> {
    let file = fs::read_to_string(path)
        .with_context(|| format!("failed to read config: {}", path.display()))?;
    let config = toml::from_str(&file)
        .with_context(|| format!("invalid TOML config: {}", path.display()))?;
    Ok(config)
}

fn load_config_by_path(path: &Path) -> Result<AppConfig> {
    // If filename is exactly ".devalang", try to detect JSON vs TOML by content
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if name == ".devalang" {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("failed to read config: {}", path.display()))?;
            let trimmed = raw.trim_start();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                let cfg = serde_json::from_str(&raw)
                    .with_context(|| format!("invalid JSON config: {}", path.display()))?;
                return Ok(cfg);
            } else {
                let cfg = toml::from_str(&raw)
                    .with_context(|| format!("invalid TOML config: {}", path.display()))?;
                return Ok(cfg);
            }
        }
    }

    // otherwise choose by extension
    if let Some(ext) = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
    {
        match ext.as_str() {
            "json" => load_json(path),
            "toml" => load_toml(path),
            _ => {
                // default: try json then toml
                match load_json(path) {
                    Ok(cfg) => Ok(cfg),
                    Err(_) => load_toml(path),
                }
            }
        }
    } else {
        // default: try json then toml
        match load_json(path) {
            Ok(cfg) => Ok(cfg),
            Err(_) => load_toml(path),
        }
    }
}

fn pick_config_priority(candidates: &[PathBuf]) -> PathBuf {
    // Priority: devalang.toml > devalang.json > .devalang
    for pref in ["devalang.toml", "devalang.json", ".devalang"] {
        if let Some(found) = candidates.iter().find(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .map(|n| n.eq_ignore_ascii_case(pref))
                .unwrap_or(false)
        }) {
            return found.clone();
        }
    }
    // fallback to first
    candidates[0].clone()
}

fn select_config_interactive(candidates: &[PathBuf]) -> Option<PathBuf> {
    // Use inquire crate for interactive selection if available
    // Guard the call so code still works in non-interactive environments
    if atty::is(atty::Stream::Stdin) {
        let options: Vec<String> = candidates
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        // try to use inquire; fall back if unavailable at runtime
        if let Ok(selected) =
            inquire::Select::new("Multiple config files found; select one:", options).prompt()
        {
            return Some(PathBuf::from(selected));
        }
    }
    None
}

fn write_default_json(root: &Path, path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        if parent != root {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory: {}", parent.display())
            })?;
        }
    }
    let json = serde_json::to_string_pretty(config).context("serialize default config")?;
    let mut file = File::create(path)
        .with_context(|| format!("failed to create config file: {}", path.display()))?;
    file.write_all(json.as_bytes())
        .with_context(|| format!("unable to write config file: {}", path.display()))?;
    Ok(())
}
