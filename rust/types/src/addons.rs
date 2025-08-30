use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct DiscoveredAddon {
    pub path: std::path::PathBuf,
    pub name: String,
    pub extension: String,
    pub addon_type: String,
}

#[derive(Debug, Clone)]
pub struct AddonWithMetadata {
    pub name: String,
    pub path: String,
    pub addon_type: String,
    pub metadata: AddonMetadata,
}

#[derive(Debug, Clone)]
pub struct AddonMetadata {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub access: String,
}

#[derive(Debug, Clone)]
pub enum AddonType {
    Bank,
    Plugin,
    Preset,
    Template,
}

#[derive(Debug, Deserialize)]
pub struct BankInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

#[derive(Debug, Deserialize)]
pub struct BankFile {
    pub bank: BankInfo,
    pub triggers: Option<Vec<BankTrigger>>,
    pub audio_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BankTrigger {
    pub name: String,
    pub path: String,
}
