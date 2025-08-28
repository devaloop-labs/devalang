use crate::config::settings::set_user_config_bool;

#[cfg(feature = "cli")]
pub async fn handle_telemetry_enable_command() -> Result<(), String> {
    set_user_config_bool("telemetry", true);

    println!("Telemetry has been enabled.");

    Ok(())
}

#[cfg(feature = "cli")]
pub async fn handle_telemetry_disable_command() -> Result<(), String> {
    set_user_config_bool("telemetry", false);

    println!("Telemetry has been disabled.");

    Ok(())
}
