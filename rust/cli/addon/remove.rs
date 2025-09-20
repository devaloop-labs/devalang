use crate::config::ops::load_config;
use devalang_core::config::driver::ProjectConfigExt;
use devalang_utils::path as path_utils;
use std::fs;

pub async fn remove_addon(name: String) -> Result<(), String> {
    let deva_dir = path_utils::ensure_deva_dir()?;

    // Helper to extract publisher from a slug like 'publisher.name'
    let extract_publisher = |s: &str| s.splitn(2, '.').next().unwrap_or("").to_string();

    // Try to find in config first (banks/plugins)
    if let Ok(config_path) = path_utils::get_devalang_config_path() {
        if let Some(mut config) = load_config(Some(&config_path)) {
            // BANKS
            if let Some(banks) = config.banks.as_mut() {
                if let Some(pos) = banks.iter().position(|b| {
                    let slug = b.path.strip_prefix("devalang://bank/").unwrap_or(&b.path);
                    // accept exact slug, exact path, or match by local name suffix (publisher/name or publisher.name)
                    slug == name
                        || b.path == name
                        || slug.ends_with(&format!("/{}", name))
                        || slug.ends_with(&format!(".{}", name))
                }) {
                    let entry = banks.remove(pos);
                    let slug = entry
                        .path
                        .strip_prefix("devalang://bank/")
                        .unwrap_or(&entry.path)
                        .to_string();

                    // parse publisher/name from slug (support 'publisher/name' or 'publisher.name')
                    let (publisher, local_name) = if slug.contains('/') {
                        let mut it = slug.splitn(2, '/');
                        (
                            it.next().unwrap().to_string(),
                            it.next().unwrap().to_string(),
                        )
                    } else if slug.contains('.') {
                        let mut it = slug.splitn(2, '.');
                        (
                            it.next().unwrap().to_string(),
                            it.next().unwrap().to_string(),
                        )
                    } else {
                        return Err(format!("Cannot parse bank slug '{}'", slug));
                    };

                    let local_path = deva_dir.join("banks").join(&publisher).join(&local_name);

                    if !local_path.exists() {
                        return Err(format!(
                            "Local files for bank '{}' not found at '{}', aborting",
                            slug,
                            local_path.display()
                        ));
                    }

                    fs::remove_dir_all(&local_path)
                        .map_err(|e| format!("Failed to remove addon files: {}", e))?;

                    if let Err(e) = config.write_config(&config) {
                        eprintln!("Warning: failed to write updated config: {}", e);
                    }

                    println!("✅ Bank '{}' removed (publisher '{}')", slug, publisher);
                    return Ok(());
                }
            }

            // PLUGINS
            if let Some(plugins) = config.plugins.as_mut() {
                if let Some(pos) = plugins.iter().position(|p| {
                    let slug = p.path.strip_prefix("devalang://plugin/").unwrap_or(&p.path);
                    // accept exact slug, exact path, or match by local name suffix
                    slug == name
                        || p.path == name
                        || slug.ends_with(&format!("/{}", name))
                        || slug.ends_with(&format!(".{}", name))
                }) {
                    let entry = plugins.remove(pos);
                    let slug = entry
                        .path
                        .strip_prefix("devalang://plugin/")
                        .unwrap_or(&entry.path)
                        .to_string();

                    // parse publisher/name from slug (support 'publisher/name' or 'publisher.name')
                    let (publisher, local_name) = if slug.contains('/') {
                        let mut it = slug.splitn(2, '/');
                        (
                            it.next().unwrap().to_string(),
                            it.next().unwrap().to_string(),
                        )
                    } else if slug.contains('.') {
                        let mut it = slug.splitn(2, '.');
                        (
                            it.next().unwrap().to_string(),
                            it.next().unwrap().to_string(),
                        )
                    } else {
                        return Err(format!("Cannot parse plugin slug '{}'", slug));
                    };

                    let local_path = deva_dir.join("plugins").join(&publisher).join(&local_name);

                    if !local_path.exists() {
                        return Err(format!(
                            "Local files for plugin '{}' not found at '{}', aborting",
                            slug,
                            local_path.display()
                        ));
                    }

                    fs::remove_dir_all(&local_path)
                        .map_err(|e| format!("Failed to remove addon files: {}", e))?;

                    if let Err(e) = config.write_config(&config) {
                        eprintln!("Warning: failed to write updated config: {}", e);
                    }

                    println!("✅ Plugin '{}' removed (publisher '{}')", slug, publisher);
                    return Ok(());
                }
            }
        }
    }

    // Not found in config: search filesystem under .deva and infer type
    let dirs = ["banks", "plugins", "presets", "templates"];
    for &d in &dirs {
        let folder = deva_dir.join(d);
        if !folder.exists() {
            continue;
        }

        // If name looks like a slug (contains '.' or '/'), parse publisher and name and try candidate paths
        if name.contains('.') || name.contains('/') {
            let (publisher, local_name) = if name.contains('/') {
                let mut it = name.splitn(2, '/');
                (
                    it.next().unwrap().to_string(),
                    it.next().unwrap().to_string(),
                )
            } else {
                let mut it = name.splitn(2, '.');
                (
                    it.next().unwrap().to_string(),
                    it.next().unwrap().to_string(),
                )
            };

            let candidate1 = folder.join(&publisher).join(&local_name);
            let candidate2 = folder.join(format!("{}.{}", publisher, local_name));
            let candidate3 = folder.join(&name);

            let candidate = if candidate1.exists() {
                candidate1
            } else if candidate2.exists() {
                candidate2
            } else {
                candidate3
            };

            if candidate.exists() {
                fs::remove_dir_all(&candidate)
                    .map_err(|e| format!("Failed to remove addon files: {}", e))?;

                // also attempt to remove from config if possible
                if let Ok(config_path) = path_utils::get_devalang_config_path() {
                    if let Some(mut config) = load_config(Some(&config_path)) {
                        match d {
                            "banks" => {
                                if let Some(banks) = config.banks.as_mut() {
                                    let pattern1 =
                                        format!("devalang://bank/{}/{}", publisher, local_name);
                                    let pattern2 =
                                        format!("devalang://bank/{}.{}", publisher, local_name);
                                    banks.retain(|b| {
                                        b.path != pattern1
                                            && b.path != pattern2
                                            && !b.path.ends_with(&local_name)
                                    });
                                }
                            }
                            "plugins" => {
                                if let Some(plugins) = config.plugins.as_mut() {
                                    let pattern1 =
                                        format!("devalang://plugin/{}/{}", publisher, local_name);
                                    let pattern2 =
                                        format!("devalang://plugin/{}.{}", publisher, local_name);
                                    plugins.retain(|p| {
                                        p.path != pattern1
                                            && p.path != pattern2
                                            && !p.path.ends_with(&local_name)
                                    });
                                }
                            }
                            _ => {}
                        }

                        if let Err(e) = config.write_config(&config) {
                            eprintln!("Warning: failed to write updated config: {}", e);
                        }
                    }
                }

                println!(
                    "✅ Addon '{}/{}' removed from .deva/{} (publisher '{}')",
                    publisher, local_name, d, publisher
                );
                return Ok(());
            }
        }

        // Otherwise, scan directory entries: match exact name or suffix '.name'
        if let Ok(entries) = fs::read_dir(&folder) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if !file_type.is_dir() {
                        continue;
                    }
                }
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if file_name == name || file_name.ends_with(&format!(".{}", name)) {
                    let slug = file_name.to_string();
                    let publisher = extract_publisher(&slug);
                    let path = entry.path();
                    fs::remove_dir_all(&path)
                        .map_err(|e| format!("Failed to remove addon files: {}", e))?;

                    // try to remove from config when banks/plugins
                    if let Ok(config_path) = path_utils::get_devalang_config_path() {
                        if let Some(mut config) = load_config(Some(&config_path)) {
                            match d {
                                "banks" => {
                                    if let Some(banks) = config.banks.as_mut() {
                                        banks.retain(|b| {
                                            b.path != format!("devalang://bank/{}", slug)
                                        });
                                    }
                                }
                                "plugins" => {
                                    if let Some(plugins) = config.plugins.as_mut() {
                                        plugins.retain(|p| {
                                            p.path != format!("devalang://plugin/{}", slug)
                                        });
                                    }
                                }
                                _ => {}
                            }

                            if let Err(e) = config.write_config(&config) {
                                eprintln!("Warning: failed to write updated config: {}", e);
                            }
                        }
                    }

                    println!(
                        "✅ Addon '{}' removed from .deva/{} (publisher '{}')",
                        slug, d, publisher
                    );
                    return Ok(());
                }
            }
        }
    }

    Err(format!("Addon '{}' not found", name))
}
