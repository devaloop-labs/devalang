use devalang_types::{TelemetrySettings, UserSettings};
use serde_json::Value as JsonValue;
use std::io::Write;

pub fn get_home_dir() -> Option<std::path::PathBuf> {
    dirs::home_dir()
}

pub fn get_devalang_homedir() -> std::path::PathBuf {
    if let Some(home_dir) = get_home_dir() {
        home_dir.join(".devalang")
    } else {
        std::path::PathBuf::from("~/.devalang")
    }
}

pub fn get_default_user_config() -> UserSettings {
    UserSettings {
        session: "".into(),
        telemetry: TelemetrySettings {
            uuid: uuid::Uuid::new_v4().to_string(),
            enabled: false,
            level: "basic".into(),
            stats: false,
        },
    }
}

pub fn get_user_config() -> Option<UserSettings> {
    if let Some(config_path) = get_devalang_homedir().join("config.json").into() {
        let file = std::fs::File::open(config_path).ok()?;
        let settings = serde_json::from_reader(file).ok()?;
        Some(settings)
    } else {
        None
    }
}

pub fn write_user_config_file() {
    if let Some(config_path) = get_devalang_homedir().join("config.json").into() {
        let settings = get_user_config().unwrap_or_else(get_default_user_config);

        let config_json = serde_json::to_string(&settings).unwrap();

        if let Err(e) = write_config_atomic(&config_path, &config_json) {
            println!("Could not write config file: {}", e);
        }
    } else {
        println!("Could not create config file");
    }
}

pub fn ensure_user_config_file_exists() {
    if let Some(config_path) = get_devalang_homedir().join("config.json").into() {
        if !config_path.exists() {
            write_user_config_file();
        }
    }
}

pub fn set_user_config_value(key: &str, value: JsonValue) {
    let mut settings = get_user_config().unwrap_or_default();

    match (key, &value) {
        ("telemetry", JsonValue::Bool(b)) => {
            settings.telemetry.enabled = *b;
        }
        ("session", JsonValue::String(s)) => {
            settings.session = s.clone();
        }
        _ => {
            println!("Unsupported key or value type for '{}': {:?}", key, value);
        }
    }

    if let Some(config_path) = get_devalang_homedir().join("config.json").into() {
        let config_json = serde_json::to_string(&settings).unwrap();
        if let Err(e) = write_config_atomic(&config_path, &config_json) {
            println!("Could not write config file: {}", e);
        }
    } else {
        println!("Could not create config file");
    }
}

pub fn write_config_atomic(
    config_path: &std::path::PathBuf,
    contents: &str,
) -> std::io::Result<()> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_path = config_path.with_extension("json.tmp");
    let mut tmp_file = std::fs::File::create(&tmp_path)?;
    tmp_file.write_all(contents.as_bytes())?;
    tmp_file.sync_all()?;
    std::fs::rename(&tmp_path, config_path)?;

    Ok(())
}
