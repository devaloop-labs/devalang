use crate::{
    config::ops::load_config,
    web::cdn::{download_from_cdn, get_cdn_url},
};
use devalang_utils::path as path_utils;
use std::{fs, path::Path};

pub async fn install_plugin(name: &str, target_dir: &Path) -> Result<(), String> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/plugin/{}/download", cdn_url, name);

    let plugin_dir = target_dir.join("plugins");

    // Ensure .deva/tmp exists and build archive path
    let deva_dir = path_utils::ensure_deva_dir()?;
    let tmp_dir = deva_dir.join("tmp");
    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir)
            .map_err(|e| format!("Failed to create tmp dir '{}': {}", tmp_dir.display(), e))?;
    }

    let archive_path = tmp_dir.join(format!("{}.devaplugin", name));
    let extract_path = plugin_dir.join(name);

    if extract_path.exists() {
        println!(
            "Plugin '{}' already exists at '{}'. Skipping install.",
            name,
            extract_path.display()
        );
        return Ok(());
    }

    download_from_cdn(&url, &archive_path)
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;

    // Add the plugin to the config: locate project root from target_dir
    let project_root = path_utils::find_project_root_from(target_dir)
        .ok_or_else(|| "Failed to determine project root from target_dir".to_string())?;

    let config_path = project_root.join(path_utils::DEVALANG_CONFIG);
    if !config_path.exists() {
        return Err(format!(
            "Config file not found at '{}'. Please run 'devalang init' before adding an addon",
            config_path.display()
        ));
    }

    let _config = load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    let _dependency_path = &format!("devalang://plugin/{}", name);

    devalang_utils::file::extract_zip_safely(&archive_path, &extract_path)
        .map_err(|e| format!("Failed to extract: {}", e))?;

    // TODO: Add the plugin to the config

    Ok(())
}
