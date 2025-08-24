use std::path::{ Path, PathBuf };
use crate::{
    common::cdn::get_cdn_url,
    config::loader::{ add_plugin_to_config, load_config },
    installer::utils::{ download_file, extract_archive },
};

pub async fn install_plugin(name: &str, target_dir: &Path) -> Result<(), String> {
    let cdn_url = get_cdn_url();
    let url = format!("{}/plugin/{}/download", cdn_url, name);

    let plugin_dir = target_dir.join("plugin");
    let archive_path = PathBuf::from(format!("./.deva/tmp/{}.devaplugin", name));
    let extract_path = plugin_dir.join(name);

    if extract_path.exists() {
        println!(
            "Plugin '{}' already exists at '{}'. Skipping install.",
            name,
            extract_path.display()
        );
        return Ok(());
    }

    download_file(&url, &archive_path).await.map_err(|e| format!("Failed to download: {}", e))?;

    extract_archive(&archive_path, &extract_path).await.map_err(|e|
        format!("Failed to extract: {}", e)
    )?;

    // Add the plugin to the config
    let root_dir = target_dir
        .parent()
        .ok_or_else(|| "Failed to determine root directory".to_string())?;

    let config_path = root_dir.join(".devalang");
    if (!config_path.exists()) {
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

    let dependency_path = &format!("devalang://plugin/{}", name);

    add_plugin_to_config(&mut config, &extract_path, &dependency_path);

    Ok(())
}
