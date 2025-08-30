use devalang_types::{
    PluginEntry as SharedPluginEntry, ProjectConfig as SharedProjectConfig,
    ProjectConfigBankEntry as SharedProjectConfigBankEntry,
    ProjectConfigDefaults as SharedProjectConfigDefaults,
    ProjectConfigPluginEntry as SharedProjectConfigPluginEntry,
};
use devalang_utils::path as path_utils;
use std::path::PathBuf;

pub type ProjectConfig = SharedProjectConfig;
pub type ProjectConfigDefaults = SharedProjectConfigDefaults;
pub type ProjectConfigBankEntry = SharedProjectConfigBankEntry;
pub type ProjectConfigPluginEntry = SharedProjectConfigPluginEntry;
pub type PluginEntry = SharedPluginEntry;

pub trait ProjectConfigExt {
    fn new_config() -> Self;
    fn with_defaults(
        entry: Option<String>,
        output: Option<String>,
        watch: Option<bool>,
        repeat: Option<bool>,
        debug: Option<bool>,
        compress: Option<bool>,
    ) -> Self;
    fn get() -> Result<Self, String>
    where
        Self: Sized;
    fn from_string(config_string: &str) -> Result<(Self, String), String>
    where
        Self: Sized;
    fn write_config(&self, new_config: &Self) -> Result<(), String>
    where
        Self: Sized;
}

impl ProjectConfigExt for SharedProjectConfig {
    fn new_config() -> Self {
        SharedProjectConfig::default()
    }

    fn with_defaults(
        entry: Option<String>,
        output: Option<String>,
        watch: Option<bool>,
        repeat: Option<bool>,
        debug: Option<bool>,
        compress: Option<bool>,
    ) -> Self {
        SharedProjectConfig {
            defaults: SharedProjectConfigDefaults {
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

    fn get() -> Result<Self, String> {
        let config_path = path_utils::get_devalang_config_path()?;

        let config_content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: SharedProjectConfig = toml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    fn from_string(config_string: &str) -> Result<(Self, String), String> {
        let config: SharedProjectConfig = toml::from_str(config_string)
            .map_err(|e| format!("Failed to parse config string: {}", e))?;
        let config_path = ".devalang".to_string();

        Ok((config, config_path))
    }

    fn write_config(&self, new_config: &Self) -> Result<(), String> {
        let config_path: PathBuf = match path_utils::get_project_root() {
            Ok(root) => root.join(path_utils::DEVALANG_CONFIG),
            Err(_) => PathBuf::from(path_utils::DEVALANG_CONFIG),
        };

        let content = toml::to_string(new_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&config_path, content).map_err(|e| {
            format!(
                "Failed to write config to file '{}': {}",
                config_path.display(),
                e
            )
        })?;

        Ok(())
    }
}
