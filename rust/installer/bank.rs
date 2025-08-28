use crate::{
    config::loader::{add_bank_to_config, load_config},
    installer::{
        addon::{AddonType, ask_api_for_signed_url},
        utils::{download_file, extract_archive},
    },
    utils::logger::{LogLevel, Logger},
};
use std::path::{Path, PathBuf};

pub async fn install_bank(name: &str, target_dir: &Path) -> Result<(), String> {
    let logger = Logger::new();

    let signed_url = ask_api_for_signed_url(AddonType::Bank, name).await?;

    let bank_dir = target_dir.join("bank");
    let archive_path = PathBuf::from(format!("./.deva/tmp/{}.devabank", name));
    let extract_path = bank_dir.join(name);

    download_file(&signed_url, &archive_path)
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    if extract_path.exists() {
        logger.log_message(
            LogLevel::Warning,
            &format!(
                "Bank '{}' already exists at '{}'. Skipping install.",
                name,
                extract_path.display()
            ),
        );

        return Ok(());
    }

    // Add the bank to the config
    let root_dir = target_dir
        .parent()
        .ok_or_else(|| "Failed to determine root directory".to_string())?;

    let config_path = root_dir.join(".devalang");
    if !config_path.exists() {
        return Err(format!(
            "Config file not found at '{}'. Please run 'devalang init' before adding an addon",
            config_path.display()
        ));
    }

    let mut config = load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    let dependency_path = &format!("devalang://bank/{}", name);

    extract_archive(&archive_path, &extract_path)
        .await
        .map_err(|e| format!("Failed to extract: {}", e))?;

    add_bank_to_config(&mut config, &extract_path, dependency_path);

    Ok(())
}
