use devalang_types::{AddonMetadata, AddonWithMetadata, DiscoveredAddon};
use devalang_utils::path as path_utils;

pub async fn install_selected_addons(
    addons: Vec<DiscoveredAddon>,
) -> Result<Vec<AddonWithMetadata>, String> {
    let mut addons_enriched = Vec::new();

    let tmp_dir = path_utils::ensure_deva_dir()?.join("tmp");

    for addon in addons {
        std::fs::create_dir_all(tmp_dir.join(&addon.name))
            .map_err(|e| format!("Failed to create directory for addon {}: {}", addon.name, e))?;

        let addon_path = tmp_dir.join(&addon.name);
        devalang_utils::file::extract_zip_safely(&addon.path, &addon_path)
            .map_err(|e| format!("Failed to extract addon {}: {}", addon.name, e))?;

        let base = path_utils::ensure_deva_dir()?;
        let target_addon_dir = match addon.addon_type.as_str() {
            "bank" => base.join("banks"),
            "plugin" => base.join("plugins"),
            "preset" => base.join("presets"),
            "template" => base.join("templates"),
            _ => {
                return Err(format!("Unknown addon type for addon {}", addon.name));
            }
        };

        std::fs::create_dir_all(&target_addon_dir).map_err(|e| {
            format!(
                "Failed to create target directory for addon {}: {}",
                addon.name, e
            )
        })?;

        let target_addon_path_dir = target_addon_dir.join(&addon.name);
        if target_addon_path_dir.exists() {
            println!(
                "Target addon directory {} already exists",
                target_addon_path_dir.display()
            );
            continue;
        }

        if let Err(e) = std::fs::rename(&addon_path, &target_addon_path_dir) {
            crate::cli::discover::fs::copy_dir_all(&addon_path, &target_addon_path_dir).map_err(
                |err| {
                    format!(
                        "Failed to move addon {}: {} (rename error: {})",
                        addon.name, err, e
                    )
                },
            )?;
            let _ = std::fs::remove_dir_all(&addon_path);
        }

        let addon_metadata_filename = match addon.addon_type.as_str() {
            "bank" => "bank.toml",
            "plugin" => "plugin.toml",
            "preset" => "preset.toml",
            "template" => "template.toml",
            _ => {
                return Err(format!("Unknown addon type for addon {}", addon.name));
            }
        };

        let addon_metadata_path = target_addon_path_dir.join(addon_metadata_filename);
        let addon_metadata_content = std::fs::read_to_string(&addon_metadata_path)
            .map_err(|e| format!("Failed to read metadata for addon {}: {}", addon.name, e))?;

        let parsed_meta = crate::cli::discover::metadata::parse_metadata_file(
            &addon.addon_type,
            &addon_metadata_content,
        )
        .unwrap_or(AddonMetadata {
            name: addon.name.clone(),
            author: "unknown".to_string(),
            version: "".to_string(),
            description: "".to_string(),
            access: "".to_string(),
        });

        addons_enriched.push(AddonWithMetadata {
            name: addon.name.clone(),
            path: addon.path.clone().to_string_lossy().to_string(),
            addon_type: addon.addon_type.clone(),
            metadata: parsed_meta,
        });

        if let Err(e) = std::fs::remove_file(&addon.path) {
            eprintln!(
                "Failed to remove zipped file for addon {}: {}",
                addon.name, e
            );
        }
    }

    // Best-effort cleanup of temporary extraction directory
    let _ = std::fs::remove_dir_all(&tmp_dir);

    Ok(addons_enriched)
}
