#![cfg(feature = "cli")]

use super::metadata::{AddonMetadata, get_cdn_url};
use super::utils::ask_api_for_signed_url;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Downloads a file from the CDN
pub async fn download_from_cdn(url: &str, destination: &Path) -> Result<()> {
    let cdn_url = get_cdn_url();

    if !url.starts_with(&cdn_url) {
        return Err(anyhow::anyhow!("Invalid CDN URL: {}", url));
    }

    let response = reqwest::get(url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to download: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download file: HTTP {}",
            response.status()
        ));
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

    fs::write(destination, bytes).map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;

    Ok(())
}

/// Extracts a tar.gz file
pub fn extract_tar_gz(archive_path: &Path, extract_path: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)
        .map_err(|e| anyhow::anyhow!("Failed to open archive: {}", e))?;

    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    archive
        .unpack(extract_path)
        .map_err(|e| anyhow::anyhow!("Failed to extract tar.gz: {}", e))?;

    Ok(())
}

/// Extracts a zip file safely
pub fn extract_zip_safely(archive_path: &Path, extract_path: &Path) -> Result<()> {
    let file = fs::File::open(archive_path)
        .map_err(|e| anyhow::anyhow!("Failed to open archive: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| anyhow::anyhow!("Failed to read archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| anyhow::anyhow!("Failed to read file from archive: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => extract_path.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)
                        .map_err(|e| anyhow::anyhow!("Failed to create directory: {}", e))?;
                }
            }
            let mut outfile = fs::File::create(&outpath)
                .map_err(|e| anyhow::anyhow!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| anyhow::anyhow!("Failed to extract file: {}", e))?;
        }
    }

    Ok(())
}

/// Downloads and installs an addon
pub async fn download_addon(slug: &str, addon_metadata: &AddonMetadata) -> Result<()> {
    // Get the .deva directory from the current project (not home dir)
    let deva_dir = crate::tools::cli::config::path::ensure_deva_dir()?;

    let target_dir = match addon_metadata.addon_type {
        super::metadata::AddonType::Bank => deva_dir.join("banks"),
        super::metadata::AddonType::Plugin => deva_dir.join("plugins"),
        super::metadata::AddonType::Preset => deva_dir.join("presets"),
        super::metadata::AddonType::Template => deva_dir.join("templates"),
    };

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create target directory: {}", e))?;
    }

    let user_provided_publisher = slug.contains('.');
    let display_name = if user_provided_publisher {
        format!("{}.{}", addon_metadata.publisher, addon_metadata.name)
    } else {
        addon_metadata.name.clone()
    };

    let tmp_root = deva_dir.join("tmp");
    if !tmp_root.exists() {
        fs::create_dir_all(&tmp_root)
            .map_err(|e| anyhow::anyhow!("Failed to create tmp directory: {}", e))?;
    }

    let archive_path = tmp_root.join(&display_name).with_extension("tar.gz");
    let extract_path = target_dir
        .join(&addon_metadata.publisher)
        .join(&addon_metadata.name);

    // Check if addon already exists
    if extract_path.exists() {
        return Ok(());
    }

    // Request signed URL (silent - handled by install command logger)
    let signed_url = if user_provided_publisher {
        ask_api_for_signed_url(
            addon_metadata.addon_type.clone(),
            addon_metadata.publisher.clone(),
            &addon_metadata.name,
        )
        .await?
    } else {
        ask_api_for_signed_url(
            addon_metadata.addon_type.clone(),
            String::new(),
            &addon_metadata.name,
        )
        .await?
    };

    // Download the archive
    download_from_cdn(&signed_url, &archive_path).await?;

    // Extract the archive

    // Detect archive type by extension
    if archive_path.extension().and_then(|s| s.to_str()) == Some("gz") {
        extract_tar_gz(&archive_path, &extract_path)?;
    } else {
        extract_zip_safely(&archive_path, &extract_path)?;
    }

    // Clean up temporary files
    let _ = fs::remove_dir_all(&tmp_root);

    Ok(())
}
