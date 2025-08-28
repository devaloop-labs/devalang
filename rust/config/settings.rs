use serde::{Deserialize, Serialize};
use std::{any::Any, io::Write};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserSettings {
    pub session: String,
    pub telemetry: TelemetrySettings,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TelemetrySettings {
    pub uuid: String,
    pub stats: bool,
    pub level: String,
    pub enabled: bool,
}

pub fn get_home_dir() -> std::path::PathBuf {
    dirs::home_dir().unwrap_or_default()
}

pub fn get_devalang_homedir() -> std::path::PathBuf {
    get_home_dir().join(".devalang")
}

pub fn get_default_user_config() -> UserSettings {
    UserSettings {
        session: "".into(),
        telemetry: TelemetrySettings {
            uuid: uuid::Uuid::new_v4().to_string(),
            enabled: true,
            level: "basic".into(),
            stats: true,
        },
    }
}

pub fn get_user_config() -> Option<UserSettings> {
    let config_path = get_devalang_homedir().join("config.json");
    if config_path.try_exists().is_ok() {
        let file = std::fs::File::open(config_path).ok()?;
        let settings = serde_json::from_reader(file).ok()?;
        Some(settings)
    } else {
        None
    }
}

pub fn write_user_config_file() {
    let settings = get_user_config().unwrap_or(get_default_user_config());

    let config_path = get_devalang_homedir().join("config.json");
    let mut file = std::fs::File::create(config_path).unwrap();
    let config_json = serde_json::to_string(&settings).unwrap();

    file.write_all(config_json.as_bytes()).unwrap();
}

pub fn set_user_config_bool(key: &str, value: bool) {
    let mut settings = get_user_config().unwrap_or_default();

    match key {
        "telemetry" => {
            settings.telemetry.enabled = value;
        }
        _ => {}
    }

    let config_path = get_devalang_homedir().join("config.json");
    let config_json = serde_json::to_string(&settings).unwrap();
    let mut file = std::fs::File::create(config_path).unwrap();

    file.write_all(config_json.as_bytes()).unwrap();
}

pub fn set_user_config_string(key: &str, value: String) {
    let mut settings = get_user_config().unwrap_or_default();

    match key {
        "session" => {
            settings.session = value;
        }
        _ => {}
    }

    let config_path = get_devalang_homedir().join("config.json");
    let config_json = serde_json::to_string(&settings).unwrap();
    let mut file = std::fs::File::create(config_path).unwrap();

    file.write_all(config_json.as_bytes()).unwrap();
}
