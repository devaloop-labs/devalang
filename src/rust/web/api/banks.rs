//! Bank manifest loading for WASM
//!
//! Loads bank.toml manifests, fetches WAV samples, and registers them in the engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit};

use crate::web::registry::{banks, debug, samples};
use crate::web::utils::wav_parser;

#[derive(Deserialize, Serialize)]
struct SimpleBankManifest {
    name: String,
    alias: String,
    version: String,
    description: String,
    triggers: HashMap<String, String>,
}

#[derive(Deserialize)]
struct BankToml {
    bank: BankInfo,
    #[serde(default)]
    triggers: Vec<TriggerInfo>,
}

#[derive(Deserialize)]
struct BankInfo {
    name: String,
    #[serde(alias = "author", alias = "publisher")]
    publisher: Option<String>,
    #[serde(alias = "audioPath", default = "default_audio_path")]
    audio_path: String,
}

#[derive(Deserialize)]
struct TriggerInfo {
    name: String,
    path: String,
}

/// Register a bank from a simple JSON manifest (for testing/manual registration)
///
/// Manifest format:
/// ```json
/// {
///   "name": "devaloop.808",
///   "alias": "kit",
///   "version": "1.0.0",
///   "description": "808 drum bank",
///   "triggers": {
///     "kick": "http://example.com/kick.wav",
///     "snare": "http://example.com/snare.wav"
///   }
/// }
/// ```
///
/// This only registers the bank metadata. Samples must be registered separately using register_sample().
#[wasm_bindgen]
pub fn register_bank_json(manifest_json: &str) -> Result<(), JsValue> {
    let manifest: SimpleBankManifest = serde_json::from_str(manifest_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    banks::register_bank(
        manifest.name.clone(),
        manifest.alias.clone(),
        manifest.triggers.clone(),
    );

    Ok(())
}

fn default_audio_path() -> String {
    "audio/".to_string()
}

/// Sanitize trigger path (remove ./, leading slashes, audio_dir prefix)
fn sanitize_trigger_path(raw: &str, audio_dir: &str) -> String {
    let mut p = raw.replace('\\', "/").trim().to_string();

    if p.starts_with("./") {
        p = p[2..].to_string();
    }

    while p.starts_with('/') {
        p.remove(0);
    }

    if p.starts_with(audio_dir) {
        p = p[audio_dir.len()..].to_string();
    }

    if p.starts_with("./") {
        p = p[2..].to_string();
    }

    p
}

/// Load and register a complete bank from bank.toml hosted at base_url
///
/// Steps:
/// 1. Fetch base_url + "/bank.toml"
/// 2. Parse triggers
/// 3. For each trigger.path => fetch WAV file (relative to base_url)
/// 4. Parse WAV directly in Rust (no Web Audio API needed)
/// 5. Register samples with URI: devalang://bank/{publisher.name}/{path}
/// 6. Call register_addon() with query string for triggers
///
/// Returns: { ok, bank, base_url, triggers: [{ name, uri, relative, frames }] }
#[wasm_bindgen]
pub async fn register_bank_from_manifest(base_url: String) -> Result<JsValue, JsValue> {
    let trimmed = base_url.trim_end_matches('/');
    let manifest_url = format!("{}/bank.toml", trimmed);

    // Step 1: Fetch bank.toml
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(&manifest_url, &opts)
        .map_err(|e| JsValue::from_str(&format!("Request error: {:?}", e)))?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| JsValue::from_str(&format!("Fetch failed: {:?}", e)))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| JsValue::from_str("Invalid response type"))?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP {} loading bank.toml from {}",
            resp.status(),
            manifest_url
        )));
    }

    let text_js = JsFuture::from(
        resp.text()
            .map_err(|e| JsValue::from_str(&format!("resp.text() error: {:?}", e)))?,
    )
    .await
    .map_err(|e| JsValue::from_str(&format!("text() await error: {:?}", e)))?;

    let toml_str = text_js
        .as_string()
        .ok_or_else(|| JsValue::from_str("bank.toml is not text"))?;

    // Step 2: Parse TOML
    let parsed: BankToml = toml::from_str(&toml_str)
        .map_err(|e| JsValue::from_str(&format!("TOML parse error: {}", e)))?;

    let bank_name = parsed.bank.name.clone();
    let publisher = parsed
        .bank
        .publisher
        .clone()
        .ok_or_else(|| JsValue::from_str("bank.author/publisher missing"))?;
    let full_name = format!("{}.{}", publisher, bank_name);

    let mut audio_path_norm = parsed.bank.audio_path.replace('\\', "/");
    if !audio_path_norm.ends_with('/') {
        audio_path_norm.push('/');
    }
    while audio_path_norm.starts_with('/') {
        audio_path_norm.remove(0);
    }

    // Step 3-5: Load each trigger sample
    let mut registered_trigger_infos: Vec<(String, String, usize)> = Vec::new();

    for trigger in parsed.triggers.iter() {
        let clean_path = sanitize_trigger_path(&trigger.path, &audio_path_norm);
        let file_url = format!("{}/{}{}", trimmed, audio_path_norm, clean_path);

        // Fetch WAV file
        let opts_f = RequestInit::new();
        opts_f.set_method("GET");

        let req_f = Request::new_with_str_and_init(&file_url, &opts_f)
            .map_err(|e| JsValue::from_str(&format!("Sample request error: {:?}", e)))?;

        let resp_v = JsFuture::from(window.fetch_with_request(&req_f))
            .await
            .map_err(|e| JsValue::from_str(&format!("Fetch sample failed: {:?}", e)))?;

        let resp_s: web_sys::Response = resp_v
            .dyn_into()
            .map_err(|_| JsValue::from_str("Bad sample response"))?;

        if !resp_s.ok() {
            let msg = format!("⚠️ HTTP {} for sample {}", resp_s.status(), file_url);
            web_sys::console::warn_1(&JsValue::from_str(&msg));
            debug::log_sample_load(format!("❌ {}", msg));
            continue;
        }

        let ab_promise = resp_s
            .array_buffer()
            .map_err(|e| JsValue::from_str(&format!("array_buffer() error: {:?}", e)))?;

        let ab = JsFuture::from(ab_promise)
            .await
            .map_err(|e| JsValue::from_str(&format!("array_buffer() await error: {:?}", e)))?;

        let u8arr = js_sys::Uint8Array::new(&ab);
        let mut bytes = vec![0u8; u8arr.length() as usize];
        u8arr.copy_to(&mut bytes[..]);

        // Parse WAV
        match wav_parser::parse_wav_generic(&bytes) {
            Ok((_channels, _sample_rate, mono_i16)) => {
                let uri = format!("devalang://bank/{}/{}", full_name, clean_path);

                // Register sample
                samples::register_sample(uri.clone(), mono_i16.clone());

                registered_trigger_infos.push((trigger.name.clone(), clean_path, mono_i16.len()));
            }
            Err(e) => {
                let msg = format!("⚠️ WAV parse error '{}': {}", file_url, e);
                web_sys::console::warn_1(&JsValue::from_str(&msg));
                debug::log_sample_load(format!("❌ {}", msg));
            }
        }
    }

    // Step 6: Register addon with trigger query string
    let trigger_query: String = if registered_trigger_infos.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = registered_trigger_infos
            .iter()
            .map(|(n, p, _)| format!("{}:{}", n, p))
            .collect();
        format!("?triggers={}", parts.join(","))
    };

    // Note: addon_path kept for compatibility, not currently used
    let _addon_path = format!("devalang://bank/{}{}", full_name, trigger_query);

    // Register bank with triggers map
    let mut triggers_map = std::collections::HashMap::new();
    for (name, path, _) in &registered_trigger_infos {
        let uri = format!("devalang://bank/{}/{}", full_name, path);
        triggers_map.insert(name.clone(), uri);
    }

    banks::register_bank(full_name.clone(), bank_name.clone(), triggers_map);
    debug::log_playback_debug(format!(
        "Registered bank from manifest: {} ({} triggers)",
        full_name,
        registered_trigger_infos.len()
    ));

    // Build response
    #[derive(Serialize)]
    struct TriggerResponse {
        name: String,
        uri: String,
        relative: String,
        frames: usize,
    }

    #[derive(Serialize)]
    struct Response {
        ok: bool,
        bank: String,
        base_url: String,
        triggers: Vec<TriggerResponse>,
    }

    let response = Response {
        ok: true,
        bank: full_name.clone(),
        base_url: trimmed.to_string(),
        triggers: registered_trigger_infos
            .iter()
            .map(|(n, p, frames)| TriggerResponse {
                name: n.clone(),
                relative: p.clone(),
                uri: format!("devalang://bank/{}/{}", full_name, p),
                frames: *frames,
            })
            .collect(),
    };

    serde_wasm_bindgen::to_value(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Load a bank from a URL (auto-detects bank.toml or bank.json)
///
/// Tries in order:
/// 1. Exact URL if it ends with .toml/.json
/// 2. {base_url}/bank.toml
/// 3. {base_url}/bank.json
///
/// Example:
/// - `load_bank_from_url("https://example.com/banks/devaloop/808")`
///   → tries "https://example.com/banks/devaloop/808/bank.toml" then "bank.json"
/// - `load_bank_from_url("https://example.com/banks/kit.bank.json")`
///   → loads exactly that JSON file
#[wasm_bindgen]
pub async fn load_bank_from_url(url: String) -> Result<JsValue, JsValue> {
    let url_lower = url.to_lowercase();

    // If URL explicitly ends with .toml or .json, use it directly
    if url_lower.ends_with(".bank.toml") || url_lower.ends_with(".toml") {
        return register_bank_from_manifest_toml(&url).await;
    } else if url_lower.ends_with(".bank.json") || url_lower.ends_with(".json") {
        return register_bank_from_manifest_json(&url).await;
    }

    // Otherwise treat as base URL and try both formats
    let base_url = url.trim_end_matches('/');

    // Try .toml first
    let toml_url = format!("{}/bank.toml", base_url);
    match register_bank_from_manifest_toml(&toml_url).await {
        Ok(result) => return Ok(result),
        Err(e) => {
            web_sys::console::log_1(
                &format!(
                    "TOML failed ({}), trying JSON...",
                    e.as_string().unwrap_or_default()
                )
                .into(),
            );
        }
    }

    // Try .json
    let json_url = format!("{}/bank.json", base_url);
    register_bank_from_manifest_json(&json_url).await
}

/// Internal helper to load from TOML manifest URL
async fn register_bank_from_manifest_toml(manifest_url: &str) -> Result<JsValue, JsValue> {
    // Extract base URL from manifest URL
    let base_url = if let Some(idx) = manifest_url.rfind("/bank.toml") {
        &manifest_url[..idx]
    } else if let Some(idx) = manifest_url.rfind('/') {
        &manifest_url[..idx]
    } else {
        manifest_url
    };

    register_bank_from_manifest(base_url.to_string()).await
}

/// Internal helper to load from JSON manifest URL
async fn register_bank_from_manifest_json(manifest_url: &str) -> Result<JsValue, JsValue> {
    // Extract base URL
    let base_url = if let Some(idx) = manifest_url.rfind("/bank.json") {
        &manifest_url[..idx]
    } else if let Some(idx) = manifest_url.rfind('/') {
        &manifest_url[..idx]
    } else {
        manifest_url
    };

    // Fetch JSON manifest
    let opts = RequestInit::new();
    opts.set_method("GET");

    let request = Request::new_with_str_and_init(manifest_url, &opts)
        .map_err(|e| JsValue::from_str(&format!("Request error: {:?}", e)))?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| JsValue::from_str(&format!("Fetch failed: {:?}", e)))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| JsValue::from_str("Invalid response type"))?;

    if !resp.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP {} loading bank.json from {}",
            resp.status(),
            manifest_url
        )));
    }

    let text_promise = resp
        .text()
        .map_err(|e| JsValue::from_str(&format!("text() error: {:?}", e)))?;

    let text_value = JsFuture::from(text_promise)
        .await
        .map_err(|e| JsValue::from_str(&format!("text() await error: {:?}", e)))?;

    let text = text_value
        .as_string()
        .ok_or_else(|| JsValue::from_str("Response not a string"))?;

    // Parse JSON
    let manifest: SimpleBankManifest = serde_json::from_str(&text)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))?;

    let full_name = manifest.name.clone();
    let alias = manifest.alias.clone();

    web_sys::console::log_1(
        &format!("[devalang] Loading JSON bank: {} ({})", full_name, alias).into(),
    );

    // Fetch each sample
    let mut registered_trigger_infos = Vec::new();

    for (trigger_name, sample_url) in &manifest.triggers {
        // Resolve relative URLs
        let full_url = if sample_url.starts_with("http://") || sample_url.starts_with("https://") {
            sample_url.clone()
        } else {
            format!("{}/{}", base_url, sample_url.trim_start_matches('/'))
        };

        web_sys::console::log_1(&format!("  Fetching sample: {}", full_url).into());

        // Fetch sample
        let sample_req = Request::new_with_str_and_init(&full_url, &opts)
            .map_err(|e| JsValue::from_str(&format!("Sample request error: {:?}", e)))?;

        let sample_resp_value = JsFuture::from(window.fetch_with_request(&sample_req))
            .await
            .map_err(|e| JsValue::from_str(&format!("Fetch sample failed: {:?}", e)))?;

        let sample_resp: web_sys::Response = sample_resp_value
            .dyn_into()
            .map_err(|_| JsValue::from_str("Bad sample response"))?;

        if !sample_resp.ok() {
            let msg = format!("⚠️ HTTP {} for sample {}", sample_resp.status(), full_url);
            web_sys::console::warn_1(&JsValue::from_str(&msg));
            continue;
        }

        let ab_promise = sample_resp
            .array_buffer()
            .map_err(|e| JsValue::from_str(&format!("array_buffer() error: {:?}", e)))?;

        let ab = JsFuture::from(ab_promise)
            .await
            .map_err(|e| JsValue::from_str(&format!("array_buffer() await error: {:?}", e)))?;

        let u8arr = js_sys::Uint8Array::new(&ab);
        let mut bytes = vec![0u8; u8arr.length() as usize];
        u8arr.copy_to(&mut bytes[..]);

        // Parse WAV
        match wav_parser::parse_wav_generic(&bytes) {
            Ok((_channels, _sample_rate, mono_i16)) => {
                // Register sample with trigger name as identifier (consistent with inject_registered_banks)
                // URI format: devalang://bank/{full_name}/{trigger_name}
                let uri = format!("devalang://bank/{}/{}", full_name, trigger_name);

                // Register sample
                samples::register_sample(uri.clone(), mono_i16.clone());

                registered_trigger_infos.push((
                    trigger_name.clone(),
                    sample_url.clone(), // Store the original path for reference
                    mono_i16.len(),
                ));
            }
            Err(e) => {
                let msg = format!("⚠️ WAV parse error '{}': {}", full_url, e);
                web_sys::console::warn_1(&JsValue::from_str(&msg));
            }
        }
    }

    // Register bank
    banks::register_bank(full_name.clone(), alias.clone(), manifest.triggers.clone());

    // Return response
    #[derive(Serialize)]
    struct TriggerResponse {
        name: String,
        uri: String,
        relative: String,
        frames: usize,
    }

    #[derive(Serialize)]
    struct Response {
        ok: bool,
        bank: String,
        base_url: String,
        triggers: Vec<TriggerResponse>,
    }

    let response = Response {
        ok: true,
        bank: full_name.clone(),
        base_url: base_url.to_string(),
        triggers: registered_trigger_infos
            .iter()
            .map(|(n, p, frames)| TriggerResponse {
                name: n.clone(),
                relative: p.clone(),
                uri: format!("devalang://bank/{}/{}", full_name, p),
                frames: *frames,
            })
            .collect(),
    };

    serde_wasm_bindgen::to_value(&response)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
