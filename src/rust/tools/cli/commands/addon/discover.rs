#![cfg(feature = "cli")]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::tools::cli::config::urls::get_api_url;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddonSearchResult {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub addon_type: String,
    pub downloads: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddonSearchResponse {
    pub addons: Vec<AddonSearchResult>,
    pub total: usize,
}

pub async fn discover_addons(
    search_term: Option<String>,
    addon_type: Option<String>,
    author: Option<String>,
) -> Result<Vec<AddonSearchResult>> {
    let api_url = get_api_url();

    // Build search query
    let mut query_params = vec![];

    if let Some(term) = search_term {
        query_params.push(format!("q={}", urlencoding::encode(&term)));
    }

    if let Some(t) = addon_type {
        query_params.push(format!("type={}", urlencoding::encode(&t)));
    }

    if let Some(a) = author {
        query_params.push(format!("author={}", urlencoding::encode(&a)));
    }

    let query_string = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", query_params.join("&"))
    };

    let url = format!("{}/api/addons/search{}", api_url, query_string);

    // Fetch from API
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Devalang-CLI/2.0")
        .send()
        .await
        .context("Failed to connect to Devalang Forge")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch addons: HTTP {}",
            response.status()
        ));
    }

    let search_response: AddonSearchResponse = response
        .json()
        .await
        .context("Failed to parse addon search response")?;

    Ok(search_response.addons)
}

pub fn display_addon_results(addons: &[AddonSearchResult]) {
    if addons.is_empty() {
        println!("\n❌ No addons found matching your criteria.\n");
        return;
    }

    println!("\n📦 Found {} addon(s):\n", addons.len());

    for addon in addons {
        let type_emoji = match addon.addon_type.as_str() {
            "bank" => "🥁",
            "plugin" => "🔌",
            "preset" => "🎛️",
            "template" => "📄",
            _ => "📦",
        };

        println!("{} {} ({})", type_emoji, addon.name, addon.slug);
        println!(
            "   Version: {} | Author: {} | Downloads: {}",
            addon.version, addon.author, addon.downloads
        );

        if !addon.description.is_empty() {
            println!("   {}", addon.description);
        }

        if !addon.tags.is_empty() {
            println!("   Tags: {}", addon.tags.join(", "));
        }

        println!("   Install: devalang addon install {}\n", addon.slug);
    }

    println!("💡 Tip: Visit https://devalang.com/forge to browse all addons\n");
}
