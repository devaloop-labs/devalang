use crate::{
    cli::install::{bank::install_bank, plugin::install_plugin},
    config::settings::get_user_config,
    web::api::get_api_url,
};
use devalang_types::AddonType;
use std::path::Path;

pub async fn install_addon(
    addon_type: AddonType,
    name: &str,
    target_dir: &Path,
) -> Result<(), String> {
    match addon_type {
        AddonType::Bank => install_bank(name, target_dir).await,
        AddonType::Plugin => install_plugin(name, target_dir).await,
        AddonType::Preset => Err("Preset installation not implemented".into()),
        AddonType::Template => Err("Template installation not implemented".into()),
    }
}

pub async fn ask_api_for_signed_url(addon_type: AddonType, slug: &str) -> Result<String, String> {
    let api_url = get_api_url();

    use devalang_utils::logger::LogLevel;
    use devalang_utils::logger::Logger;

    // Require an authenticated user for addon installation: token must be present and non-empty
    let stored_token_opt = get_user_config().and_then(|cfg| {
        let t = cfg.session.clone();
        if t.trim().is_empty() { None } else { Some(t) }
    });

    if stored_token_opt.is_none() {
        let logger = Logger::new();
        let msg = "Authentication required — run `devalang login` to authenticate";
        logger.log_message(LogLevel::Error, msg);
        return Err("Authentication required: run 'devalang login' to authenticate".to_string());
    }

    let request_url = if let Some(token) = &stored_token_opt {
        format!(
            "{}/v1/assets/url?type={}&slug={}&token={}",
            api_url,
            match addon_type {
                AddonType::Bank => "bank",
                AddonType::Plugin => "plugin",
                AddonType::Preset => "preset",
                AddonType::Template => "template",
            },
            slug,
            token
        )
    } else {
        format!(
            "{}/v1/assets/url?type={}&slug={}",
            api_url,
            match addon_type {
                AddonType::Bank => "bank",
                AddonType::Plugin => "plugin",
                AddonType::Preset => "preset",
                AddonType::Template => "template",
            },
            slug
        )
    };

    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(token) = stored_token_opt {
        headers.insert(
            "Authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
    }

    let client: reqwest::Client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|_| "Failed to build HTTP client".to_string())?;

    let resp = client
        .get(&request_url)
        .send()
        .await
        .map_err(|e| format!("Failed to receive response: {}", e))?;

    let status = resp.status();
    let body_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Try to parse JSON; if parsing fails, return body for diagnostics
    let json: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Invalid JSON response (status {}): {}",
                status, body_text
            ));
        }
    };

    // Extract payload.url safely
    let signed_url_opt = json
        .get("payload")
        .and_then(|p| p.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    if let Some(signed_url) = signed_url_opt {
        Ok(signed_url)
    } else {
        // Provide detailed diagnostics to help user understand why it's null
        let err_msg = format!("API returned no URL (status {}): {}", status, body_text);
        Err(err_msg)
    }
}
