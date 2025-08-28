use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::{Cursor, copy};
use std::path::Path;
use zip::ZipArchive;

pub async fn download_file(url: &str, destination: &Path) -> Result<(), Box<dyn Error>> {
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

pub async fn extract_archive(
    zip_path: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(BufReader::new(file))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = destination.join(file.mangled_name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }

            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // Clear the temporary folder after extraction
    if zip_path.exists() {
        std::fs::remove_file(zip_path)?;
    }

    Ok(())
}
