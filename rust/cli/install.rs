use crate::installer::bank::install_bank;

#[cfg(feature = "cli")]
pub async fn handle_install_bank_command(name: String) -> Result<(), String> {
    let deva_dir = std::path::Path::new("./.deva/");

    println!("⬇️  Installing bank '{}'...", name);
    println!("📂 Target directory: {}", deva_dir.display());

    if let Err(e) = install_bank(name.as_str(), deva_dir).await {
        eprintln!("❌ Error installing bank '{}': {}", name, e);
    } else {
        println!("✅ Bank '{}' installed successfully!", name);
    }

    Ok(())
}
