use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ProjectConfig {
    pub defaults: ProjectConfigDefaults,
    pub banks: Option<Vec<ProjectConfigBankEntry>>,
    pub plugins: Option<Vec<PluginEntry>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ProjectConfigDefaults {
    pub entry: Option<String>,
    pub output: Option<String>,
    pub watch: Option<bool>,
    pub repeat: Option<bool>,
    pub debug: Option<bool>,
    pub compress: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ProjectConfigBankMetadata {
    pub bank: HashMap<String, String>,
    pub triggers: Option<Vec<HashMap<String, String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProjectConfigBankEntry {
    pub path: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PluginEntry {
    pub path: String,
    pub version: String,
    pub author: String,
    pub access: String,
}

impl ProjectConfig {
    pub fn new() -> Self {
        ProjectConfig {
            defaults: ProjectConfigDefaults {
                entry: None,
                output: None,
                watch: None,
                repeat: None,
                debug: None,
                compress: None,
            },
            banks: Some(Vec::new()),
            plugins: Some(Vec::new()),
        }
    }

    pub fn with_defaults(
        entry: Option<String>,
        output: Option<String>,
        watch: Option<bool>,
        repeat: Option<bool>,
        debug: Option<bool>,
        compress: Option<bool>,
    ) -> Self {
        ProjectConfig {
            defaults: ProjectConfigDefaults {
                entry,
                output,
                watch,
                repeat,
                debug,
                compress,
            },
            banks: Some(Vec::new()),
            plugins: Some(Vec::new()),
        }
    }

    pub fn get() -> Result<ProjectConfig, String> {
        let root = std::env::current_dir().unwrap();
        let config_path = root.join(".devalang");

        if config_path.try_exists().is_err() {
            return Err(format!(
                "Config file not found at path: {}",
                config_path.display()
            ));
        }

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: ProjectConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    pub fn from_string(config_string: &str) -> Result<(Self, String), String> {
        let config: ProjectConfig = toml::from_str(config_string)
            .map_err(|e| format!("Failed to parse config string: {}", e))?;
        let config_path = ".devalang".to_string();

        Ok((config, config_path))
    }

    pub fn write(&self, new_config: &ProjectConfig) -> Result<(), String> {
        let config_path = ".devalang";

        let content = toml::to_string(new_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(config_path, content)
            .map_err(|e| format!("Failed to write config to file '{}': {}", config_path, e))?;

        Ok(())
    }
}
