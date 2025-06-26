use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DevalangConfig {
    pub defaults: Defaults,
}

#[derive(Debug, Deserialize)]
pub struct Defaults {
    pub entry: Option<String>,

    pub output: Option<String>,

    pub watch: Option<bool>,
}
