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
