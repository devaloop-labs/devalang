use devalang_utils::logger::{LogLevel, Logger};

#[derive(Clone)]
struct InstalledAddon {
    name: String,
    addon_type: String,
}

pub async fn list_addons() -> Result<(), String> {
    let deva_dir = devalang_utils::path::ensure_deva_dir()?;
    let banks_dir = deva_dir.join("banks");
    let plugins_dir = deva_dir.join("plugins");
    let presets_dir = deva_dir.join("presets");
    let templates_dir = deva_dir.join("templates");

    let mut installed_addons = Vec::new();

    if banks_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&banks_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            installed_addons.push(InstalledAddon {
                                name: name.to_string(),
                                addon_type: "bank".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if plugins_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            installed_addons.push(InstalledAddon {
                                name: name.to_string(),
                                addon_type: "plugin".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if presets_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&presets_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            installed_addons.push(InstalledAddon {
                                name: name.to_string(),
                                addon_type: "preset".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if templates_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            installed_addons.push(InstalledAddon {
                                name: name.to_string(),
                                addon_type: "template".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    let logger = Logger::new();

    if installed_addons.is_empty() {
        logger.log_message(LogLevel::Info, "No addon installed.");
    } else {
        let installed_banks = installed_addons.iter().filter(|a| a.addon_type == "bank");

        let installed_plugins = installed_addons.iter().filter(|a| a.addon_type == "plugin");

        let installed_presets = installed_addons.iter().filter(|a| a.addon_type == "preset");

        let installed_templates = installed_addons
            .iter()
            .filter(|a| a.addon_type == "template");

        if installed_banks.clone().count() > 0 {
            let trace: Vec<String> = installed_addons
                .iter()
                .map(|a| format!("{}", a.name))
                .collect();
            let trace: Vec<&str> = trace.iter().map(|s| s.as_str()).collect();

            logger.log_message_with_trace(LogLevel::Info, "Installed banks :", trace);
        }

        if installed_plugins.clone().count() > 0 {
            let trace: Vec<String> = installed_addons
                .iter()
                .map(|a| format!("{}", a.name))
                .collect();
            let trace: Vec<&str> = trace.iter().map(|s| s.as_str()).collect();

            logger.log_message_with_trace(LogLevel::Info, "Installed plugins :", trace);
        }

        if installed_presets.clone().count() > 0 {
            let trace: Vec<String> = installed_addons
                .iter()
                .map(|a| format!("{}", a.name))
                .collect();
            let trace: Vec<&str> = trace.iter().map(|s| s.as_str()).collect();

            logger.log_message_with_trace(LogLevel::Info, "Installed presets :", trace);
        }

        if installed_templates.clone().count() > 0 {
            let trace: Vec<String> = installed_addons
                .iter()
                .map(|a| format!("{}", a.name))
                .collect();
            let trace: Vec<&str> = trace.iter().map(|s| s.as_str()).collect();

            logger.log_message_with_trace(LogLevel::Info, "Installed templates :", trace);
        }

        if let Some(addon_not_in_config) = find_unused_addons(installed_addons.clone()) {
            let trace: Vec<String> = addon_not_in_config
                .iter()
                .map(|a| format!("{} ({})", a.name, a.addon_type))
                .collect();
            let trace: Vec<&str> = trace.iter().map(|s| s.as_str()).collect();

            logger.log_message_with_trace(
                LogLevel::Warning,
                "Some addons are installed but not referenced in the .devalang configuration file :",
                trace,
            );
        }

        logger.log_message(
            LogLevel::Success,
            format!("Found {} installed addon(s).", installed_addons.len()).as_str(),
        );
    }

    Ok(())
}

fn find_unused_addons(addons: Vec<InstalledAddon>) -> Option<Vec<InstalledAddon>> {
    let mut unused_addons = Vec::new();

    let config_path = match devalang_utils::path::get_devalang_config_path() {
        Ok(path) => path,
        Err(_) => return None,
    };

    let config = match crate::config::ops::load_config(Some(&config_path)) {
        Some(cfg) => cfg,
        None => return None,
    };

    let mut referenced_addons = Vec::new();

    if let Some(banks) = config.banks {
        for bank in banks {
            if let Some(name) = bank.path.strip_prefix("devalang://bank/") {
                referenced_addons.push((name.to_string(), "bank".to_string()));
            }
        }
    }

    if let Some(plugins) = config.plugins {
        for plugin in plugins {
            if let Some(name) = plugin.path.strip_prefix("devalang://plugin/") {
                referenced_addons.push((name.to_string(), "plugin".to_string()));
            }
        }
    }

    // TODO: Enable when presets and templates are supported in config
    // if let Some(presets) = config.presets {
    //     for preset in presets {
    //         if let Some(name) = preset.path.strip_prefix("devalang://preset/") {
    //             referenced_addons.push((name.to_string(), "preset".to_string()));
    //         }
    //     }
    // }

    // TODO: Enable when presets and templates are supported in config
    // if let Some(templates) = config.templates {
    //     for template in templates {
    //         if let Some(name) = template.path.strip_prefix("devalang://template/") {
    //             referenced_addons.push((name.to_string(), "template".to_string()));
    //         }
    //     }
    // }

    for addon in addons {
        if !referenced_addons.contains(&(addon.name.clone(), addon.addon_type.clone())) {
            unused_addons.push(addon);
        }
    }

    if unused_addons.is_empty() {
        None
    } else {
        Some(unused_addons)
    }
}
