#![cfg(feature = "cli")]

use super::download::download_addon;
use super::metadata::get_addon_from_api;
use super::utils::extract_addon_archive;
use anyhow::{Context, Result};

/// Installs an addon from remote API or local archive
pub async fn install_addon(slug: String, local: bool, _no_clear_tmp: bool) -> Result<()> {
    if local {
        // Install from local .deva directory
        install_local_addon(&slug).await
    } else {
        // Install from remote API
        let addon_metadata = get_addon_from_api(&slug).await?;
        download_addon(&slug, &addon_metadata).await?;
        Ok(())
    }
}

/// Installs an addon from a local .tar.gz file in .deva directory
async fn install_local_addon(slug: &str) -> Result<()> {
    let deva_dir = crate::tools::cli::config::path::ensure_deva_dir()?;

    // Parse slug to get archive name
    let archive_name = if slug.ends_with(".tar.gz") {
        slug.to_string()
    } else {
        format!("{}.tar.gz", slug)
    };

    let archive_path = deva_dir.join(&archive_name);

    if !archive_path.exists() {
        return Err(anyhow::anyhow!(
            "Local addon archive not found: {}\nTry running 'devalang addon discover --local' to see available local addons.",
            archive_path.display()
        ));
    }

    // Extract and install the archive
    extract_addon_archive(&archive_path, &deva_dir)
        .context("Failed to extract local addon archive")?;

    Ok(())
}
