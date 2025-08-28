use serde::Deserialize;
use std::path::Path;
use toml::Value as TomlValue;

#[derive(Debug, Deserialize, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(skip)]
    pub exports: Vec<ExportEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExportEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub default: Option<TomlValue>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginFile {
    pub plugin: PluginInfo,
    #[serde(default)]
    pub export: Vec<ExportEntry>,
}

/// Load a plugin from local .deva directory given author and name
pub fn load_plugin(author: &str, name: &str) -> Result<(PluginInfo, Vec<u8>), String> {
    // Align with other loaders (banks) that use relative ./.deva paths
    let root = Path::new("./.deva");
    // Preferred layout: ./.deva/plugin/<author>.<name>/
    let plugin_dir_preferred = root.join("plugin").join(format!("{}.{}", author, name));
    let toml_path_preferred = plugin_dir_preferred.join("plugin.toml");
    let wasm_path_preferred_bg = plugin_dir_preferred.join(format!("{}_bg.wasm", name));
    let wasm_path_preferred_plain = plugin_dir_preferred.join(format!("{}.wasm", name));

    // Legacy layout (fallback): ./.deva/plugin/<author>/<name>/
    let plugin_dir_fallback = root.join("plugin").join(author).join(name);
    let toml_path_fallback = plugin_dir_fallback.join("plugin.toml");
    let wasm_path_fallback_bg = plugin_dir_fallback.join(format!("{}_bg.wasm", name));
    let wasm_path_fallback_plain = plugin_dir_fallback.join(format!("{}.wasm", name));

    // Resolve actual paths to use
    let (toml_path, wasm_path) = if toml_path_preferred.exists() && wasm_path_preferred_bg.exists()
    {
        (toml_path_preferred, wasm_path_preferred_bg)
    } else if toml_path_preferred.exists() && wasm_path_preferred_plain.exists() {
        (toml_path_preferred, wasm_path_preferred_plain)
    } else if toml_path_fallback.exists() && wasm_path_fallback_bg.exists() {
        (toml_path_fallback, wasm_path_fallback_bg)
    } else if toml_path_fallback.exists() && wasm_path_fallback_plain.exists() {
        (toml_path_fallback, wasm_path_fallback_plain)
    } else {
        // If either file is missing in both layouts, produce specific errors for missing files in preferred layout
        if !toml_path_preferred.exists() {
            return Err(format!(
                "❌ Plugin file not found: {}",
                toml_path_preferred.display()
            ));
        }
        if !wasm_path_preferred_bg.exists() && !wasm_path_preferred_plain.exists() {
            return Err(format!(
                "❌ Plugin wasm not found: '{}' or '{}'",
                wasm_path_preferred_bg.display(),
                wasm_path_preferred_plain.display()
            ));
        }
        unreachable!();
    };

    let toml_content = std::fs::read_to_string(&toml_path)
        .map_err(|e| format!("Failed to read '{}': {}", toml_path.display(), e))?;
    let plugin_file: PluginFile = toml::from_str(&toml_content)
        .map_err(|e| format!("Failed to parse '{}': {}", toml_path.display(), e))?;

    let wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| format!("Failed to read '{}': {}", wasm_path.display(), e))?;

    let mut info = plugin_file.plugin.clone();
    info.exports = plugin_file.export.clone();

    Ok((info, wasm_bytes))
}

/// Load a plugin from dot notation: "author.name"
pub fn load_plugin_from_dot(dot: &str) -> Result<(PluginInfo, Vec<u8>), String> {
    let mut parts = dot.split('.');
    let author = parts
        .next()
        .ok_or_else(|| "Invalid plugin name, missing author".to_string())?;
    let name = parts
        .next()
        .ok_or_else(|| "Invalid plugin name, missing name".to_string())?;
    if parts.next().is_some() {
        return Err("Invalid plugin name format, expected <author>.<name>".into());
    }
    load_plugin(author, name)
}

pub fn load_plugin_from_uri(uri: &str) -> Result<(PluginInfo, Vec<u8>), String> {
    if !uri.starts_with("devalang://plugin/") {
        return Err("Invalid plugin URI".into());
    }

    // Expect format: devalang://plugin/author.name
    let payload = uri.trim_start_matches("devalang://plugin/");
    let mut parts = payload.split('.');
    let author = parts
        .next()
        .ok_or_else(|| "Invalid plugin URI, missing author".to_string())?;
    let name = parts
        .next()
        .ok_or_else(|| "Invalid plugin URI, missing name".to_string())?;
    if parts.next().is_some() {
        return Err("Invalid plugin URI format, expected devalang://plugin/<author>.<name>".into());
    }

    load_plugin(author, name)
}
