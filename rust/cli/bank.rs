use std::fs;

use serde::{ Deserialize, Serialize };
use crate::{
    common::cdn::get_cdn_url,
    config::loader::{ load_config, remove_bank_from_config, update_bank_version_in_config },
    core::shared::bank::{ BankFile, BankInfo },
    installer::bank::install_bank,
};

#[derive(Debug, Deserialize)]
pub struct BankList {
    bank: Vec<BankInfo>,
}

#[derive(Debug, Deserialize)]
pub struct BankInfoFetched {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub latest_version: String,
}

#[derive(Debug, Deserialize)]
pub struct BankVersion {
    pub version: String,
}

pub async fn handle_update_bank_command(name: Option<String>) -> Result<(), String> {
    let deva_dir = std::path::Path::new("./.deva");
    let bank_dir = deva_dir.join("bank");

    if !bank_dir.exists() {
        fs
            ::create_dir_all(bank_dir.clone())
            .map_err(|e| format!("Failed to create bank directory: {}", e))?;
    }

    if let Some(name) = name {
        let bank_path = bank_dir.join(&name);
        if !bank_path.exists() {
            return Err(format!("Bank '{}' is not installed", name));
        }

        // Update specific bank
        let latest_version = match fetch_latest_version(name.clone()).await {
            Ok(version) => version,
            Err(err) => {
                eprintln!("❌ Error fetching latest version for '{}': {}", name, err);
                return Err(format!("Failed to fetch latest version for '{}'", name));
            }
        };

        let local_bank_info_path = bank_path.join("bank.toml");
        let local_version = match
            fs
                ::read_to_string(&local_bank_info_path)
                .ok()
                .and_then(|content| toml::from_str::<BankFile>(&content).ok())
                .map(|bf| bf.bank.version)
        {
            Some(version) => version,
            None => {
                eprintln!("⚠️ Unable to read local version for '{}', forcing reinstall...", name);
                "".to_string() // Force update
            }
        };

        if local_version != latest_version.version {
            if let Err(e) = update_bank(&name, &latest_version.version).await {
                eprintln!("❌ Error updating bank '{}': {}", name, e);
            } else {
                println!("✅ Bank '{}' updated to version '{}'", name, latest_version.version);
            }
        } else {
            println!("Bank '{}' is already up-to-date (version {})", name, latest_version.version);

            // Verify if the bank directory exists
            if !bank_path.exists() {
                eprintln!("❌ Bank directory for '{}' does not exist, reinstalling...", name);
                if let Err(e) = install_bank(&name, deva_dir).await {
                    eprintln!("❌ Error reinstalling bank '{}': {}", name, e);
                } else {
                    println!("✅ Bank '{}' reinstalled successfully!", name);
                }
            }
        }
    } else {
        // Update all banks
        let root_dir = deva_dir
            .parent()
            .ok_or_else(|| "Failed to determine root directory".to_string())?;

        let config_path = root_dir.join(".devalang");
        if !config_path.exists() {
            return Err(
                format!(
                    "Config file not found at '{}'. Please run 'devalang init' before adding an addon",
                    config_path.display()
                )
            );
        }

        let mut config = load_config(Some(&config_path)).ok_or_else(||
            format!("Failed to load config from '{}'", config_path.display())
        )?;

        let config_banks = config.banks.clone();

        // Install or update all banks

        if let Some(banks) = config_banks {
            for bank in banks {
                let bank_name = bank.path
                    .strip_prefix("devalang://bank/")
                    .unwrap_or(&bank.path)
                    .to_string();

                let latest_version = match fetch_latest_version(bank_name.clone()).await {
                    Ok(version) => version,
                    Err(err) => {
                        eprintln!("❌ Error fetching latest version for '{}': {}", bank_name, err);
                        continue;
                    }
                };

                if let Some(local_bank_version) = bank.version {
                    if latest_version.version != local_bank_version {
                        if let Err(e) = update_bank(&bank_name, &latest_version.version).await {
                            eprintln!("❌ Error updating bank '{}': {}", bank_name, e);
                        } else {
                            println!(
                                "✅ Bank '{}' updated to version '{}'",
                                bank_name,
                                latest_version.version
                            );
                        }
                    } else {
                        println!(
                            "Bank '{}' is already up-to-date (version {})",
                            bank_name,
                            local_bank_version
                        );

                        // Verify if the bank directory exists
                        let bank_path = bank_dir.join(&bank_name);

                        if !bank_path.exists() {
                            eprintln!("❌ Bank directory for '{}' does not exist, reinstalling...", bank_name);
                            if let Err(e) = install_bank(&bank_name, deva_dir).await {
                                eprintln!("❌ Error reinstalling bank '{}': {}", bank_name, e);
                            } else {
                                println!("✅ Bank '{}' reinstalled successfully!", bank_name);
                            }
                        }
                    }
                } else {
                    // If the bank version is not specified in the config, install it
                    if let Err(e) = install_bank(&bank_name, deva_dir).await {
                        eprintln!("❌ Error installing bank '{}': {}", bank_name, e);
                    } else {
                        println!("✅ Bank '{}' installed successfully!", bank_name);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn update_bank(bank_name: &str, latest_version: &str) -> Result<(), String> {
    let deva_dir = std::path::Path::new("./.deva");

    // First, delete the existing bank directory
    let bank_dir = deva_dir.join("bank").join(bank_name);

    if bank_dir.exists() {
        std::fs::remove_dir_all(&bank_dir).unwrap_or_else(|_| {
            eprintln!("⚠️ Failed to remove old bank directory for '{}', aborting !", bank_name);
            std::process::exit(1);
        });
    }

    // Now, install the new version
    if let Err(e) = install_bank(bank_name, deva_dir).await {
        eprintln!("❌ Error installing bank '{}': {}", bank_name, e);
    } else {
        println!("✅ Bank '{}' installed successfully!", bank_name);
    }

    let root_dir = deva_dir
        .parent()
        .ok_or_else(|| "Failed to determine root directory".to_string())?;

    let config_path = root_dir.join(".devalang");
    if !config_path.exists() {
        return Err(
            format!(
                "Config file not found at '{}'. Please run 'devalang init' before adding an addon",
                config_path.display()
            )
        );
    }

    let mut config = load_config(Some(&config_path)).ok_or_else(||
        format!("Failed to load config from '{}'", config_path.display())
    )?;

    // Update the bank version in the config
    update_bank_version_in_config(&mut config, bank_name, latest_version);

    Ok(())
}

pub async fn handle_remove_bank_command(name: String) -> Result<(), String> {
    let deva_dir = std::path::Path::new("./.deva");
    let bank_dir = deva_dir.join("bank");

    let bank_path = bank_dir.join(&name);

    if !bank_path.exists() {
        return Err(format!("Bank '{}' is not installed", name));
    }

    std::fs::remove_dir_all(&bank_path).map_err(|e| format!("Failed to remove bank: {}", e))?;

    // Remove the bank from the config
    let root_dir = deva_dir
        .parent()
        .ok_or_else(|| "Failed to determine root directory".to_string())?;

    let config_path = root_dir.join(".devalang");
    if !config_path.exists() {
        return Err(
            format!(
                "Config file not found at '{}'. Please run 'devalang init' before adding an addon",
                config_path.display()
            )
        );
    }

    let mut config = load_config(Some(&config_path)).ok_or_else(||
        format!("Failed to load config from '{}'", config_path.display())
    )?;

    remove_bank_from_config(&mut config, &name);

    println!("✅ Bank '{}' removed successfully", name);

    Ok(())
}

pub async fn handle_bank_available_command() -> Result<(), String> {
    let bank_list = match list_external_banks().await {
        Ok(list) => list,
        Err(err) => {
            eprintln!("❌ Error fetching bank list: {}", err);
            return Err("Failed to fetch bank list".into());
        }
    };

    println!("Available banks for current project :");
    println!(" ");

    for bank in bank_list {
        println!("📦 {}", bank.name);
        println!("   - Version: {}", bank.version);
        println!("   - Description: {}", bank.description);
        println!("   - Author: {}", bank.author);
        println!(" ");
    }

    Ok(())
}

pub async fn handle_bank_info_command(
    name: String
) -> Result<BankInfo, Box<dyn std::error::Error>> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/bank/{}/info", cdn_url, name);

    let response = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(err) => {
            return Err("Failed to fetch bank info".into());
        }
    };

    if !response.status().is_success() {
        return Err(format!("Failed to fetch bank info: HTTP {}", response.status()).into());
    }

    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(err) => {
            eprintln!("❌ Error reading response body: {}", err);
            return Err("Failed to read response body".into());
        }
    };

    let parsed: BankInfo = serde_json::from_slice(&bytes)?;

    println!("📦 Bank Info for '{}':", name);
    println!("  - Name: {}", parsed.name);
    println!("  - Version: {}", parsed.version);
    println!("  - Description: {}", parsed.description);
    println!("  - Author: {}", parsed.author);

    Ok(parsed)
}

pub async fn handle_bank_list_command() -> Result<(), String> {
    let bank_list = match list_installed_banks().await {
        Ok(list) => list,
        Err(err) => {
            eprintln!("❌ Error fetching bank list: {}", err);
            return Err("Failed to fetch bank list".into());
        }
    };

    println!("Installed banks for current project :");

    for bank_toml in bank_list {
        let latest_version = match fetch_latest_version(bank_toml.bank.name.clone()).await {
            Ok(version) => version,
            Err(err) => {
                eprintln!(
                    "❌ Error fetching latest version for '{}': {}",
                    bank_toml.bank.name,
                    err
                );
                continue;
            }
        };

        let is_latest = if latest_version.version == bank_toml.bank.version { "✅" } else { "❗" };

        println!(" ");
        println!("📦 {}", bank_toml.bank.name);
        println!(
            "  - Version: v{} {} (latest: v{})",
            bank_toml.bank.version,
            is_latest,
            latest_version.version
        );
        println!("  - Description: {}", bank_toml.bank.description);
        println!("  - Author: {}", bank_toml.bank.author);
    }

    Ok(())
}

async fn fetch_latest_version(
    bank_name: String
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

async fn list_external_banks() -> Result<Vec<BankInfo>, Box<dyn std::error::Error>> {
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

async fn list_installed_banks() -> Result<Vec<BankFile>, String> {
    let deva_dir = std::path::Path::new("./.deva");
    let bank_dir = deva_dir.join("bank");

    let mut banks = Vec::new();

    if !bank_dir.exists() {
        return Ok(banks); // No banks installed
    }

    // let installed_banks = std::fs
    //     ::read_dir(bank_dir)
    //     .map_err(|e| format!("Failed to read bank directory: {}", e))?;

    let root_dir = deva_dir
        .parent()
        .ok_or_else(|| "Failed to determine root directory".to_string())?;

    let config_path = root_dir.join(".devalang");
    if !config_path.exists() {
        return Err(
            format!(
                "Config file not found at '{}'. Please run 'devalang init' before adding an addon",
                config_path.display()
            )
        );
    }

    let mut config = load_config(Some(&config_path)).ok_or_else(||
        format!("Failed to load config from '{}'", config_path.display())
    )?;

    let config_banks = config.banks.clone();

    if let Some(banks_in_toml) = config_banks {
        for bank in banks_in_toml {
            let bank_name = bank.path
                .strip_prefix("devalang://bank/")
                .unwrap_or(&bank.path)
                .to_string();

            let bank_path = bank_dir.join(&bank_name);
            if bank_path.exists() {
                let bank_info_path = bank_path.join("bank.toml");

                if bank_info_path.exists() {
                    let content = std::fs
                        ::read_to_string(&bank_info_path)
                        .map_err(|e| format!("Failed to read bank info: {}", e))?;

                    match toml::from_str::<BankFile>(&content) {
                        Ok(bank_info) => banks.push(bank_info),
                        Err(err) => {
                            eprintln!("❌ Error parsing bank info for '{}': {}", bank_name, err);
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
