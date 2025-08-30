use crate::web::cdn::get_cdn_url;
use devalang_types::{BankFile, BankInfo};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BankList {
    pub bank: Vec<BankInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BankVersion {
    pub version: String,
}

pub async fn handle_bank_info_command(
    name: String,
) -> Result<BankInfo, Box<dyn std::error::Error>> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/bank/{}/info", cdn_url, name);

    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch bank info: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    let parsed: BankInfo = serde_json::from_slice(&bytes)?;

    println!("📦 Bank Info for '{}':", name);
    println!("  - Name: {}", parsed.name);
    println!("  - Version: {}", parsed.version);
    println!("  - Description: {}", parsed.description);
    println!("  - Author: {}", parsed.author);

    Ok(parsed)
}

pub async fn fetch_latest_version(
    bank_name: String,
) -> Result<BankVersion, Box<dyn std::error::Error>> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/bank/{}/version", cdn_url, bank_name);

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("❌ Failed to fetch version: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    let version: BankVersion = serde_json::from_slice(&bytes)?;

    Ok(version)
}

pub async fn list_external_banks() -> Result<Vec<BankInfo>, Box<dyn std::error::Error>> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/bank/list", cdn_url);

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("❌ Failed to fetch bank list: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;

    let parsed: BankList = serde_json::from_slice(&bytes)?;

    Ok(parsed.bank)
}

pub async fn list_installed_banks() -> Result<Vec<BankFile>, String> {
    let deva_dir = devalang_utils::path::ensure_deva_dir()?;
    let bank_dir = deva_dir.join("banks");

    let mut banks = Vec::new();

    if !bank_dir.exists() {
        return Ok(banks); // No banks installed
    }

    let config_path = devalang_utils::path::get_devalang_config_path()?;
    let config = crate::config::ops::load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    let config_banks = config.banks.clone();

    if let Some(banks_in_toml) = config_banks {
        for bank in banks_in_toml {
            let bank_name = bank
                .path
                .strip_prefix("devalang://bank/")
                .unwrap_or(&bank.path)
                .to_string();

            let bank_path = bank_dir.join(&bank_name);
            if bank_path.exists() {
                let bank_info_path = bank_path.join("bank.toml");

                if bank_info_path.exists() {
                    let content = std::fs::read_to_string(&bank_info_path)
                        .map_err(|e| format!("Failed to read bank info: {}", e))?;

                    match toml::from_str::<BankFile>(&content) {
                        Ok(bank_info) => banks.push(bank_info),
                        Err(_err) => {
                            eprintln!("❌ Error parsing bank info for '{}'", bank_name);
                        }
                    }
                } else {
                    eprintln!("❌ Bank info file not found for '{}'", bank_name);
                }
            }
        }
    }

    Ok(banks)
}
