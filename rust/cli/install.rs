use crate::installer::addon::{install_addon, AddonType};

#[cfg(feature = "cli")]
pub async fn handle_install_command(name: String, addon_type: AddonType) -> Result<(), String> {
    let deva_dir = std::path::Path::new("./.deva/");

    println!("⬇️  Installing {:?} '{}'...", addon_type, name);
    println!("📂 Target directory: {}", deva_dir.display());

    if let Err(e) = install_addon(addon_type.clone(), name.as_str(), deva_dir).await {
        eprintln!("❌ Error installing {:?} '{}': {}", addon_type, name, e);
    } else {
        println!("✅ {:?} '{}' installed successfully!", addon_type, name);
    }

    Ok(())
}
