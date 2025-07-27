use std::path::Path;

use crate::installer::bank::install_bank;

#[derive(Debug, Clone)]
pub enum AddonType {
    Bank,
    Plugin,
    Preset,
}

pub async fn install_addon(
    addon_type: AddonType,
    name: &str,
    target_dir: &Path,
) -> Result<(), String> {
    match addon_type {
        AddonType::Bank => install_bank(name, target_dir).await,
        AddonType::Plugin => Err("Plugin installation not implemented".into()),
        AddonType::Preset => Err("Preset installation not implemented".into()),
    }
}