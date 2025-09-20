use serde::{ Deserialize, Serialize };
use toml::Value as TomlValue;

use crate::TelemetrySettings;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginExport {
    pub name: String,
    pub kind: String,
    /// Optional default value provided by the plugin manifest (keeps toml::Value to preserve types)
    #[serde(default)]
    pub default: Option<TomlValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginInfo {
    pub author: String,
    pub name: String,
    /// Optional version string when available from a plugin file
    #[serde(default)]
    pub version: Option<String>,
    /// Optional human-friendly description
    #[serde(default)]
    pub description: Option<String>,
    pub exports: Vec<PluginExport>,
}

// --- Config types (centralised) ------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectConfig {
    pub defaults: ProjectConfigDefaults,
    pub banks: Option<Vec<ProjectConfigBankEntry>>,
    pub plugins: Option<Vec<ProjectConfigPluginEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectConfigDefaults {
    pub entry: Option<String>,
    pub output: Option<String>,
    pub watch: Option<bool>,
    pub repeat: Option<bool>,
    pub debug: Option<bool>,
    pub compress: Option<bool>,
    pub sample_rate: Option<u32>,
    /// Preferred audio format for exports (e.g. "wav16", "wav24", "wav32").
    /// Stored as a string to avoid cross-crate enum coupling.
    #[serde(default)]
    pub audio_format: Option<String>,

    /// Preferred output formats (e.g. ["wav", "mid"]).
    /// Stored as a list of strings for flexibility and backwards compatibility.
    #[serde(default)]
    pub output_format: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectConfigBankEntry {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProjectConfigPluginEntry {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PluginEntry {
    pub path: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub access: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserSettings {
    pub session: String,
    pub telemetry: TelemetrySettings,
}
