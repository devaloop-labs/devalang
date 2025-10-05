#![cfg(feature = "cli")]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

fn get_telemetry_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Failed to find home directory");
    home.join(".devalang").join("telemetry.json")
}

pub fn is_telemetry_enabled() -> bool {
    let config_path = get_telemetry_config_path();

    if !config_path.exists() {
        return false;
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str::<TelemetryConfig>(&content) {
            Ok(config) => config.enabled,
            Err(_) => false,
        },
        Err(_) => false,
    }
}

pub fn enable_telemetry() -> Result<()> {
    let config_path = get_telemetry_config_path();

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config = TelemetryConfig { enabled: true };
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, json)?;

    Ok(())
}

pub fn disable_telemetry() -> Result<()> {
    let config_path = get_telemetry_config_path();

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config = TelemetryConfig { enabled: false };
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, json)?;

    Ok(())
}

pub fn get_telemetry_status() -> String {
    if is_telemetry_enabled() {
        "enabled".to_string()
    } else {
        "disabled".to_string()
    }
}
