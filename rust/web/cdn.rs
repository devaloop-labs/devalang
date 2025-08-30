use std::error::Error;
use std::fs::File;
use std::io::{Cursor, copy};
use std::path::Path;

pub fn get_cdn_url() -> String {
    let cdn_url = "https://cdn.devalang.com";
    // let cdn_url = "http://127.0.0.1:8888";
    cdn_url.to_string()
}

pub async fn download_from_cdn(url: &str, destination: &Path) -> Result<(), Box<dyn Error>> {
    if !url.starts_with(&get_cdn_url()) {
        return Err(format!("Invalid CDN URL: {}", url).into());
    }

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: HTTP {}", response.status()).into());
    }

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let bytes = response.bytes().await?;
    let mut content = Cursor::new(bytes);
    let mut file = File::create(destination)?;

    copy(&mut content, &mut file)?;

    Ok(())
}
