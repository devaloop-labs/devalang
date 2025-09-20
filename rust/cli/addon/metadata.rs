use crate::web::api::get_api_url;

#[derive(Debug, Clone)]
pub struct AddonToDownloadMetadata {
    pub name: String,
    pub publisher: String,
    pub addon_type: devalang_types::AddonType,
}

pub async fn get_addon_publisher_from_api(slug: &str) -> Result<String, String> {
    let api_url = get_api_url();

    let request_url = format!("{}/v1/products/getBySlug/{}", api_url, slug);

    let client: reqwest::Client = reqwest::Client::builder()
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

    let json: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Invalid JSON response (status {}): {}",
                status, body_text
            ));
        }
    };

    let payload = json.get("payload");
    let publisher = payload
        .unwrap_or(&serde_json::Value::Null)
        .get("publisher")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(publisher)
}

pub async fn get_addon_from_api(slug: &str) -> Result<AddonToDownloadMetadata, String> {
    let api_url = get_api_url();

    let request_url = format!("{}/v1/products/getBySlug/{}", api_url, slug);

    let client: reqwest::Client = reqwest::Client::builder()
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

    let json: serde_json::Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!(
                "Invalid JSON response (status {}): {}",
                status, body_text
            ));
        }
    };

    let payload = json.get("payload");
    let addon_type = payload
        .unwrap_or(&serde_json::Value::Null)
        .get("addon")
        .unwrap_or(&serde_json::Value::Null)
        .get("addon_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let addon_type_enum = match addon_type {
        "bank" => devalang_types::AddonType::Bank,
        "plugin" => devalang_types::AddonType::Plugin,
        "preset" => devalang_types::AddonType::Preset,
        "template" => devalang_types::AddonType::Template,
        _ => {
            return Err(format!("Unknown addon type: {}", addon_type));
        }
    };

    let addon_metadata = AddonToDownloadMetadata {
        name: payload
            .unwrap_or(&serde_json::Value::Null)
            .get("addon")
            .unwrap_or(&serde_json::Value::Null)
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        publisher: payload
            .unwrap_or(&serde_json::Value::Null)
            .get("publisher")
            .unwrap_or(&serde_json::Value::Null)
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        addon_type: addon_type_enum,
    };

    Ok(addon_metadata)
}
