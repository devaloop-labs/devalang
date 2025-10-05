#![cfg(feature = "cli")]

use super::download::download_addon;
use super::metadata::get_addon_from_api;
use anyhow::Result;

/// Installs an addon
pub async fn install_addon(slug: String, _no_clear_tmp: bool) -> Result<()> {
    let addon_metadata = get_addon_from_api(&slug).await?;

    download_addon(&slug, &addon_metadata).await?;

    // Success is handled by caller in mod.rs
    Ok(())
}
