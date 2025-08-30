use crate::cli::install::addon::install_addon;
#[cfg(feature = "cli")]
use devalang_types::AddonType;
use devalang_utils::path as path_utils;

/// Handles the installation command for a given addon type and name.
#[cfg(feature = "cli")]
pub async fn handle_install_command(name: String, addon_type: AddonType) -> Result<(), String> {
    use devalang_utils::{
        logger::{LogLevel, Logger},
        spinner::start_spinner,
    };

    let logger = Logger::new();
    let deva_dir = path_utils::ensure_deva_dir()?;

    let spinner = start_spinner("Installing...");

    if let Err(e) = install_addon(addon_type.clone(), name.as_str(), &deva_dir).await {
        spinner.finish_and_clear();
        logger.log_message_with_trace(
            LogLevel::Error,
            &format!("Error installing {:?} '{}'", addon_type, name),
            vec![&e],
        );
    } else {
        spinner.finish_and_clear();
        logger.log_message(
            LogLevel::Success,
            &format!("{:?} '{}' installed successfully!", addon_type, name),
        );
    }

    Ok(())
}
