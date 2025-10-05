#![cfg(feature = "cli")]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Devalang user configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// Session token for authentication
    pub session: String,
    /// Telemetry configuration
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// User's unique UUID
    pub uuid: String,
    /// Telemetry enabled or not
    pub enabled: bool,
    /// Telemetry level
    pub level: String,
    /// Statistics enabled or not
    pub stats: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            session: String::new(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            uuid: uuid::Uuid::new_v4().to_string(),
            enabled: false,
            level: "basic".to_string(),
            stats: false,
        }
    }
}

/// Gets the user's home directory
pub fn get_home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// Gets the Devalang configuration directory
pub fn get_devalang_homedir() -> PathBuf {
    if let Some(home_dir) = get_home_dir() {
        home_dir.join(".devalang")
    } else {
        PathBuf::from("~/.devalang")
    }
}

/// Gets the path to the user configuration file
pub fn get_user_config_path() -> PathBuf {
    get_devalang_homedir().join("config.json")
}

/// Loads the user configuration
pub fn get_user_config() -> Option<UserConfig> {
    let config_path = get_user_config_path();

    if !config_path.exists() {
        return None;
    }

    let file = fs::File::open(config_path).ok()?;
    serde_json::from_reader(file).ok()
}

/// Saves the user configuration
pub fn write_user_config(config: &UserConfig) -> Result<()> {
    let config_path = get_user_config_path();

    // Create parent directory if necessary
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Atomic write via temporary file
    let tmp_path = config_path.with_extension("json.tmp");
    let config_json = serde_json::to_string_pretty(config)?;

    let mut tmp_file = fs::File::create(&tmp_path)?;
    tmp_file.write_all(config_json.as_bytes())?;
    tmp_file.sync_all()?;

    fs::rename(&tmp_path, config_path)?;

    Ok(())
}

/// Ensures the configuration file exists
pub fn ensure_user_config_exists() -> Result<()> {
    let config_path = get_user_config_path();

    if !config_path.exists() {
        let config = UserConfig::default();
        write_user_config(&config)?;
    }

    Ok(())
}

/// Updates the session token
pub fn set_session_token(token: String) -> Result<()> {
    let mut config = get_user_config().unwrap_or_default();
    config.session = token;
    write_user_config(&config)?;
    Ok(())
}

/// Gets the session token
pub fn get_session_token() -> Option<String> {
    get_user_config().and_then(|cfg| {
        let token = cfg.session.trim();
        if token.is_empty() {
            None
        } else {
            Some(token.to_string())
        }
    })
}

/// Removes the session token (logout)
pub fn clear_session_token() -> Result<()> {
    let mut config = get_user_config().unwrap_or_default();
    config.session = String::new();
    write_user_config(&config)?;
    Ok(())
}
