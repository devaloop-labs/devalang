use devalang_types::AddonMetadata;
use toml::Value;

pub fn parse_metadata_file(addon_type: &str, metadata_content: &str) -> Option<AddonMetadata> {
    let parsed = metadata_content.parse::<Value>().ok()?;

    let table = (match addon_type {
        "bank" => parsed.get("bank"),
        "plugin" => parsed.get("plugin"),
        "preset" => parsed.get("preset"),
        "template" => parsed.get("template"),
        _ => None,
    })?;

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let version = table
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = table
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let author = table
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let access = table
        .get("access")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(AddonMetadata {
        name,
        author,
        version,
        description,
        access,
    })
}
