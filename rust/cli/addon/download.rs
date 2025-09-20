use crate::{
    cli::addon::{metadata::AddonToDownloadMetadata, utils::ask_api_for_signed_url},
    config::ops::load_config,
    web::cdn::download_from_cdn,
};
use devalang_core::config::driver::{ProjectConfig, ProjectConfigExt};
use devalang_types::{AddonType, ProjectConfigBankEntry, ProjectConfigPluginEntry};
use devalang_utils::{
    file::extract_zip_safely,
    logger::{LogLevel, Logger},
    spinner::start_spinner,
};
use std::fs;

pub async fn download_addon(
    slug: &str,
    addon_metadata: &AddonToDownloadMetadata,
) -> Result<(), String> {
    let logger = Logger::new();
    let deva_dir = devalang_utils::path::ensure_deva_dir()?;

    let target_dir = match addon_metadata.addon_type {
        AddonType::Bank => deva_dir.join("banks"),
        AddonType::Plugin => deva_dir.join("plugins"),
        AddonType::Preset => deva_dir.join("presets"),
        AddonType::Template => deva_dir.join("templates"),
    };

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir).map_err(|e| {
            format!(
                "Failed to create target dir '{}': {}",
                target_dir.display(),
                e
            )
        })?;
    }

    let user_provided_publisher = slug.contains('.');
    let display_name = if user_provided_publisher {
        format!("{}.{}", addon_metadata.publisher, addon_metadata.name)
    } else {
        addon_metadata.name.clone()
    };

    let archive_path = {
        let tmp_root = deva_dir.join("tmp");
        if !tmp_root.exists() {
            fs::create_dir_all(&tmp_root)
                .map_err(|e| format!("Failed to create tmp dir '{}': {}", tmp_root.display(), e))?;
        }
        tmp_root.join(&display_name).with_extension("tar.gz")
    };

    if let Some(parent) = archive_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to prepare tmp dir '{}': {}", parent.display(), e))?;
        }
    }

    let extract_path = target_dir
        .join(&addon_metadata.publisher)
        .join(&addon_metadata.name);

    let signed_url = {
        let spinner =
            start_spinner(format!("Requesting download link for {}", display_name).as_str());
        let request = if user_provided_publisher {
            ask_api_for_signed_url(
                addon_metadata.addon_type.clone(),
                addon_metadata.publisher.clone(),
                &addon_metadata.name,
            )
        } else {
            ask_api_for_signed_url(
                addon_metadata.addon_type.clone(),
                String::new(),
                &addon_metadata.name,
            )
        };

        match request.await {
            Ok(url) => url,
            Err(err) => {
                let message = format!("Failed to obtain download link: {}", err);
                println!("{}", message);
                return Err(message);
            }
        }
    };

    let config_path = devalang_utils::path::get_devalang_config_path()?;
    let mut config = load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    if extract_path.exists() {
        logger.log_message(
            LogLevel::Info,
            format!(
                "Addon '{}' already present at {}",
                display_name.as_str(),
                extract_path.display()
            )
            .as_str(),
        );

        if ensure_config_entry(&mut config, addon_metadata) {
            if let Err(err) = config.write_config(&config) {
                logger.log_message(
                    LogLevel::Error,
                    format!("Failed to write config: {}", err).as_str(),
                );
            }
        }

        if matches!(
            addon_metadata.addon_type,
            AddonType::Preset | AddonType::Template
        ) {
            logger.log_message(
                LogLevel::Info,
                "Presets and templates are not tracked in project config yet.",
            );
        }

        return Ok(());
    }

    let download_spinner = start_spinner("Downloading archive...");
    match download_from_cdn(&signed_url, &archive_path).await {
        Ok(_) => println!("Downloaded archive to {}", archive_path.display()),
        Err(err) => {
            let message = format!("Failed to download archive: {}", err);
            println!("{}", message);
            return Err(message);
        }
    }

    let extract_spinner = start_spinner("Extracting archive");
    match extract_zip_safely(&archive_path, &extract_path) {
        Ok(_) => println!("Installed at {}", extract_path.display()),
        Err(err) => {
            println!("Failed to extract archive: {}", err);
            return Err(err);
        }
    }

    let mut config_updated = false;
    if ensure_config_entry(&mut config, addon_metadata) {
        match config.write_config(&config) {
            Ok(_) => {
                config_updated = true;
            }
            Err(err) => {
                logger.log_message(
                    LogLevel::Error,
                    format!("Failed to write config: {}", err).as_str(),
                );
            }
        }
    }

    logger.log_message(
        LogLevel::Info,
        format!(
            "Addon '{}' installed at {}",
            display_name,
            extract_path.display()
        )
        .as_str(),
    );

    if config_updated {
        logger.log_message(LogLevel::Info, "Project config updated");
    } else if matches!(
        addon_metadata.addon_type,
        AddonType::Preset | AddonType::Template
    ) {
        logger.log_message(
            LogLevel::Info,
            "Presets and templates are not tracked in project config yet.",
        );
    }

    // Cleanup temporary files used during download/install
    if let Err(err) = devalang_utils::file::clear_tmp_folder() {
        logger.log_message(
            LogLevel::Warning,
            format!("Failed to clear tmp folder: {}", err).as_str(),
        );
    }

    Ok(())
}

fn ensure_config_entry(
    config: &mut ProjectConfig,
    addon_metadata: &AddonToDownloadMetadata,
) -> bool {
    match addon_metadata.addon_type {
        AddonType::Bank => {
            let dependency_path = format!(
                "devalang://bank/{}/{}",
                addon_metadata.publisher, addon_metadata.name
            );
            let banks = config.banks.get_or_insert_with(Vec::new);
            if banks.iter().any(|entry| entry.path == dependency_path) {
                false
            } else {
                banks.push(ProjectConfigBankEntry {
                    path: dependency_path,
                });
                true
            }
        }
        AddonType::Plugin => {
            let dependency_path = format!(
                "devalang://plugin/{}/{}",
                addon_metadata.publisher, addon_metadata.name
            );
            let plugins = config.plugins.get_or_insert_with(Vec::new);
            if plugins.iter().any(|entry| entry.path == dependency_path) {
                false
            } else {
                plugins.push(ProjectConfigPluginEntry {
                    path: dependency_path,
                });
                true
            }
        }
        AddonType::Preset | AddonType::Template => false,
    }
}
