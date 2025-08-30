use devalang_core::config::driver::ProjectConfigExt;
use devalang_types::{AddonWithMetadata, ProjectConfigBankEntry, ProjectConfigPluginEntry};
use devalang_utils::path as path_utils;

pub async fn add_addons_to_config(addons: Vec<AddonWithMetadata>) -> Result<(), String> {
    let config_path = path_utils::get_devalang_config_path()?;
    let mut config = crate::config::ops::load_config(Some(&config_path))
        .ok_or_else(|| format!("Failed to load config from '{}'", config_path.display()))?;

    for addon in addons {
        let addon_path_as_devalang_protocol = format!(
            "devalang://{}/{}.{}",
            addon.addon_type, addon.metadata.author, addon.metadata.name
        );

        match addon.addon_type.as_str() {
            "bank" => {
                if config.banks.is_none() {
                    config.banks = Some(Vec::new());
                }

                let banks = config.banks.as_mut().unwrap();

                let exists = banks
                    .iter()
                    .any(|b| b.path == addon_path_as_devalang_protocol);
                if exists {
                    println!("Bank '{}' already in config", addon.name);
                    continue;
                }

                banks.push(ProjectConfigBankEntry {
                    path: addon_path_as_devalang_protocol,
                    version: Some(addon.metadata.version.clone()),
                });
            }

            "plugin" => {
                if config.plugins.is_none() {
                    config.plugins = Some(Vec::new());
                }

                let plugins = config.plugins.as_mut().unwrap();

                let exists = plugins
                    .iter()
                    .any(|p| p.path == addon_path_as_devalang_protocol);
                if exists {
                    println!("Plugin '{}' already in config", addon.name);
                    continue;
                }

                plugins.push(ProjectConfigPluginEntry {
                    path: addon_path_as_devalang_protocol,
                    version: Some(addon.metadata.version.clone()),
                });
            }

            // "preset" => {
            //     if config.presets.is_none() {
            //         config.presets = Some(Vec::new());
            //     }

            //     let presets = config.presets.as_mut().unwrap();

            //     let exists = presets.iter().any(|p| p.path == addon_path_as_deva_protocol);
            //     if exists {
            //         println!("Preset '{}' already in config", addon.name);
            //         continue;
            //     }

            //     presets.push(ProjectConfigPresetEntry {
            //         path: addon_path_as_deva_protocol,
            //         version: Some(addon.metadata.version.clone()),
            //     });
            // }

            // "template" => {
            //     if config.templates.is_none() {
            //         config.templates = Some(Vec::new());
            //     }

            //     let templates = config.templates.as_mut().unwrap();

            //     let exists = templates.iter().any(|t| t.path == addon_path_as_deva_protocol);
            //     if exists {
            //         println!("Template '{}' already in config", addon.name);
            //         continue;
            //     }

            //     templates.push(ProjectConfigTemplateEntry {
            //         path: addon_path_as_deva_protocol,
            //         version: Some(addon.metadata.version.clone()),
            //     });
            // }
            _ => {
                println!(
                    "Unknown addon type '{}' for addon '{}'",
                    addon.addon_type, addon.name
                );
            }
        }
    }

    // Update config with new addons
    if let Err(e) = config.write_config(&config) {
        return Err(format!("Failed to write config: {}", e));
    }

    Ok(())
}
