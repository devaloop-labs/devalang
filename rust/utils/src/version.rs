use crate::{ path::get_package_root, signature::get_signature };

/// Returns the CLI version with a runtime-first strategy:
/// 1. DEVALANG_CLI_VERSION env var
/// 2. project-version.json located next to the running binary
/// 3. package.json located in the package root
/// 4. compile-time env!("CARGO_PKG_VERSION") as a last resort
pub fn get_version() -> String {
    // 1) env override
    if let Ok(v) = std::env::var("DEVALANG_CLI_VERSION") {
        if !v.trim().is_empty() {
            return v;
        }
    }

    // 2) project-version.json next to binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            let pv = bin_dir.join("project-version.json");
            if pv.exists() {
                if let Ok(contents) = std::fs::read_to_string(pv) {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&contents) {
                        if let Some(v) = parsed.get("version").and_then(|s| s.as_str()) {
                            return v.to_string();
                        }
                    }
                }
            }
        }
    }

    // 3) package.json via package root
    if let Some(root) = get_package_root() {
        let pkg = root.join("package.json");
        if pkg.exists() {
            if let Ok(contents) = std::fs::read_to_string(pkg) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&contents) {
                    if let Some(v) = parsed.get("version").and_then(|s| s.as_str()) {
                        return v.to_string();
                    }
                }
            }
        }
    }

    // 4) compile-time fallback
    let compile_time = option_env!("CARGO_PKG_VERSION").unwrap_or("0.0.0");
    compile_time.to_string()
}

pub fn get_version_with_signature() -> String {
    let version = get_version();
    let signature = get_signature(&version);

    println!("{}", signature);

    "(c) 2025 Devaloop. All rights reserved.".to_string()
}
