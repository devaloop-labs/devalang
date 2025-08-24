use std::path::Path;
use crate::{
    common::{ api::get_api_url },
    installer::{ bank::install_bank, plugin::install_plugin },
};
use dirs::home_dir;

#[derive(Debug, Clone)]
pub enum AddonType {
    Bank,
    Plugin,
    Preset,
}

pub async fn install_addon(
    addon_type: AddonType,
    name: &str,
    target_dir: &Path
) -> Result<(), String> {
    match addon_type {
        AddonType::Bank => install_bank(name, target_dir).await,
        AddonType::Plugin => install_plugin(name, target_dir).await,
        AddonType::Preset => Err("Preset installation not implemented".into()),
    }
}

pub async fn ask_api_for_signed_url(addon_type: AddonType, slug: &str) -> Result<String, String> {
    let api_url = get_api_url();

    let mut stored_token_path = home_dir().unwrap();
    stored_token_path.push(".devalang");
    stored_token_path.push("session_token.json");

    let stored_token = std::fs::read_to_string(&stored_token_path).unwrap_or_default();

    let request_url = format!(
        "{}/v1/assets/url?type={}&slug={}&token={}",
        api_url,
        match addon_type {
            AddonType::Bank => "bank",
            AddonType::Plugin => "plugin",
            AddonType::Preset => "preset",
        },
        slug,
        stored_token
    );

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert("Authorization", format!("Bearer {}", stored_token).parse().unwrap());

    let client: reqwest::Client = reqwest::Client
        ::builder()
        .default_headers(headers)
        .build()
        .map_err(|_| "Failed to build HTTP client".to_string())?;

    let req = client
        .get(&request_url)
        .send().await
        .map_err(|_| "Failed to receive response".to_string())?;

    let response_body: serde_json::Value = req
        .json().await
        .map_err(|_| "Failed to read response body".to_string())?;

    let signed_url: String = serde_json
        ::from_value(
            response_body
                .get("payload")
                .cloned()
                .unwrap_or_default()
                .get("url")
                .cloned()
                .unwrap_or_default()
        )
        .map_err(|_| "Failed to parse response body".to_string())?;

    Ok(signed_url)
}
