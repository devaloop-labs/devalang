use crate::cli::discover::config::add_addons_to_config;
use crate::cli::discover::install::install_selected_addons;
use devalang_types::DiscoveredAddon;
use devalang_utils::{
    logger::{LogLevel, Logger},
    path as path_utils,
    spinner::start_spinner,
};

pub async fn handle_discover_command() -> Result<(), String> {
    let deva_dir = path_utils::ensure_deva_dir()?;

    // Search for addons (banks, plugins, presets, templates) in the .deva directory
    let valid_addons_extensions = ["devabank", "devaplugin", "devapreset", "devatemplate"];

    let mut addons_found = Vec::new();

    // Recursively walk the .deva directory and collect addon files matching
    // the known addon extensions. This allows discovery in nested folders.
    fn walk_dir_collect(base: &std::path::Path, exts: &[&str], out: &mut Vec<DiscoveredAddon>) {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_dir() {
                    walk_dir_collect(&p, exts, out);
                } else if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        if exts.contains(&ext) {
                            let name = p
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let addon_type = match ext {
                                "devabank" => "bank",
                                "devaplugin" => "plugin",
                                "devapreset" => "preset",
                                "devatemplate" => "template",
                                _ => "unknown",
                            };

                            out.push(DiscoveredAddon {
                                path: p.clone(),
                                name,
                                extension: ext.to_string(),
                                addon_type: addon_type.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    walk_dir_collect(&deva_dir, &valid_addons_extensions, &mut addons_found);

    let logger = Logger::new();

    let banks_found = addons_found
        .iter()
        .filter(|addon| addon.addon_type == "bank")
        .cloned()
        .collect::<Vec<_>>();

    let plugins_found = addons_found
        .iter()
        .filter(|addon| addon.addon_type == "plugin")
        .cloned()
        .collect::<Vec<_>>();

    let presets_found = addons_found
        .iter()
        .filter(|addon| addon.addon_type == "preset")
        .cloned()
        .collect::<Vec<_>>();

    let templates_found = addons_found
        .iter()
        .filter(|addon| addon.addon_type == "template")
        .cloned()
        .collect::<Vec<_>>();

    let mut all_addons = Vec::with_capacity(
        banks_found.len() + plugins_found.len() + presets_found.len() + templates_found.len(),
    );
    all_addons.extend(banks_found.iter().cloned());
    all_addons.extend(plugins_found.iter().cloned());
    all_addons.extend(presets_found.iter().cloned());
    all_addons.extend(templates_found.iter().cloned());

    println!();

    if all_addons.is_empty() {
        logger.log_message(LogLevel::Error, "No addons found in the '.deva' folder");
        return Ok(());
    }

    if !banks_found.is_empty() {
        let mut bank_traces: Vec<String> = Vec::new();

        for addon in &banks_found {
            bank_traces.push(addon.name.to_string());
        }

        let trace_refs: Vec<&str> = bank_traces.iter().map(|s| s.as_str()).collect();

        logger.log_message_with_trace(
            LogLevel::Info,
            format!("Found {} compiled banks in workspace", banks_found.len()).as_str(),
            trace_refs,
        );
    }

    if !plugins_found.is_empty() {
        let mut plugin_traces: Vec<String> = Vec::new();

        for addon in &plugins_found {
            plugin_traces.push(addon.name.to_string());
        }

        let trace_refs: Vec<&str> = plugin_traces.iter().map(|s| s.as_str()).collect();

        logger.log_message_with_trace(
            LogLevel::Info,
            format!(
                "Found {} compiled plugins in workspace",
                plugins_found.len()
            )
            .as_str(),
            trace_refs,
        );
    }

    if !presets_found.is_empty() {
        let mut preset_traces: Vec<String> = Vec::new();

        for addon in &presets_found {
            preset_traces.push(addon.name.to_string());
        }

        let trace_refs: Vec<&str> = preset_traces.iter().map(|s| s.as_str()).collect();

        logger.log_message_with_trace(
            LogLevel::Info,
            format!(
                "Found {} compiled presets in workspace",
                presets_found.len()
            )
            .as_str(),
            trace_refs,
        );
    }

    if !templates_found.is_empty() {
        let mut template_traces: Vec<String> = Vec::new();

        for addon in &templates_found {
            template_traces.push(addon.name.to_string());
        }

        let trace_refs: Vec<&str> = template_traces.iter().map(|s| s.as_str()).collect();

        logger.log_message_with_trace(
            LogLevel::Info,
            format!(
                "Found {} compiled templates in workspace",
                templates_found.len()
            )
            .as_str(),
            trace_refs,
        );
    }

    println!();

    // Build user-friendly, unique labels tied to each addon by including the path
    let choice_labels: Vec<String> = all_addons
        .iter()
        .map(|addon| {
            format!(
                "{}: {} ({})",
                addon.addon_type,
                addon.name,
                addon.path.display()
            )
        })
        .collect();

    let selected_addons = match inquire::MultiSelect::new(
        "Select addons to install:",
        choice_labels.clone(),
    )
    .prompt()
    {
        Ok(selected_addons) => selected_addons,
        Err(err) => {
            logger.log_message(
                LogLevel::Error,
                format!("Error selecting addons: {}", err).as_str(),
            );

            return Err(format!("Error selecting addons: {}", err));
        }
    };

    let spinner = start_spinner("Installing addons...");

    let addons_to_install = selected_addons
        .iter()
        .filter_map(|label| {
            all_addons.iter().find(|addon| {
                let candidate = format!(
                    "{}: {} ({})",
                    addon.addon_type,
                    addon.name,
                    addon.path.display()
                );
                &candidate == label
            })
        })
        .cloned()
        .collect::<Vec<_>>();

    let install_selected_addons_result = install_selected_addons(addons_to_install).await;
    match install_selected_addons_result {
        Ok(addons_enriched) => {
            if let Err(e) = add_addons_to_config(addons_enriched).await {
                spinner.finish_and_clear();
                logger.log_message(
                    LogLevel::Error,
                    format!("Failed to add addons to config: {}", e).as_str(),
                );
                return Err(format!("Failed to add addons to config: {}", e));
            }

            spinner.finish_and_clear();
            println!();
            logger.log_message(
                LogLevel::Success,
                "Successfully installed addons !".to_string().as_str(),
            );
        }
        Err(e) => {
            spinner.finish_and_clear();
            logger.log_message(
                LogLevel::Error,
                format!("Failed to install addons: {}", e).as_str(),
            );
            return Err(format!("Failed to install addons: {}", e));
        }
    }

    Ok(())
}
