#![cfg(feature = "cli")]

use crate::tools::cli::config::path::ensure_deva_dir;
use anyhow::Result;
use std::fs;

#[derive(Clone)]
struct InstalledAddon {
    name: String,
    addon_type: String,
}

/// Lists all installed addons
pub async fn list_addons() -> Result<()> {
    let deva_dir = ensure_deva_dir()?;

    let banks_dir = deva_dir.join("banks");
    let plugins_dir = deva_dir.join("plugins");
    let presets_dir = deva_dir.join("presets");
    let templates_dir = deva_dir.join("templates");

    let mut installed_addons = Vec::new();

    // Scan banks
    if banks_dir.exists() {
        if let Ok(entries) = fs::read_dir(&banks_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(publisher) = entry.file_name().to_str() {
                            // Scanner les addons du publisher
                            if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                                for sub_entry in sub_entries.flatten() {
                                    if sub_entry.path().is_dir() {
                                        if let Some(name) = sub_entry.file_name().to_str() {
                                            installed_addons.push(InstalledAddon {
                                                name: format!("{}.{}", publisher, name),
                                                addon_type: "bank".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Scanner les plugins
    if plugins_dir.exists() {
        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(publisher) = entry.file_name().to_str() {
                            // Scanner les addons du publisher
                            if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                                for sub_entry in sub_entries.flatten() {
                                    if sub_entry.path().is_dir() {
                                        if let Some(name) = sub_entry.file_name().to_str() {
                                            installed_addons.push(InstalledAddon {
                                                name: format!("{}.{}", publisher, name),
                                                addon_type: "plugin".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Scanner les presets
    if presets_dir.exists() {
        if let Ok(entries) = fs::read_dir(&presets_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(publisher) = entry.file_name().to_str() {
                            // Scanner les addons du publisher
                            if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                                for sub_entry in sub_entries.flatten() {
                                    if sub_entry.path().is_dir() {
                                        if let Some(name) = sub_entry.file_name().to_str() {
                                            installed_addons.push(InstalledAddon {
                                                name: format!("{}.{}", publisher, name),
                                                addon_type: "preset".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Scanner les templates
    if templates_dir.exists() {
        if let Ok(entries) = fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(publisher) = entry.file_name().to_str() {
                            // Scanner les addons du publisher
                            if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                                for sub_entry in sub_entries.flatten() {
                                    if sub_entry.path().is_dir() {
                                        if let Some(name) = sub_entry.file_name().to_str() {
                                            installed_addons.push(InstalledAddon {
                                                name: format!("{}.{}", publisher, name),
                                                addon_type: "template".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if installed_addons.is_empty() {
        println!("No addon installed.");
    } else {
        let installed_banks: Vec<_> = installed_addons
            .iter()
            .filter(|a| a.addon_type == "bank")
            .collect();

        let installed_plugins: Vec<_> = installed_addons
            .iter()
            .filter(|a| a.addon_type == "plugin")
            .collect();

        let installed_presets: Vec<_> = installed_addons
            .iter()
            .filter(|a| a.addon_type == "preset")
            .collect();

        let installed_templates: Vec<_> = installed_addons
            .iter()
            .filter(|a| a.addon_type == "template")
            .collect();

        if !installed_banks.is_empty() {
            println!("\n📦 Installed banks:");
            for addon in installed_banks {
                println!("  - {}", addon.name);
            }
        }

        if !installed_plugins.is_empty() {
            println!("\n🔌 Installed plugins:");
            for addon in installed_plugins {
                println!("  - {}", addon.name);
            }
        }

        if !installed_presets.is_empty() {
            println!("\n🎨 Installed presets:");
            for addon in installed_presets {
                println!("  - {}", addon.name);
            }
        }

        if !installed_templates.is_empty() {
            println!("\n📝 Installed templates:");
            for addon in installed_templates {
                println!("  - {}", addon.name);
            }
        }

        println!("\nFound {} addon(s).", installed_addons.len());
    }

    Ok(())
}
