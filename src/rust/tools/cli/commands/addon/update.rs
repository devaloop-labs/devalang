#![cfg(feature = "cli")]

use super::download::download_addon;
use super::metadata::{AddonType, get_addon_from_api, get_addon_publisher_from_api, get_cdn_url};
use anyhow::Result;
use std::fs;

#[derive(serde::Deserialize)]
pub struct AddonVersion {
    pub version: String,
}

/// Retrieves the latest version of an addon from the CDN
async fn fetch_latest_version(addon_type: &AddonType, addon_name: &str) -> Result<AddonVersion> {
    let cdn_url = get_cdn_url();
    let publisher = get_addon_publisher_from_api(addon_name).await?;

    let addon_type_str = addon_type.to_string();

    let url = format!(
        "{}/{}/{}/{}/version",
        cdn_url, addon_type_str, publisher, addon_name
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch version: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch version: HTTP {}",
            response.status()
        ));
    }

    let version: AddonVersion = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse version: {}", e))?;

    Ok(version)
}

/// Reads the local version of an addon from its TOML file
fn read_local_version(addon_path: &std::path::Path, addon_type: &AddonType) -> String {
    let toml_file = match addon_type {
        AddonType::Bank => "bank.toml",
        AddonType::Plugin => "plugin.toml",
        AddonType::Preset => "preset.toml",
        AddonType::Template => "template.toml",
    };

    let toml_path = addon_path.join(toml_file);

    if !toml_path.exists() {
        return String::new();
    }

    if let Ok(content) = fs::read_to_string(&toml_path) {
        if let Ok(value) = toml::from_str::<toml::Value>(&content) {
            let section = match addon_type {
                AddonType::Bank => "bank",
                AddonType::Plugin => "plugin",
                AddonType::Preset => "preset",
                AddonType::Template => "template",
            };

            if let Some(version) = value
                .get(section)
                .and_then(|s| s.get("version"))
                .and_then(|v| v.as_str())
            {
                return version.to_string();
            }
        }
    }

    String::new()
}

/// Updates an addon
pub async fn update_addon(slug: String) -> Result<()> {
    let addon_metadata = get_addon_from_api(&slug).await?;

    let deva_dir = crate::tools::cli::config::path::ensure_deva_dir()?;

    let publisher_and_name = format!("{}.{}", addon_metadata.publisher, addon_metadata.name);

    // Retrieve the latest version
    let latest = fetch_latest_version(&addon_metadata.addon_type, &addon_metadata.name).await?;

    // Determine the addon path
    let addon_dir = match addon_metadata.addon_type {
        AddonType::Bank => deva_dir.join("banks"),
        AddonType::Plugin => deva_dir.join("plugins"),
        AddonType::Preset => deva_dir.join("presets"),
        AddonType::Template => deva_dir.join("templates"),
    };

    let local_path = addon_dir
        .join(&addon_metadata.publisher)
        .join(&addon_metadata.name);

    let local_version = if local_path.exists() {
        read_local_version(&local_path, &addon_metadata.addon_type)
    } else {
        String::new()
    };

    if local_version.is_empty() {
        return Err(anyhow::anyhow!(
            "Addon '{}' is not installed",
            publisher_and_name
        ));
    }

    if local_version == latest.version {
        // Already up-to-date, no action needed
        return Ok(());
    }

    // Remove the old version
    if local_path.exists() {
        fs::remove_dir_all(&local_path)
            .map_err(|e| anyhow::anyhow!("Failed to remove old addon files: {}", e))?;
    }

    // Download the new version
    download_addon(&publisher_and_name, &addon_metadata).await?;

    // Success is handled by caller in mod.rs
    Ok(())
}
