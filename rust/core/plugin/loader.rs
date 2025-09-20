use devalang_types::{plugin::PluginExport, plugin::PluginInfo as SharedPluginInfo};
use devalang_utils::path as path_utils;
use serde::Deserialize;
use toml::Value as TomlValue;

#[derive(Debug, Deserialize, Clone)]
struct LocalExportEntry {
    pub name: String,
    // Local plugin.toml uses 'kind' to describe export type (e.g. "func", "number")
    pub kind: String,
    #[serde(default)]
    pub default: Option<TomlValue>,
}

#[derive(Debug, Deserialize, Clone)]
struct LocalPluginFile {
    pub plugin: LocalPluginInfo,
    #[serde(rename = "exports", default)]
    pub exports: Vec<LocalExportEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct LocalPluginInfo {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

pub fn load_plugin(author: &str, name: &str) -> Result<(SharedPluginInfo, Vec<u8>), String> {
    let root = path_utils::get_deva_dir()?;
    let plugin_dir_preferred = root.join("plugins").join(format!("{}.{}", author, name));
    let toml_path_preferred = plugin_dir_preferred.join("plugin.toml");
    let wasm_path_preferred_bg = plugin_dir_preferred.join(format!("{}_bg.wasm", name));
    let wasm_path_preferred_plain = plugin_dir_preferred.join(format!("{}.wasm", name));

    // Legacy layout (fallback): ./.deva/plugin/<author>/<name>/
    let plugin_dir_fallback = root.join("plugins").join(author).join(name);
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
    let plugin_file: LocalPluginFile = toml::from_str(&toml_content)
        .map_err(|e| format!("Failed to parse '{}': {}", toml_path.display(), e))?;

    let wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| format!("Failed to read '{}': {}", wasm_path.display(), e))?;

    // Map local parsed plugin info to shared PluginInfo
    let mut exports: Vec<PluginExport> = Vec::new();
    for e in plugin_file.exports.iter() {
        exports.push(PluginExport {
            name: e.name.clone(),
            kind: e.kind.clone(),
            default: e.default.clone(),
        });
    }

    let info = SharedPluginInfo {
        author: plugin_file
            .plugin
            .author
            .unwrap_or_else(|| author.to_string()),
        name: plugin_file.plugin.name.clone(),
        version: plugin_file.plugin.version.clone(),
        description: plugin_file.plugin.description.clone(),
        exports,
    };

    Ok((info, wasm_bytes))
}

/// Load a plugin from dot notation: "author.name"
pub fn load_plugin_from_dot(dot: &str) -> Result<(SharedPluginInfo, Vec<u8>), String> {
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

pub fn load_plugin_from_uri(uri: &str) -> Result<(SharedPluginInfo, Vec<u8>), String> {
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
