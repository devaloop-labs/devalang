#![cfg(feature = "cli")]

use crate::tools::cli::config::path::ensure_deva_dir;
use anyhow::Result;
use std::fs;

/// Removes an installed addon
pub async fn remove_addon(slug: String) -> Result<()> {
    let deva_dir = ensure_deva_dir()?;

    // Parse the slug (can be "publisher.name" or just "name")
    let (publisher, addon_name) = if slug.contains('.') {
        let mut parts = slug.splitn(2, '.');
        (
            parts.next().unwrap().to_string(),
            parts.next().unwrap().to_string(),
        )
    } else if slug.contains('/') {
        let mut parts = slug.splitn(2, '/');
        (
            parts.next().unwrap().to_string(),
            parts.next().unwrap().to_string(),
        )
    } else {
        return Err(anyhow::anyhow!(
            "Invalid addon name format. Use 'publisher.name' or 'publisher/name'"
        ));
    };

    // Search in all addon types
    let dirs = ["banks", "plugins", "presets", "templates"];
    let mut found = false;

    for &d in &dirs {
        let folder = deva_dir.join(d);
        if !folder.exists() {
            continue;
        }

        let addon_path = folder.join(&publisher).join(&addon_name);

        if addon_path.exists() {
            fs::remove_dir_all(&addon_path)
                .map_err(|e| anyhow::anyhow!("Failed to remove addon files: {}", e))?;

            // Success is handled by the caller in mod.rs
            found = true;
            break;
        }
    }

    if !found {
        return Err(anyhow::anyhow!(
            "Addon '{}.{}' not found",
            publisher,
            addon_name
        ));
    }

    Ok(())
}
