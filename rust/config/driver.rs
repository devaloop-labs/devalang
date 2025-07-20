use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Config {
    pub defaults: ConfigDefaults,
    pub banks: Option<Vec<BankEntry>>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ConfigDefaults {
    pub entry: Option<String>,
    pub output: Option<String>,
    pub watch: Option<bool>,
    pub repeat: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BankEntry {
    pub path: String,
    pub version: Option<String>,
    pub author: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            defaults: ConfigDefaults {
                entry: None,
                output: None,
                watch: None,
                repeat: None,
            },
            banks: Some(Vec::new()),
        }
    }

    pub fn with_defaults(
        entry: Option<String>,
        output: Option<String>,
        watch: Option<bool>,
        repeat: Option<bool>
    ) -> Self {
        Config {
            defaults: ConfigDefaults {
                entry,
                output,
                watch,
                repeat,
            },
            banks: Some(Vec::new()),
        }
    }

    pub fn from_string(config_string: &str) -> Result<(Self, String), String> {
        let config: Config = toml
            ::from_str(config_string)
            .map_err(|e| format!("Failed to parse config string: {}", e))?;
        let config_path = ".devalang".to_string();

        Ok((config, config_path))
    }

    pub fn write(&self, new_config: &Config) -> Result<(), String> {
        let config_path = ".devalang";

        let content = toml
            ::to_string(new_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs
            ::write(config_path, content)
            .map_err(|e| format!("Failed to write config to file '{}': {}", config_path, e))?;

        Ok(())
    }
}
