use crate::config::settings::set_user_config_value;
use devalang_utils::logger::{LogLevel, Logger};

#[cfg(feature = "cli")]
pub async fn handle_telemetry_enable_command() -> Result<(), String> {
    set_user_config_value("telemetry", serde_json::Value::Bool(true));

    let logger = Logger::new();
    logger.log_message(LogLevel::Info, "Telemetry has been enabled.");

    Ok(())
}

#[cfg(feature = "cli")]
pub async fn handle_telemetry_disable_command() -> Result<(), String> {
    set_user_config_value("telemetry", serde_json::Value::Bool(false));

    let logger = Logger::new();
    logger.log_message(LogLevel::Info, "Telemetry has been disabled.");

    Ok(())
}
