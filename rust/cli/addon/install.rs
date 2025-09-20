use devalang_utils::logger::Logger;

use crate::cli::addon::{download::download_addon, metadata::get_addon_from_api};

pub async fn install_addon(slug: String, no_clear_tmp: bool) -> Result<(), String> {
    let addon_metadata = get_addon_from_api(&slug).await?;

    if let Err(e) = download_addon(&slug, &addon_metadata).await {
        eprintln!("Failed to download addon '{}': {}", slug, e);
    }

    let logger = Logger::new();
    logger.log_message(
        devalang_utils::logger::LogLevel::Success,
        &format!(
            "Successfully installed addon '{}.{}' ({})",
            addon_metadata.publisher,
            addon_metadata.name,
            match addon_metadata.addon_type {
                devalang_types::AddonType::Bank => "bank",
                devalang_types::AddonType::Plugin => "plugin",
                devalang_types::AddonType::Preset => "preset",
                devalang_types::AddonType::Template => "template",
            }
        ),
    );

    if !no_clear_tmp {
        let _ = devalang_utils::file::clear_tmp_folder();
    }

    Ok(())
}
