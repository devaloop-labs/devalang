use crate::{ path::get_package_root, signature::get_signature };

pub fn get_version() -> String {
    if let Some(root) = get_package_root() {
        let project_version_json = root.join("package.json");
        if let Ok(version) = std::fs::read_to_string(project_version_json) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&version) {
                if let Some(version_val) = parsed.get("version") {
                    if let Some(s) = version_val.as_str() {
                        return s.to_string();
                    }
                }
            }
        }
    }

    "0.0.0".to_string()
}

pub fn get_version_with_signature() -> String {
    let version = get_version();
    let signature = get_signature(&version);

    println!("{}", signature);

    "(c) 2025 Devaloop. All rights reserved.".to_string()
}
