use crate::installer::addon::{AddonType, install_addon};

/// Handles the installation command for a given addon type and name.
#[cfg(feature = "cli")]
pub async fn handle_install_command(name: String, addon_type: AddonType) -> Result<(), String> {
    use crate::utils::{
        logger::{LogLevel, Logger},
        spinner::with_spinner,
    };
    use std::{thread, time::Duration};

    let logger = Logger::new();
    let deva_dir = std::path::Path::new("./.deva/");

    let spinner = with_spinner("Installing...", || {
        thread::sleep(Duration::from_millis(800));
    });

    if let Err(e) = install_addon(addon_type.clone(), name.as_str(), deva_dir).await {
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
