#![cfg(feature = "cli")]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::tools::cli::config::path::ensure_deva_dir;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddonSearchResult {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub addon_type: String,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub path: PathBuf, // Path to the local .tar.gz file
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddonSearchResponse {
    pub addons: Vec<AddonSearchResult>,
    pub total: usize,
}

/// Discovers local addons in the .deva folder by scanning .tar.gz archives
/// or searches remote addons via API
pub async fn discover_addons(
    search_term: Option<String>,
    addon_type: Option<String>,
    author: Option<String>,
    local: bool,
) -> Result<Vec<AddonSearchResult>> {
    if local {
        discover_local_addons(search_term, addon_type, author).await
    } else {
        discover_remote_addons(search_term, addon_type, author).await
    }
}

/// Discovers local addons in the project .deva directory
async fn discover_local_addons(
    search_term: Option<String>,
    addon_type: Option<String>,
    author: Option<String>,
) -> Result<Vec<AddonSearchResult>> {
    // Get the .deva directory path from the project root
    let deva_dir = ensure_deva_dir()
        .context("Failed to get .deva directory. Run 'devalang init' or create a .deva folder in your project.")?;

    // Recursively scan to find all .tar.gz files
    let mut discovered_addons = vec![];
    walk_dir_collect_tar_gz(&deva_dir, &mut discovered_addons)?;

    // Pre-classify archives by inspecting their content
    for addon in discovered_addons.iter_mut() {
        if let Ok(detected_type) = detect_addon_type_in_archive(&addon.path) {
            if detected_type != "unknown" {
                addon.addon_type = detected_type;
            }
        }
    }

    // Filter according to search criteria
    let mut filtered_addons = discovered_addons;
    
    if let Some(ref term) = search_term {
        let term_lower = term.to_lowercase();
        filtered_addons.retain(|addon| {
            addon.name.to_lowercase().contains(&term_lower)
                || addon.description.to_lowercase().contains(&term_lower)
        });
    }

    if let Some(ref atype) = addon_type {
        let atype_lower = atype.to_lowercase();
        filtered_addons.retain(|addon| addon.addon_type.to_lowercase() == atype_lower);
    }

    if let Some(ref auth) = author {
        let auth_lower = auth.to_lowercase();
        filtered_addons.retain(|addon| addon.author.to_lowercase() == auth_lower);
    }

    Ok(filtered_addons)
}

/// Discovers remote addons via API (placeholder for future implementation)
async fn discover_remote_addons(
    _search_term: Option<String>,
    _addon_type: Option<String>,
    _author: Option<String>,
) -> Result<Vec<AddonSearchResult>> {
    // TODO: Implement remote API discovery
    Err(anyhow::anyhow!(
        "Remote addon discovery not yet implemented. Use --local flag to discover local addons."
    ))
}

/// Recursively walks a directory and collects all .tar.gz files
fn walk_dir_collect_tar_gz(base: &Path, out: &mut Vec<AddonSearchResult>) -> Result<()> {
    if !base.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(base)
        .with_context(|| format!("Failed to read directory: {}", base.display()))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_dir() {
            // Recurse into subdirectories
            walk_dir_collect_tar_gz(&path, out)?;
        } else if path.is_file() {
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");

            if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
                // Extract the name (remove extension)
                let name = if filename.ends_with(".tar.gz") {
                    filename.trim_end_matches(".tar.gz")
                } else {
                    filename.trim_end_matches(".tgz")
                };

                // Try to parse the publisher.name format
                let (author, addon_name) = if let Some(dot_pos) = name.find('.') {
                    let publisher = &name[..dot_pos];
                    let addon = &name[dot_pos + 1..];
                    (publisher.to_string(), addon.to_string())
                } else {
                    // Fallback: use parent folder name as author
                    let parent_name = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    (parent_name.to_string(), name.to_string())
                };

                out.push(AddonSearchResult {
                    slug: format!("{}.{}", author, addon_name),
                    name: addon_name.clone(),
                    description: String::new(), // Will be filled during extraction
                    version: "local".to_string(),
                    author: author.clone(),
                    addon_type: "unknown".to_string(), // Will be detected later
                    downloads: 0,
                    tags: vec![],
                    path: path.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Detects the addon type by inspecting the archive content
fn detect_addon_type_in_archive(archive_path: &Path) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    let file = File::open(archive_path)
        .with_context(|| format!("Failed to open archive: {}", archive_path.display()))?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    // Look for indicator files of the addon type
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        let path_str = path.to_string_lossy();

        // Detect based on present files
        if path_str.contains("bank.toml") {
            return Ok("bank".to_string());
        } else if path_str.contains("plugin.toml") {
            return Ok("plugin".to_string());
        } else if path_str.contains("preset.toml") {
            return Ok("preset".to_string());
        } else if path_str.contains("template.toml") {
            return Ok("template".to_string());
        }
    }

    Ok("unknown".to_string())
}

/// Prompts user to select and install addons interactively
pub async fn prompt_and_install_addons(addons: &[AddonSearchResult], local: bool) -> Result<()> {
    if addons.is_empty() {
        println!("\n❌ No addons available to install.\n");
        return Ok(());
    }

    // Build choice labels
    let choices: Vec<String> = addons
        .iter()
        .map(|addon| {
            let type_emoji = match addon.addon_type.as_str() {
                "bank" => "🥁",
                "plugin" => "🔌",
                "preset" => "🎛️",
                "template" => "📄",
                _ => "📦",
            };
            format!("{} {} by {} ({})", type_emoji, addon.name, addon.author, addon.addon_type)
        })
        .collect();

    // Prompt for multi-selection
    let selected = inquire::MultiSelect::new("Select addons to install:", choices.clone())
        .prompt()
        .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?;

    if selected.is_empty() {
        println!("\n❌ No addons selected.\n");
        return Ok(());
    }

    // Find selected addons
    let to_install: Vec<&AddonSearchResult> = selected
        .iter()
        .filter_map(|choice| {
            addons.iter().find(|addon| {
                let label = format!(
                    "{} {} by {} ({})",
                    match addon.addon_type.as_str() {
                        "bank" => "🥁",
                        "plugin" => "🔌",
                        "preset" => "🎛️",
                        "template" => "📄",
                        _ => "📦",
                    },
                    addon.name,
                    addon.author,
                    addon.addon_type
                );
                &label == choice
            })
        })
        .collect();

    println!("\n📦 Installing {} addon(s)...\n", to_install.len());

    // Install each selected addon
    let mut success_count = 0;
    let mut failed: Vec<String> = vec![];

    for addon in to_install {
        let slug = if local {
            // For local install, use the filename from path
            addon.path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(&addon.slug)
                .to_string()
        } else {
            addon.slug.clone()
        };

        print!("   • Installing {}...", addon.name);
        std::io::Write::flush(&mut std::io::stdout()).ok();

        match super::install::install_addon(slug.clone(), local, false).await {
            Ok(_) => {
                println!(" ✅");
                success_count += 1;
            }
            Err(e) => {
                println!(" ❌");
                failed.push(format!("{}: {}", addon.name, e));
            }
        }
    }

    println!();
    if success_count > 0 {
        println!("✅ Successfully installed {} addon(s)", success_count);
    }
    if !failed.is_empty() {
        println!("❌ Failed to install {} addon(s):", failed.len());
        for f in failed {
            println!("   • {}", f);
        }
    }
    println!();

    Ok(())
}

pub fn display_addon_results(addons: &[AddonSearchResult], local: bool) {
    if addons.is_empty() {
        if local {
            println!("\n❌ No addons found in project .deva directory.\n");
            println!("💡 Tip: Copy compiled addon archives (.tar.gz) to ./.deva/\n");
        } else {
            println!("\n❌ No remote addons found.\n");
        }
        return;
    }

    // Group by type for better display
    let banks: Vec<_> = addons.iter().filter(|a| a.addon_type == "bank").collect();
    let plugins: Vec<_> = addons.iter().filter(|a| a.addon_type == "plugin").collect();
    let presets: Vec<_> = addons.iter().filter(|a| a.addon_type == "preset").collect();
    let templates: Vec<_> = addons.iter().filter(|a| a.addon_type == "template").collect();
    let unknown: Vec<_> = addons.iter().filter(|a| a.addon_type == "unknown").collect();

    let location = if local { "project .deva directory" } else { "remote" };
    println!("\n📦 Found {} addon(s) in {}:\n", addons.len(), location);

    // Display banks
    if !banks.is_empty() {
        println!("🥁 Banks ({})", banks.len());
        for addon in banks {
            println!("   • {} by {}", addon.name, addon.author);
            println!("     Path: {}", addon.path.display());
        }
        println!();
    }

    // Display plugins
    if !plugins.is_empty() {
        println!("� Plugins ({})", plugins.len());
        for addon in plugins {
            println!("   • {} by {}", addon.name, addon.author);
            println!("     Path: {}", addon.path.display());
        }
        println!();
    }

    // Display presets
    if !presets.is_empty() {
        println!("🎛️  Presets ({})", presets.len());
        for addon in presets {
            println!("   • {} by {}", addon.name, addon.author);
            println!("     Path: {}", addon.path.display());
        }
        println!();
    }

    // Display templates
    if !templates.is_empty() {
        println!("📄 Templates ({})", templates.len());
        for addon in templates {
            println!("   • {} by {}", addon.name, addon.author);
            println!("     Path: {}", addon.path.display());
        }
        println!();
    }

    // Display unknown types
    if !unknown.is_empty() {
        println!("❓ Unknown type ({})", unknown.len());
        for addon in unknown {
            println!("   • {} by {}", addon.name, addon.author);
            println!("     Path: {}", addon.path.display());
        }
        println!();
    }

    if local {
        println!("💡 Use 'devalang addon discover --local --install' to install these addons interactively\n");
    } else {
        println!("💡 Use 'devalang addon install <slug>' to install an addon\n");
    }
}
