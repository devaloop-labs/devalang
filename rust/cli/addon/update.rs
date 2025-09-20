use crate::config::ops::load_config;
use crate::{
    cli::addon::{
        download::download_addon,
        metadata::{get_addon_from_api, get_addon_publisher_from_api},
    },
    web::cdn::get_cdn_url,
};
use devalang_core::config::driver::ProjectConfigExt;
use devalang_types::AddonType;
use devalang_utils::path as path_utils;
use std::fs;
use toml::Value as TomlValue;

#[derive(serde::Deserialize)]
pub struct AddonVersion {
    pub version: String,
}

async fn fetch_latest_version(
    addon_type: AddonType,
    addon_name: String,
) -> Result<AddonVersion, Box<dyn std::error::Error>> {
    let cdn_url = get_cdn_url();

    let addon_type = match addon_type {
        AddonType::Bank => "bank",
        AddonType::Plugin => "plugin",
        AddonType::Preset => "preset",
        AddonType::Template => "template",
    };

    let publisher_identifier = get_addon_publisher_from_api(&addon_name)
        .await
        .unwrap_or("unknown".to_string());

    let url = format!(
        "{}/{}/{}/{}/version",
        cdn_url, addon_type, publisher_identifier, addon_name
    );

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("❌ Failed to fetch version: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    let version: AddonVersion = serde_json::from_slice(&bytes)?;

    Ok(version)
}

pub async fn update_addon(slug: String) -> Result<(), String> {
    let addon_metadata = get_addon_from_api(&slug).await?;
    let deva_dir = path_utils::ensure_deva_dir()?;

    // Represent the addon as "<publisher>.<addon>" for user-visible names
    let publisher_and_name = format!("{}.{}", addon_metadata.publisher, addon_metadata.name);

    match addon_metadata.addon_type {
        devalang_types::AddonType::Bank => {
            match fetch_latest_version(
                addon_metadata.addon_type.clone(),
                addon_metadata.name.clone(),
            )
            .await
            {
                Ok(latest) => {
                    // Determine local version from bank.toml if available
                    let local_bank_path = deva_dir
                        .join("banks")
                        .join(&addon_metadata.publisher)
                        .join(&addon_metadata.name);
                    let local_version = if local_bank_path.exists() {
                        let bank_toml = local_bank_path.join("bank.toml");
                        if bank_toml.exists() {
                            if let Ok(content) = fs::read_to_string(&bank_toml) {
                                if let Ok(parsed) =
                                    toml::from_str::<devalang_types::BankFile>(&content)
                                {
                                    parsed.bank.version
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };

                    if local_version != latest.version {
                        println!(
                            "Updating bank '{}' from '{}' to '{}'...",
                            publisher_and_name, local_version, latest.version
                        );

                        // remove existing folder if present
                        let bank_dir = deva_dir
                            .join("banks")
                            .join(&addon_metadata.publisher)
                            .join(&addon_metadata.name);
                        if bank_dir.exists() {
                            fs::remove_dir_all(&bank_dir)
                                .map_err(|e| format!("Failed to remove old bank files: {}", e))?;
                        }

                        // download new (use publisher.addon as the external identifier)
                        download_addon(&publisher_and_name, &addon_metadata).await?;

                        // update config version when present
                        if let Ok(config_path) = path_utils::get_devalang_config_path() {
                            if let Some(mut config) = load_config(Some(&config_path)) {
                                if let Some(banks) = config.banks.as_mut() {
                                    for bank in banks.iter_mut() {
                                        let name_in_path = bank
                                            .path
                                            .strip_prefix("devalang://bank/")
                                            .unwrap_or(&bank.path);
                                    }

                                    if let Err(e) = config.write_config(&config) {
                                        eprintln!("Warning: failed to write updated config: {}", e);
                                    }
                                }
                            }
                        }

                        println!(
                            "✅ Bank '{}' updated to version '{}'",
                            publisher_and_name, latest.version
                        );
                    } else {
                        println!(
                            "Bank '{}' is already up-to-date (version {})",
                            publisher_and_name, latest.version
                        );
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Failed to fetch latest version for bank '{}': {}",
                        publisher_and_name, e
                    ));
                }
            }
        }

        devalang_types::AddonType::Plugin => {
            // Try to fetch latest version via CDN API; if available compare, otherwise fallback to redownload
            match fetch_latest_version(
                addon_metadata.addon_type.clone(),
                addon_metadata.name.clone(),
            )
            .await
            {
                Ok(latest) => {
                    // Determine local plugin version by reading plugin.toml from preferred or fallback layout
                    let preferred = deva_dir.join("plugins").join(format!(
                        "{}/{}",
                        addon_metadata.publisher, addon_metadata.name
                    ));
                    let fallback = deva_dir
                        .join("plugins")
                        .join(&addon_metadata.publisher)
                        .join(&addon_metadata.name);

                    let mut local_version = String::new();
                    for candidate in [&preferred, &fallback] {
                        let toml_path = candidate.join("plugin.toml");
                        if toml_path.exists() {
                            if let Ok(content) = fs::read_to_string(&toml_path) {
                                if let Ok(value) = toml::from_str::<TomlValue>(&content) {
                                    if let Some(v) = value
                                        .get("plugin")
                                        .and_then(|p| p.get("version"))
                                        .and_then(|s| s.as_str())
                                    {
                                        local_version = v.to_string();
                                    }
                                }
                            }
                            break;
                        }
                    }

                    if local_version != latest.version {
                        println!(
                            "Updating plugin '{}' from '{}' to '{}'...",
                            publisher_and_name, local_version, latest.version
                        );

                        // remove any existing layout
                        if preferred.exists() {
                            fs::remove_dir_all(&preferred)
                                .map_err(|e| format!("Failed to remove old plugin files: {}", e))?;
                        }
                        if fallback.exists() {
                            fs::remove_dir_all(&fallback)
                                .map_err(|e| format!("Failed to remove old plugin files: {}", e))?;
                        }

                        // download new
                        download_addon(&publisher_and_name, &addon_metadata).await?;

                        // update config version when present
                        if let Ok(config_path) = path_utils::get_devalang_config_path() {
                            if let Some(mut config) = load_config(Some(&config_path)) {
                                if let Some(plugins) = config.plugins.as_mut() {
                                    for p in plugins.iter_mut() {
                                        let name_in_path = p
                                            .path
                                            .strip_prefix("devalang://plugin/")
                                            .unwrap_or(&p.path);
                                    }

                                    if let Err(e) = config.write_config(&config) {
                                        eprintln!("Warning: failed to write updated config: {}", e);
                                    }
                                }
                            }
                        }

                        println!(
                            "✅ Plugin '{}' updated to version '{}'",
                            publisher_and_name, latest.version
                        );
                    } else {
                        println!(
                            "Plugin '{}' is already up-to-date (version {})",
                            publisher_and_name, latest.version
                        );
                    }
                }
                Err(_) => {
                    // Fallback: redownload everything
                    println!(
                        "No version info for plugin '{}', redownloading to ensure latest.",
                        publisher_and_name
                    );

                    let plugin_dir = deva_dir
                        .join("plugins")
                        .join(&addon_metadata.publisher)
                        .join(&addon_metadata.name);
                    if plugin_dir.exists() {
                        fs::remove_dir_all(&plugin_dir)
                            .map_err(|e| format!("Failed to remove old plugin files: {}", e))?;
                    }

                    download_addon(&publisher_and_name, &addon_metadata).await?;

                    // clear version in config (unknown)
                    if let Ok(config_path) = path_utils::get_devalang_config_path() {
                        if let Some(mut config) = load_config(Some(&config_path)) {
                            if let Some(plugins) = config.plugins.as_mut() {
                                for p in plugins.iter_mut() {
                                    let name_in_path = p
                                        .path
                                        .strip_prefix("devalang://plugin/")
                                        .unwrap_or(&p.path);
                                }

                                if let Err(e) = config.write_config(&config) {
                                    eprintln!("Warning: failed to write updated config: {}", e);
                                }
                            }
                        }
                    }

                    println!("✅ Plugin '{}' updated", publisher_and_name);
                }
            }
        }

        devalang_types::AddonType::Preset | devalang_types::AddonType::Template => {
            println!(
                "Update for presets/templates is not yet implemented; reinstalling to be safe."
            );
            let target_dir = match addon_metadata.addon_type {
                devalang_types::AddonType::Preset => deva_dir.join("presets"),
                _ => deva_dir.join("templates"),
            };

            let candidate = target_dir
                .join(&addon_metadata.publisher)
                .join(&addon_metadata.name);
            if candidate.exists() {
                fs::remove_dir_all(&candidate)
                    .map_err(|e| format!("Failed to remove old files: {}", e))?;
            }

            // use publisher.addon representation for user-facing messaging
            download_addon(&publisher_and_name, &addon_metadata).await?;
            println!("✅ Addon '{}' updated (reinstalled)", publisher_and_name);
        }
    }

    Ok(())
}
