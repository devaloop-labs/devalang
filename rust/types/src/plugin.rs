use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginExport {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub default: Option<TomlValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginInfo {
    pub author: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub exports: Vec<PluginExport>,
}
