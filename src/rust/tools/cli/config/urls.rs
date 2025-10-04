#![cfg(feature = "cli")]

use std::env;

/// Get the SSO URL from environment or use default
pub fn get_sso_url() -> String {
    env::var("DEVALANG_SSO_URL").unwrap_or_else(|_| "http://127.0.0.1:5174".to_string())
}

/// Get the API URL from environment or use default
pub fn get_api_url() -> String {
    env::var("DEVALANG_API_URL").unwrap_or_else(|_| "https://api.devalang.com".to_string())
}

/// Get the CDN URL from environment or use default
pub fn get_cdn_url() -> String {
    env::var("DEVALANG_CDN_URL").unwrap_or_else(|_| "https://cdn.devalang.com".to_string())
}

/// Get the Forge URL from environment or use default
pub fn get_forge_url() -> String {
    env::var("DEVALANG_FORGE_URL").unwrap_or_else(|_| "https://forge.devalang.com".to_string())
}
