use crate::config::driver::ProjectConfig;
use std::fs;
use std::path::Path;

pub fn load_config(path: Option<&Path>) -> Option<ProjectConfig> {
    let config_path_buf;
    let config_path = match path {
        Some(p) => p,
        None => {
            config_path_buf = match devalang_utils::path::get_devalang_config_path() {
                Ok(p) => p,
                Err(_) => {
                    return None;
                }
            };
            &config_path_buf
        }
    };

    if config_path.exists() {
        let content = fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}
