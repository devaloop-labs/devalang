use crate::{
    common::api::get_api_url,
    config::{driver::ProjectConfig, settings::get_user_config, stats::ProjectStats},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Default, Clone)]
pub enum TelemetryErrorLevel {
    #[default]
    None,
    Warning,
    Critical,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TelemetryProjectInfo {
    config: Option<TelemetryProjectInfoConfig>,
    stats: Option<TelemetryProjectInfoStats>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TelemetryProjectInfoStats {
    counts: TelemetryProjectInfoStatsCounts,
    features: TelemetryProjectInfoStatsFeatures,
    audio: TelemetryProjectInfoStatsAudio,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TelemetryProjectInfoStatsCounts {
    pub nb_files: usize,
    pub nb_modules: usize,
    pub nb_lines: usize,
    pub nb_banks: usize,
    pub nb_plugins: usize,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TelemetryProjectInfoStatsFeatures {
    pub uses_imports: bool,
    pub uses_functions: bool,
    pub uses_groups: bool,
    pub uses_automations: bool,
    pub uses_loops: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TelemetryProjectInfoStatsAudio {
    pub avg_bpm: Option<u32>,
    pub has_synths: bool,
    pub has_samples: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct TelemetryProjectInfoConfig {
    pub entry_defined: bool,
    pub output_defined: bool,
    pub watch_defined: bool,
    pub repeat_defined: bool,
    pub debug_defined: bool,
    pub compress_defined: bool,
}

#[derive(Serialize, Deserialize, Default, Clone)]
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

impl TelemetryEvent {
    pub fn set_timestamp(&mut self, timestamp: String) {
        self.timestamp = timestamp;
    }

    pub fn set_duration(&mut self, duration: u64) {
        self.duration = duration;
    }

    pub fn set_success(&mut self, success: bool) {
        self.success = success;
    }

    pub fn set_error(
        &mut self,
        level: TelemetryErrorLevel,
        message: Option<String>,
        exit_code: Option<i32>,
    ) {
        self.error_level = level;
        self.error_message = message;
        self.exit_code = exit_code;
    }
}

pub struct TelemetryEventCreator {
    pub events: Vec<TelemetryEvent>,
}

impl TelemetryEventCreator {
    pub fn new() -> Self {
        TelemetryEventCreator { events: Vec::new() }
    }

    pub fn create_event(&mut self, event: TelemetryEvent) {
        self.events.push(event.clone());
    }

    pub fn get_base_event(&self) -> TelemetryEvent {
        let mut stats_enabled = false;

        let user_config = get_user_config().unwrap_or_default();

        if user_config.telemetry.stats == true {
            stats_enabled = true;
        }

        let mut event: TelemetryEvent = TelemetryEvent {
            uuid: user_config.telemetry.uuid.clone(),
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            command: std::env::args().collect::<Vec<_>>(),
            project_info: None,
            error_level: TelemetryErrorLevel::None,
            error_message: None,
            exit_code: None,
            timestamp: chrono::Utc::now().to_string(),
            duration: 0,
            success: true,
        };

        let project_settings = ProjectConfig::get();
        let project_stats = ProjectStats::get();

        if project_settings.is_ok() && project_stats.is_ok() {
            let project_settings = project_settings.unwrap();
            let project_stats = project_stats.unwrap();

            let mut stats = None;

            if stats_enabled {
                stats = Some(TelemetryProjectInfoStats {
                    counts: TelemetryProjectInfoStatsCounts {
                        nb_files: project_stats.counts.nb_files,
                        nb_modules: project_stats.counts.nb_modules,
                        nb_lines: project_stats.counts.nb_lines,
                        nb_banks: project_stats.counts.nb_banks,
                        nb_plugins: project_stats.counts.nb_plugins,
                    },
                    features: TelemetryProjectInfoStatsFeatures {
                        uses_imports: project_stats.features.uses_imports,
                        uses_functions: project_stats.features.uses_functions,
                        uses_groups: project_stats.features.uses_groups,
                        uses_automations: project_stats.features.uses_automations,
                        uses_loops: project_stats.features.uses_loops,
                    },
                    audio: TelemetryProjectInfoStatsAudio {
                        avg_bpm: project_stats.audio.avg_bpm,
                        has_synths: project_stats.audio.has_synths,
                        has_samples: project_stats.audio.has_samples,
                    },
                });
            }

            event.project_info = Some(TelemetryProjectInfo {
                config: Some(TelemetryProjectInfoConfig {
                    entry_defined: project_settings.defaults.entry.is_some(),
                    output_defined: project_settings.defaults.output.is_some(),
                    watch_defined: project_settings.defaults.watch.is_some(),
                    repeat_defined: project_settings.defaults.repeat.is_some(),
                    debug_defined: project_settings.defaults.debug.is_some(),
                    compress_defined: project_settings.defaults.compress.is_some(),
                }),
                stats: stats,
            });
        } else {
            event.project_info = None;
        }

        event
    }
}

pub fn refresh_event_project_info(event: &mut TelemetryEvent) {
    let user_config = get_user_config().unwrap_or_default();
    let stats_enabled = user_config.telemetry.stats;

    let project_settings = ProjectConfig::get();
    let project_stats = ProjectStats::get();

    if project_settings.is_ok() && project_stats.is_ok() {
        let project_settings = project_settings.unwrap();
        let project_stats = project_stats.unwrap();

        let mut stats = None;
        if stats_enabled {
            stats = Some(TelemetryProjectInfoStats {
                counts: TelemetryProjectInfoStatsCounts {
                    nb_files: project_stats.counts.nb_files,
                    nb_modules: project_stats.counts.nb_modules,
                    nb_lines: project_stats.counts.nb_lines,
                    nb_banks: project_stats.counts.nb_banks,
                    nb_plugins: project_stats.counts.nb_plugins,
                },
                features: TelemetryProjectInfoStatsFeatures {
                    uses_imports: project_stats.features.uses_imports,
                    uses_functions: project_stats.features.uses_functions,
                    uses_groups: project_stats.features.uses_groups,
                    uses_automations: project_stats.features.uses_automations,
                    uses_loops: project_stats.features.uses_loops,
                },
                audio: TelemetryProjectInfoStatsAudio {
                    avg_bpm: project_stats.audio.avg_bpm,
                    has_synths: project_stats.audio.has_synths,
                    has_samples: project_stats.audio.has_samples,
                },
            });
        }

        event.project_info = Some(TelemetryProjectInfo {
            config: Some(TelemetryProjectInfoConfig {
                entry_defined: project_settings.defaults.entry.is_some(),
                output_defined: project_settings.defaults.output.is_some(),
                watch_defined: project_settings.defaults.watch.is_some(),
                repeat_defined: project_settings.defaults.repeat.is_some(),
                debug_defined: project_settings.defaults.debug.is_some(),
                compress_defined: project_settings.defaults.compress.is_some(),
            }),
            stats,
        });
    } else {
        event.project_info = None;
    }
}

#[derive(Debug)]
pub enum TelemetrySendError {
    Http(String),
}

pub async fn send_telemetry_event(event: &TelemetryEvent) -> Result<(), TelemetrySendError> {
    if let Some(cfg) = get_user_config() {
        if cfg.telemetry.enabled == false {
            return Ok(());
        }
    }

    let telemetry_url = format!("{}/v1/telemetry/send", get_api_url());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| TelemetrySendError::Http(format!("client build error: {}", e)))?;

    let mut last_err: Option<String> = None;
    for (i, delay_ms) in [0u64, 250, 500, 1000].iter().enumerate() {
        if *delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(*delay_ms)).await;
        }

        let res = client
            .post(telemetry_url.clone())
            .json(event)
            .send()
            .await
            .and_then(|r| r.error_for_status());

        match res {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                last_err = Some(err.to_string());

                if i == 3 {
                    break;
                }
            }
        }
    }

    Err(TelemetrySendError::Http(
        last_err.unwrap_or_else(|| "unknown error".to_string()),
    ))
}
