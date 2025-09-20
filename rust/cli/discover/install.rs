use devalang_types::{AddonMetadata, AddonWithMetadata, DiscoveredAddon};
use devalang_utils::path as path_utils;

pub async fn install_selected_addons(
    addons: Vec<DiscoveredAddon>,
    no_clear_tmp: bool,
) -> Result<Vec<AddonWithMetadata>, String> {
    let mut addons_enriched = Vec::new();

    let tmp_dir = path_utils::ensure_deva_dir()?.join("tmp");

    for addon in addons {
        std::fs::create_dir_all(tmp_dir.join(&addon.name))
            .map_err(|e| format!("Failed to create directory for addon {}: {}", addon.name, e))?;

        let addon_path = tmp_dir.join(&addon.name);
        devalang_utils::file::extract_zip_safely(&addon.path, &addon_path)
            .map_err(|e| format!("Failed to extract addon {}: {}", addon.name, e))?;

        // Read metadata from the extracted addon first so we can layout as <publisher>/<name>
        // If the addon type is unknown (we found a .tar.gz), try to detect the
        // metadata file inside the extracted folder to discover the real type.
        let mut detected_addon_type = addon.addon_type.clone();
        let mut parsed_meta_tmp = None;

        if detected_addon_type == "unknown" {
            // try detection by checking common metadata filenames
            let candidates = ["bank.toml", "plugin.toml", "preset.toml", "template.toml"];
            for candidate in &candidates {
                let path = addon_path.join(candidate);
                if path.exists() {
                    detected_addon_type = match *candidate {
                        "bank.toml" => "bank".to_string(),
                        "plugin.toml" => "plugin".to_string(),
                        "preset.toml" => "preset".to_string(),
                        "template.toml" => "template".to_string(),
                        _ => "unknown".to_string(),
                    };
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        parsed_meta_tmp = crate::cli::discover::metadata::parse_metadata_file(
                            &detected_addon_type,
                            &content,
                        );
                    }
                    break;
                }
            }
        } else {
            let addon_metadata_filename = match addon.addon_type.as_str() {
                "bank" => "bank.toml",
                "plugin" => "plugin.toml",
                "preset" => "preset.toml",
                "template" => "template.toml",
                _ => "",
            };
            if !addon_metadata_filename.is_empty() {
                let addon_metadata_path_tmp = addon_path.join(addon_metadata_filename);
                let addon_metadata_content_tmp =
                    std::fs::read_to_string(&addon_metadata_path_tmp).ok();
                if let Some(content) = addon_metadata_content_tmp {
                    parsed_meta_tmp = crate::cli::discover::metadata::parse_metadata_file(
                        &addon.addon_type,
                        &content,
                    );
                }
            }
        }

        let base = path_utils::ensure_deva_dir()?;
        // Use detected_addon_type (may have been found from metadata)
        // final_addon_type is what we will report and use for metadata lookup
        let final_addon_type = if detected_addon_type == "unknown" {
            addon.addon_type.clone()
        } else {
            detected_addon_type.clone()
        };

        let target_addon_dir = match final_addon_type.as_str() {
            "bank" => base.join("banks"),
            "plugin" => base.join("plugins"),
            "preset" => base.join("presets"),
            "template" => base.join("templates"),
            _ => {
                // Fallback: place unknown archives into plugins by default
                base.join("plugins")
            }
        };

        std::fs::create_dir_all(&target_addon_dir).map_err(|e| {
            format!(
                "Failed to create target directory for addon {}: {}",
                addon.name, e
            )
        })?;

        // prefer parsed metadata publisher/name, fall back to discovered publisher/name
        let publisher_folder = parsed_meta_tmp
            .as_ref()
            .and_then(|m| {
                if !m.publisher.is_empty() {
                    Some(m.publisher.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| addon.publisher.clone());

        let name_folder = parsed_meta_tmp
            .as_ref()
            .and_then(|m| {
                if !m.name.is_empty() {
                    Some(m.name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| addon.name.clone());

        let target_addon_path_dir = target_addon_dir.join(&publisher_folder).join(&name_folder);
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
        // Attempt to read final metadata file (use parsed_tmp as fallback)
        let addon_metadata_filename = match final_addon_type.as_str() {
            "bank" => "bank.toml",
            "plugin" => "plugin.toml",
            "preset" => "preset.toml",
            "template" => "template.toml",
            _ => "",
        };

        let parsed_meta = if addon_metadata_filename.is_empty() {
            parsed_meta_tmp.unwrap_or(AddonMetadata {
                name: "".to_string(),
                publisher: "".to_string(),
                version: "".to_string(),
                description: "".to_string(),
                access: "".to_string(),
            })
        } else {
            let addon_metadata_path = target_addon_path_dir.join(addon_metadata_filename);
            if let Ok(content) = std::fs::read_to_string(&addon_metadata_path) {
                crate::cli::discover::metadata::parse_metadata_file(&final_addon_type, &content)
                    .unwrap_or(parsed_meta_tmp.unwrap_or(AddonMetadata {
                        name: "".to_string(),
                        publisher: "".to_string(),
                        version: "".to_string(),
                        description: "".to_string(),
                        access: "".to_string(),
                    }))
            } else {
                parsed_meta_tmp.unwrap_or(AddonMetadata {
                    name: "".to_string(),
                    publisher: "".to_string(),
                    version: "".to_string(),
                    description: "".to_string(),
                    access: "".to_string(),
                })
            }
        };

        // Record location as the final moved directory path
        addons_enriched.push(AddonWithMetadata {
            name: name_folder.clone(),
            publisher: publisher_folder.clone(),
            path: target_addon_path_dir.clone().to_string_lossy().to_string(),
            addon_type: final_addon_type.clone(),
            metadata: parsed_meta,
        });

        if let Err(e) = std::fs::remove_file(&addon.path) {
            eprintln!(
                "Failed to remove zipped file for addon {}: {}",
                addon.name, e
            );
        }
    }

    // Best-effort cleanup of temporary extraction directory: remove `.deva/tmp`
    // only if it's empty after installing the selected addons and when the
    // user did not pass --no-clear-tmp.
    if !no_clear_tmp {
        if tmp_dir.exists() {
            match std::fs::read_dir(&tmp_dir) {
                Ok(mut rd) => {
                    if rd.next().is_none() {
                        let _ = std::fs::remove_dir_all(&tmp_dir);
                    }
                }
                Err(_) => {
                    // ignore errors here - best-effort
                }
            }
        }
    }

    Ok(addons_enriched)
}
