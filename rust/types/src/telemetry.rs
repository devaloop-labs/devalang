use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub enum TelemetryErrorLevel {
    #[default]
    None,
    Warning,
    Critical,
}

#[derive(Debug)]
pub enum TelemetrySendError {
    Http(String),
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryProjectInfo {
    pub config: Option<TelemetryProjectInfoConfig>,
    pub stats: Option<TelemetryProjectInfoStats>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryProjectInfoStats {
    pub counts: TelemetryProjectInfoStatsCounts,
    pub features: TelemetryProjectInfoStatsFeatures,
    pub audio: TelemetryProjectInfoStatsAudio,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryProjectInfoStatsCounts {
    pub nb_files: usize,
    pub nb_modules: usize,
    pub nb_lines: usize,
    pub nb_banks: usize,
    pub nb_plugins: usize,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryProjectInfoStatsFeatures {
    pub uses_imports: bool,
    pub uses_functions: bool,
    pub uses_groups: bool,
    pub uses_automations: bool,
    pub uses_loops: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryProjectInfoStatsAudio {
    pub avg_bpm: Option<u32>,
    pub has_synths: bool,
    pub has_samples: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryProjectInfoConfig {
    pub entry_defined: bool,
    pub output_defined: bool,
    pub watch_defined: bool,
    pub repeat_defined: bool,
    pub debug_defined: bool,
    pub compress_defined: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct TelemetryEvent {
    pub uuid: String,
    pub cli_version: String,
    pub os: String,
    pub command: Vec<String>,
    pub project_info: Option<TelemetryProjectInfo>,
    pub error_level: TelemetryErrorLevel,
    pub error_message: Option<String>,
    pub exit_code: Option<i32>,
    pub timestamp: String,
    pub duration: u64,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TelemetrySettings {
    pub uuid: String,
    pub stats: bool,
    pub level: String,
    pub enabled: bool,
}
