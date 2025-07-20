use std::path::{ Path, PathBuf };
use crate::{
    common::cdn::get_cdn_url,
    config::loader::{ add_bank_to_config, load_config },
    utils::installer::{ download_file, extract_archive },
};

pub async fn install_bank(name: &str, target_dir: &Path) -> Result<(), String> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/bank/{}", cdn_url, name);

    let bank_dir = target_dir.join("bank");
    let archive_path = PathBuf::from(format!("./.deva/tmp/{}.devabank", name));
    let extract_path = bank_dir.join(name);

    if extract_path.exists() {
        println!(
            "Bank '{}' already exists at '{}'. Skipping install.",
            name,
            extract_path.display()
        );
        return Ok(());
    }

    download_file(&url, &archive_path).await.map_err(|e| format!("Failed to download: {}", e))?;

    extract_archive(&archive_path, &extract_path).await.map_err(|e|
        format!("Failed to extract: {}", e)
    )?;

    // Add the bank to the config
    let root_dir = target_dir
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

    let dependency_path = &format!("devalang://bank/{}", name);

    add_bank_to_config(&mut config, &extract_path, &dependency_path);

    Ok(())
}
