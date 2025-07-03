pub mod loader;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub defaults: ConfigDefaults,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigDefaults {
    pub entry: Option<String>,
    pub output: Option<String>,
    pub watch: Option<bool>,
    pub repeat: Option<bool>,
}
