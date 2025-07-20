use std::{ fs, path::Path };
use std::collections::HashMap;
use crate::config::driver::{ BankEntry, Config };

pub fn load_config(path: Option<&Path>) -> Option<Config> {
    let config_path = path.unwrap_or_else(|| Path::new(".devalang"));

    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}

pub fn update_bank_version_in_config(config: &mut Config, dependency: &str, new_version: &str) {
    // Si le vecteur banks n'existe pas, on ne fait rien
    if config.banks.is_none() {
        println!("No banks configured.");
        return;
    }

    let banks = config.banks.as_mut().unwrap();

    if let Some(bank) = banks.iter_mut().find(|b| b.path.contains(dependency)) {
        bank.version = Some(new_version.to_string());

        if let Err(e) = config.write(config) {
            eprintln!("❌ Failed to write config: {}", e);
        } else {
            println!("✅ Bank '{}' updated to version '{}'", dependency, new_version);
        }
    } else {
        println!("Bank '{}' not found in config", dependency);
    }
}

pub fn remove_bank_from_config(config: &mut Config, dependency: &str) {
    if config.banks.is_none() {
        println!("No banks configured.");
        return;
    }

    let banks = config.banks.as_mut().unwrap();

    if let Some(index) = banks.iter().position(|b| b.path.contains(dependency)) {
        banks.remove(index);

        if let Err(e) = config.write(config) {
            eprintln!("❌ Failed to write config: {}", e);
        } else {
            println!("✅ Bank '{}' removed from config", dependency);
        }
    } else {
        println!("Bank '{}' not found in config", dependency);
    }
}

pub fn add_bank_to_config(config: &mut Config, real_path: &Path, dependency: &str) {
    if config.banks.is_none() {
        config.banks = Some(Vec::new());
    }

    let banks = config.banks.as_mut().unwrap();

    let exists = banks.iter().any(|b| b.path == dependency);
    if exists {
        println!("Bank '{}' already in config", dependency);
        return;
    }

    let metadata_path = Path::new(real_path).join("bank.toml");

    if !metadata_path.exists() {
        eprintln!("❌ Bank metadata file '{}' does not exist", metadata_path.display());
        return;
    }

    let metadata_content = fs
        ::read_to_string(&metadata_path)
        .expect("Failed to read bank metadata file");

    let metadata: HashMap<String, String> = toml
        ::from_str(&metadata_content)
        .expect("Failed to parse bank metadata file");

    let bank_to_insert = BankEntry {
        path: dependency.to_string(),
        version: Some(
            metadata
                .get("version")
                .cloned()
                .unwrap_or_else(|| "0.0.1".to_string())
        ),
        author: Some(
            metadata
                .get("author")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string())
        ),
    };

    banks.push(bank_to_insert);

    if let Err(e) = config.write(config) {
        eprintln!("❌ Failed to write config: {}", e);
    } else {
        println!("✅ Bank '{}' added to config", dependency);
    }
}
