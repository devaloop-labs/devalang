#[cfg(feature = "cli")]
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub author: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub exports: Vec<PluginExport>,
}

#[derive(Debug, Clone)]
pub struct PluginExport {
    pub name: String,
    pub kind: String,
}

#[cfg(feature = "cli")]
pub fn load_plugin(author: &str, name: &str) -> Result<(PluginInfo, Vec<u8>), String> {
    use serde::Deserialize;
    
    #[derive(Debug, Deserialize)]
    struct LocalPluginToml {
        plugin: LocalPluginInfo,
        #[serde(rename = "exports", default)]
        exports: Vec<LocalExportEntry>,
    }
    
    #[derive(Debug, Deserialize)]
    struct LocalPluginInfo {
        name: String,
        #[serde(default)]
        version: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        publisher: Option<String>,
    }
    
    #[derive(Debug, Deserialize)]
    struct LocalExportEntry {
        name: String,
        kind: String,
    }
    
    // Find .deva directory from current dir or parents
    let deva_dir = find_deva_dir()?;
    
    // Try new layout: .deva/plugins/<publisher>/<name>/
    let plugin_dir = deva_dir.join("plugins").join(author).join(name);
    let toml_path = plugin_dir.join("plugin.toml");
    let wasm_path = plugin_dir.join(format!("{}.wasm", name));
    
    if !toml_path.exists() {
        return Err(format!("❌ Plugin file not found: {}", toml_path.display()));
    }
    
    if !wasm_path.exists() {
        return Err(format!("❌ Plugin wasm not found: {}", wasm_path.display()));
    }
    
    // Load toml
    let toml_content = std::fs::read_to_string(&toml_path)
        .map_err(|e| format!("Failed to read '{}': {}", toml_path.display(), e))?;
    let plugin_toml: LocalPluginToml = toml::from_str(&toml_content)
        .map_err(|e| format!("Failed to parse '{}': {}", toml_path.display(), e))?;
    
    // Load wasm bytes
    let wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| format!("Failed to read '{}': {}", wasm_path.display(), e))?;
    
    // Convert to PluginInfo
    let info = PluginInfo {
        author: plugin_toml.plugin.publisher.unwrap_or_else(|| author.to_string()),
        name: plugin_toml.plugin.name.clone(),
        version: plugin_toml.plugin.version.clone(),
        description: plugin_toml.plugin.description.clone(),
        exports: plugin_toml.exports.iter().map(|e| PluginExport {
            name: e.name.clone(),
            kind: e.kind.clone(),
        }).collect(),
    };
    
    Ok((info, wasm_bytes))
}

#[cfg(feature = "cli")]
fn find_deva_dir() -> Result<PathBuf, String> {
    use std::env;
    
    let current_dir = env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Start from current dir and walk up to find .deva
    let mut dir = current_dir.as_path();
    loop {
        let deva_path = dir.join(".deva");
        if deva_path.exists() && deva_path.is_dir() {
            return Ok(deva_path);
        }
        
        // Move to parent directory
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return Err("Could not find .deva directory in current directory or parents".to_string()),
        }
    }
}
