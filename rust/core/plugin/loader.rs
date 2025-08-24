use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginFile {
    pub plugin: PluginInfo,
}

pub fn load_plugin(name: &str) -> Result<(PluginInfo, Vec<u8>), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let plugin_dir = root.join(".deva").join("plugin").join(name);
    let toml_path = plugin_dir.join("plugin.toml");
    let wasm_path = plugin_dir.join(format!("{}_bg.wasm", name));

    if !toml_path.exists() {
        return Err(format!("❌ Plugin file not found: {}", toml_path.display()));
    }
    if !wasm_path.exists() {
        return Err(format!("❌ Plugin wasm not found: {}", wasm_path.display()));
    }

    let toml_content = std::fs::read_to_string(&toml_path)
        .map_err(|e| format!("Failed to read '{}': {}", toml_path.display(), e))?;
    let plugin_file: PluginFile = toml::from_str(&toml_content)
        .map_err(|e| format!("Failed to parse '{}': {}", toml_path.display(), e))?;

    let wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| format!("Failed to read '{}': {}", wasm_path.display(), e))?;

    Ok((plugin_file.plugin, wasm_bytes))
}

pub fn load_plugin_from_uri(uri: &str) -> Result<(PluginInfo, Vec<u8>), String> {
    if !uri.starts_with("devalang://plugin/") {
        return Err("Invalid plugin URI".into());
    }

    let name = uri.trim_start_matches("devalang://plugin/");
    load_plugin(name)
}