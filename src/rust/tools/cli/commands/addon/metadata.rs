#![cfg(feature = "cli")]

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct AddonMetadata {
    pub name: String,
    pub publisher: String,
    pub addon_type: AddonType,
}

#[derive(Debug, Clone)]
pub enum AddonType {
    Bank,
    Plugin,
    Preset,
    Template,
}

impl std::fmt::Display for AddonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddonType::Bank => write!(f, "bank"),
            AddonType::Plugin => write!(f, "plugin"),
            AddonType::Preset => write!(f, "preset"),
            AddonType::Template => write!(f, "template"),
        }
    }
}

/// Gets the API URL from environment variables
pub fn get_api_url() -> String {
    std::env::var("DEVALANG_API_URL").unwrap_or_else(|_| "https://api.devalang.com".to_string())
}

/// Gets the CDN URL from environment variables
pub fn get_cdn_url() -> String {
    std::env::var("DEVALANG_CDN_URL").unwrap_or_else(|_| "https://cdn.devalang.com".to_string())
}

/// Gets the Forge API URL from environment variables
pub fn get_forge_api_url() -> String {
    std::env::var("DEVALANG_FORGE_URL").unwrap_or_else(|_| "https://forge.devalang.com".to_string())
}

/// Retrieves addon metadata from the API
pub async fn get_addon_from_api(slug: &str) -> Result<AddonMetadata> {
    let api_url = get_api_url();
    let request_url = format!("{}/v1/products/getBySlug/{}", api_url, slug);

    let client = reqwest::Client::builder()
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

    let payload = json
        .get("payload")
        .ok_or_else(|| anyhow::anyhow!("No payload in response"))?;

    let addon_type_str = payload
        .get("addon")
        .and_then(|a| a.get("addon_type"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing addon_type"))?;

    let addon_type = match addon_type_str {
        "bank" => AddonType::Bank,
        "plugin" => AddonType::Plugin,
        "preset" => AddonType::Preset,
        "template" => AddonType::Template,
        _ => return Err(anyhow::anyhow!("Unknown addon type: {}", addon_type_str)),
    };

    let name = payload
        .get("addon")
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing addon name"))?
        .to_string();

    let publisher = payload
        .get("publisher")
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing publisher name"))?
        .to_string();

    Ok(AddonMetadata {
        name,
        publisher,
        addon_type,
    })
}

/// Retrieves the publisher of an addon from the API
pub async fn get_addon_publisher_from_api(slug: &str) -> Result<String> {
    let api_url = get_api_url();
    let request_url = format!("{}/v1/products/getBySlug/{}", api_url, slug);

    let client = reqwest::Client::builder()
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

    let publisher = json
        .get("payload")
        .and_then(|p| p.get("publisher"))
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing publisher name"))?
        .to_string();

    Ok(publisher)
}
