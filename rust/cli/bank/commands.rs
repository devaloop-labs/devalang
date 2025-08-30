use crate::cli::install::bank::install_bank;
use crate::config::ops::load_config;
use devalang_types::{BankFile, BankInfo};
use devalang_utils::path as path_utils;
use std::fs;

pub async fn handle_update_bank_command(name: Option<String>) -> Result<(), String> {
    let deva_dir = path_utils::ensure_deva_dir()?;
    let bank_dir = deva_dir.join("banks");

    if !bank_dir.exists() {
        fs::create_dir_all(bank_dir.clone())
            .map_err(|e| format!("Failed to create bank directory: {}", e))?;
    }

    if let Some(name) = name {
        let bank_path = bank_dir.join(&name);
        if !bank_path.exists() {
            return Err(format!("Bank '{}' is not installed", name));
        }

        // Update specific bank
        let latest_version = match crate::cli::bank::api::fetch_latest_version(name.clone()).await {
            Ok(version) => version,
            Err(err) => {
                eprintln!("❌ Error fetching latest version for '{}': {}", name, err);
                return Err(format!("Failed to fetch latest version for '{}'", name));
            }
        };

        let local_bank_info_path = bank_path.join("bank.toml");
        let local_version = match fs::read_to_string(&local_bank_info_path)
            .ok()
            .and_then(|content| toml::from_str::<BankFile>(&content).ok())
            .map(|bf| bf.bank.version)
        {
            Some(version) => version,
            None => {
                eprintln!(
                    "⚠️ Unable to read local version for '{}', forcing reinstall...",
                    name
                );
                "".to_string() // Force update
            }
        };

        if local_version != latest_version.version {
            if let Err(e) = update_bank(&name, &latest_version.version).await {
                eprintln!("❌ Error updating bank '{}': {}", name, e);
            } else {
                println!(
                    "✅ Bank '{}' updated to version '{}'",
                    name, latest_version.version
                );
            }
        } else {
            println!(
                "Bank '{}' is already up-to-date (version {})",
                name, latest_version.version
            );

            // Verify if the bank directory exists
            if !bank_path.exists() {
                eprintln!(
                    "❌ Bank directory for '{}' does not exist, reinstalling...",
                    name
                );
                if let Err(e) = install_bank(&name, &deva_dir).await {
                    eprintln!("❌ Error reinstalling bank '{}': {}", name, e);
                } else {
                    println!("✅ Bank '{}' reinstalled successfully!", name);
                }
            }
        }
    } else {
        // Update all banks
        let config_path = path_utils::get_devalang_config_path()?;
        let config = load_config(Some(&config_path))
            .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

        let config_banks = config.banks.clone();

        if let Some(banks) = config_banks {
            for bank in banks {
                let bank_name = bank
                    .path
                    .strip_prefix("devalang://bank/")
                    .unwrap_or(&bank.path)
                    .to_string();

                let latest_version =
                    match crate::cli::bank::api::fetch_latest_version(bank_name.clone()).await {
                        Ok(version) => version,
                        Err(err) => {
                            eprintln!(
                                "❌ Error fetching latest version for '{}': {}",
                                bank_name, err
                            );
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
                                bank_name, latest_version.version
                            );
                        }
                    } else {
                        println!(
                            "Bank '{}' is already up-to-date (version {})",
                            bank_name, local_bank_version
                        );

                        // Verify if the bank directory exists
                        let bank_path = bank_dir.join(&bank_name);

                        if !bank_path.exists() {
                            eprintln!(
                                "❌ Bank directory for '{}' does not exist, reinstalling...",
                                bank_name
                            );
                            if let Err(e) = install_bank(&bank_name, &deva_dir).await {
                                eprintln!("❌ Error reinstalling bank '{}': {}", bank_name, e);
                            } else {
                                println!("✅ Bank '{}' reinstalled successfully!", bank_name);
                            }
                        }
                    }
                } else {
                    // If the bank version is not specified in the config, install it
                    if let Err(e) = install_bank(&bank_name, &deva_dir).await {
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

async fn update_bank(bank_name: &str, _latest_version: &str) -> Result<(), String> {
    let deva_dir = path_utils::ensure_deva_dir()?;

    // First, delete the existing bank directory
    let bank_dir = deva_dir.join("banks").join(bank_name);

    if bank_dir.exists() {
        std::fs::remove_dir_all(&bank_dir).unwrap_or_else(|_| {
            eprintln!(
                "⚠️ Failed to remove old bank directory for '{}', aborting !",
                bank_name
            );
            std::process::exit(1);
        });
    }

    // Now, install the new version
    if let Err(e) = install_bank(bank_name, &deva_dir).await {
        eprintln!("❌ Error installing bank '{}': {}", bank_name, e);
    } else {
        println!("✅ Bank '{}' installed successfully!", bank_name);
    }

    let config_path = path_utils::get_devalang_config_path()?;
    let _config = load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    // TODO Update the bank version in the config

    Ok(())
}

pub async fn handle_remove_bank_command(name: String) -> Result<(), String> {
    let deva_dir = path_utils::ensure_deva_dir()?;
    let bank_dir = deva_dir.join("banks");

    let bank_path = bank_dir.join(&name);

    if !bank_path.exists() {
        return Err(format!("Bank '{}' is not installed", name));
    }

    std::fs::remove_dir_all(&bank_path).map_err(|e| format!("Failed to remove bank: {}", e))?;

    // Remove the bank from the config
    let config_path = path_utils::get_devalang_config_path()?;
    let _config = load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    // TODO Remove the bank from the config

    println!("✅ Bank '{}' removed successfully", name);

    Ok(())
}

pub async fn handle_bank_available_command() -> Result<(), String> {
    let bank_list = match crate::cli::bank::api::list_external_banks().await {
        Ok(list) => list,
        Err(_err) => {
            eprintln!("❌ Error fetching bank list");
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
    name: String,
) -> Result<BankInfo, Box<dyn std::error::Error>> {
    crate::cli::bank::api::handle_bank_info_command(name).await
}

pub async fn handle_bank_list_command() -> Result<(), String> {
    let bank_list = match crate::cli::bank::api::list_installed_banks().await {
        Ok(list) => list,
        Err(_err) => {
            eprintln!("❌ Error fetching bank list");
            return Err("Failed to fetch bank list".into());
        }
    };

    println!("Installed banks for current project :");

    for bank_toml in bank_list {
        let latest_version =
            match crate::cli::bank::api::fetch_latest_version(bank_toml.bank.name.clone()).await {
                Ok(version) => version,
                Err(_err) => {
                    eprintln!(
                        "❌ Error fetching latest version for '{}'",
                        bank_toml.bank.name
                    );
                    continue;
                }
            };

        let is_latest = if latest_version.version == bank_toml.bank.version {
            "✅"
        } else {
            "❗"
        };

        println!(" ");
        println!("📦 {}", bank_toml.bank.name);
        println!(
            "  - Version: v{} {} (latest: v{})",
            bank_toml.bank.version, is_latest, latest_version.version
        );
        println!("  - Description: {}", bank_toml.bank.description);
        println!("  - Author: {}", bank_toml.bank.author);
    }

    Ok(())
}
