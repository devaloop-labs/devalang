use crate::{
    cli::install::addon::ask_api_for_signed_url, config::ops::load_config,
    web::cdn::download_from_cdn,
};
use devalang_types::AddonType;
use devalang_utils::{
    logger::{LogLevel, Logger},
    path as path_utils,
};
use std::path::Path;

pub async fn install_bank(name: &str, target_dir: &Path) -> Result<(), String> {
    let logger = Logger::new();

    let signed_url = ask_api_for_signed_url(AddonType::Bank, name).await?;

    let bank_dir = target_dir.join("banks");
    let archive_path = path_utils::ensure_deva_dir()?
        .join("tmp")
        .join(format!("{}.devabank", name));
    let extract_path = bank_dir.join(name);

    download_from_cdn(&signed_url, &archive_path)
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
    let config_path = path_utils::get_devalang_config_path()?;
    let _config = load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    let _dependency_path = &format!("devalang://bank/{}", name);

    devalang_utils::file::extract_zip_safely(&archive_path, &extract_path)
        .map_err(|e| format!("Failed to extract: {}", e))?;

    // TODO: Add the bank to the config

    Ok(())
}
