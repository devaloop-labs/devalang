use crate::utils::signature::get_signature;

pub fn get_version() -> &'static str {
    let version: &str = env!("CARGO_PKG_VERSION");
    version
}

pub fn get_version_with_signature() -> &'static str {
    let version = get_version();
    let signature = get_signature(version);

    println!("{}", signature);

    "(c) 2025 Devaloop. All rights reserved."
}
