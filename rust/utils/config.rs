use std::{ fs, path::Path };
use crate::core::types::config::DevalangConfig;

pub fn load_config(path: Option<&Path>) -> Option<DevalangConfig> {
    let config_path = path.unwrap_or_else(|| Path::new(".devalang"));

    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}
