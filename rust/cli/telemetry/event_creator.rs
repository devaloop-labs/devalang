use crate::config::settings::get_user_config;
use devalang_types::{
    TelemetryErrorLevel as SharedTelemetryErrorLevel, TelemetryEvent as SharedTelemetryEvent,
};
use uuid::Uuid;

pub type TelemetryEvent = SharedTelemetryEvent;
pub type TelemetryErrorLevel = SharedTelemetryErrorLevel;

pub trait TelemetryEventExt {
    fn set_timestamp(&mut self, timestamp: String);
    fn set_duration(&mut self, duration: u64);
    fn set_success(&mut self, success: bool);
    fn set_error(
        &mut self,
        level: TelemetryErrorLevel,
        message: Option<String>,
        exit_code: Option<i32>,
    );
}

impl TelemetryEventExt for SharedTelemetryEvent {
    fn set_timestamp(&mut self, timestamp: String) {
        self.timestamp = timestamp;
    }

    fn set_duration(&mut self, duration: u64) {
        self.duration = duration;
    }

    fn set_success(&mut self, success: bool) {
        self.success = success;
    }

    fn set_error(
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
        let uuid = match get_user_config() {
            Some(cfg) if !cfg.telemetry.uuid.is_empty() => cfg.telemetry.uuid.clone(),
            _ => Uuid::new_v4().to_string(),
        };

        TelemetryEvent {
            uuid,
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
        }
    }
}
