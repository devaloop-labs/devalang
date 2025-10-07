#![cfg(feature = "cli")]

use super::metadata::{AddonType, get_forge_api_url};
use crate::tools::cli::config::user::get_session_token;
use anyhow::Result;

/// Requests a signed URL from the Forge API to download an addon
pub async fn ask_api_for_signed_url(
    addon_type: AddonType,
    publisher: String,
    slug: &str,
) -> Result<String> {
    let forge_api_url = get_forge_api_url();

    // Retrieve authentication token from user config
    let stored_token_opt = get_session_token();

    if stored_token_opt.is_none() {
        return Err(anyhow::anyhow!(
            "Authentication required: run 'devalang login' to authenticate"
        ));
    }

    let kind = addon_type.to_string();

    let request_url = if let Some(token) = &stored_token_opt {
        if publisher.trim().is_empty() {
            format!(
                "{}/v1/addon/url?type={}&slug={}&token={}",
                forge_api_url, kind, slug, token
            )
        } else {
            format!(
                "{}/v1/addon/url?type={}&publisher={}&slug={}&token={}",
                forge_api_url, kind, publisher, slug, token
            )
        }
    } else {
        if publisher.trim().is_empty() {
            format!("{}/v1/addon/url?type={}&slug={}", forge_api_url, kind, slug)
        } else {
            format!(
                "{}/v1/addon/url?type={}&publisher={}&slug={}",
                forge_api_url, kind, publisher, slug
            )
        }
    };

    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = stored_token_opt {
        headers.insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?,
        );
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

    let resp = client
        .get(&request_url)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to receive response: {}", e))?;

    let status = resp.status();
    let body_text = resp
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&body_text)
        .map_err(|_| anyhow::anyhow!("Invalid JSON response (status {}): {}", status, body_text))?;

    let signed_url = json
        .get("payload")
        .and_then(|p| p.get("url"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| anyhow::anyhow!("API returned no URL (status {}): {}", status, body_text))?
        .to_string();

    Ok(signed_url)
}

/// Extracts a .tar.gz addon archive to the appropriate directory
pub fn extract_addon_archive(archive_path: &std::path::Path, deva_dir: &std::path::Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    // Open and decompress the archive
    let file = File::open(archive_path)
        .map_err(|e| anyhow::anyhow!("Failed to open archive {}: {}", archive_path.display(), e))?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    // Detect addon type by inspecting manifest files
    let addon_type = detect_addon_type_from_archive(&mut archive)
        .map_err(|e| anyhow::anyhow!("Failed to detect addon type: {}", e))?;

    // Reopen archive for extraction (Archive consumes the reader)
    let file = File::open(archive_path)
        .map_err(|e| anyhow::anyhow!("Failed to reopen archive: {}", e))?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    // Parse archive filename to get publisher and name
    let filename = archive_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid archive filename"))?;
    
    let stem = filename.trim_end_matches(".tar.gz").trim_end_matches(".tgz");
    let (publisher, name) = if let Some(dot_pos) = stem.find('.') {
        (&stem[..dot_pos], &stem[dot_pos + 1..])
    } else {
        ("unknown", stem)
    };

    // Determine target directory based on addon type
    let type_dir = match addon_type.as_str() {
        "bank" => "banks",
        "plugin" => "plugins",
        "preset" => "presets",
        "template" => "templates",
        _ => return Err(anyhow::anyhow!("Unknown addon type: {}", addon_type)),
    };

    let target_dir = deva_dir.join(type_dir).join(publisher).join(name);
    
    if target_dir.exists() {
        std::fs::remove_dir_all(&target_dir)
            .map_err(|e| anyhow::anyhow!("Failed to remove existing addon directory: {}", e))?;
    }

    std::fs::create_dir_all(&target_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create target directory: {}", e))?;

    // Extract all files
    archive.unpack(&target_dir)
        .map_err(|e| anyhow::anyhow!("Failed to extract archive: {}", e))?;

    Ok(())
}

/// Detects addon type by scanning archive for manifest files
fn detect_addon_type_from_archive(archive: &mut tar::Archive<flate2::read::GzDecoder<std::fs::File>>) -> Result<String> {
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        let path_str = path.to_string_lossy();

        if path_str.contains("bank.toml") {
            return Ok("bank".to_string());
        } else if path_str.contains("plugin.toml") {
            return Ok("plugin".to_string());
        } else if path_str.contains("preset.toml") {
            return Ok("preset".to_string());
        } else if path_str.contains("template.toml") {
            return Ok("template".to_string());
        }
    }

    Ok("unknown".to_string())
}
