use include_dir::{ Dir, DirEntry };
use std::{ fs, path::Path, io::{ Read, Seek, SeekFrom } };

pub fn copy_dir_recursive(dir: &Dir, target_root: &Path, base_path: &Path) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(subdir) => {
                copy_dir_recursive(subdir, target_root, base_path);
            }
            DirEntry::File(file) => {
                // Compute the destination path relative to the provided base.
                let rel_path = match file.path().strip_prefix(base_path) {
                    Ok(p) => p.to_owned(),
                    Err(_) => {
                        eprintln!(
                            "Warning: failed to compute relative path for {:?}, skipping",
                            file.path()
                        );
                        continue;
                    }
                };

                let dest_path = target_root.join(rel_path);

                if let Some(parent) = dest_path.parent() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!(
                            "Warning: failed to create directory {}: {}",
                            parent.display(),
                            e
                        );
                        continue;
                    }
                }

                if let Err(e) = fs::write(&dest_path, file.contents()) {
                    eprintln!("Warning: failed to write {}: {}", dest_path.display(), e);
                    continue;
                }
            }
        }
    }
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;

    if bytes >= MB {
        format!("{:.2} Mb", (bytes as f64) / (MB as f64))
    } else if bytes >= KB {
        format!("{:.2} Kb", (bytes as f64) / (KB as f64))
    } else {
        format!("{} bytes", bytes)
    }
}

pub fn extract_zip_safely(archive_path: &Path, dest: &Path) -> Result<(), String> {
    // Open file and peek magic bytes to determine archive format
    let mut file = std::fs::File
        ::open(archive_path)
        .map_err(|e| format!("Failed to open archive {}: {}", archive_path.display(), e))?;

    let mut magic = [0u8; 4];
    let n = file
        .read(&mut magic)
        .map_err(|e| format!("Failed to read archive header {}: {}", archive_path.display(), e))?;
    file
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek archive {}: {}", archive_path.display(), e))?;

    let is_gzip = n >= 2 && magic[0] == 0x1f && magic[1] == 0x8b;
    let is_zip = n >= 2 && magic[0] == 0x50 && magic[1] == 0x4b;

    if is_gzip {
        // The gzip may contain a tar archive OR a gzipped zip (PK inside).
        // Decompress to a temporary file, inspect its magic bytes, then choose extractor.
        let mut gz = flate2::read::GzDecoder::new(file);

        let mut named = tempfile::NamedTempFile
            ::new()
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        std::io
            ::copy(&mut gz, &mut named)
            .map_err(|e| format!("Failed to decompress gzip to temp: {}", e))?;
        named
            .as_file_mut()
            .seek(SeekFrom::Start(0))
            .map_err(|e| format!("Failed to seek temp file: {}", e))?;

        // Read magic of decompressed content
        let mut head = [0u8; 4];
        let n = named
            .as_file_mut()
            .read(&mut head)
            .map_err(|e| format!("Failed to read temp archive header: {}", e))?;
        named
            .as_file_mut()
            .seek(SeekFrom::Start(0))
            .map_err(|e| format!("Failed to rewind temp file: {}", e))?;

        let inner_is_zip = n >= 2 && head[0] == 0x50 && head[1] == 0x4b;

        if inner_is_zip {
            let tmp_file = named
                .reopen()
                .map_err(|e| format!("Failed to reopen temp file for zip: {}", e))?;

            let mut archive = zip::ZipArchive
                ::new(tmp_file)
                .map_err(|e| format!("Failed to read zip archive inside gzip: {}", e))?;

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to access zip archive entry {}: {}", i, e))?;
                let enclosed = match file.enclosed_name() {
                    Some(p) => p.to_owned(),
                    None => {
                        continue;
                    }
                };
                if
                    enclosed.is_absolute() ||
                    enclosed.components().any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    continue;
                }
                let outpath = dest.join(enclosed);
                if file.name().ends_with('/') || file.is_dir() {
                    std::fs
                        ::create_dir_all(&outpath)
                        .map_err(|e| format!("Failed to create dir {}: {}", outpath.display(), e))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        std::fs
                            ::create_dir_all(p)
                            .map_err(|e|
                                format!("Failed to create parent {}: {}", p.display(), e)
                            )?;
                    }
                    let mut outfile = std::fs::File
                        ::create(&outpath)
                        .map_err(|e|
                            format!("Failed to create file {}: {}", outpath.display(), e)
                        )?;
                    std::io
                        ::copy(&mut file, &mut outfile)
                        .map_err(|e| format!("Failed to write file {}: {}", outpath.display(), e))?;
                }
            }

            // NamedTempFile will be removed on drop
            return Ok(());
        }

        // otherwise treat as tar
        let tmp_file = named
            .reopen()
            .map_err(|e| format!("Failed to reopen temp file for tar: {}", e))?;
        let mut archive = tar::Archive::new(tmp_file);

        for entry in archive
            .entries()
            .map_err(|e| format!("Failed to read tar entries (from gzip): {}", e))? {
            let mut entry = entry.map_err(|e|
                format!("Failed to read tar archive entry (from gzip): {}", e)
            )?;
            let path = match entry.path() {
                Ok(p) => p.into_owned(),
                Err(_) => {
                    continue;
                }
            };
            if
                path.is_absolute() ||
                path.components().any(|c| matches!(c, std::path::Component::ParentDir))
            {
                continue;
            }
            let outpath = dest.join(&path);
            if let Some(parent) = outpath.parent() {
                std::fs
                    ::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent {}: {}", parent.display(), e))?;
            }
            entry
                .unpack(&outpath)
                .map_err(|e|
                    format!("Failed to unpack tar entry to {}: {}", outpath.display(), e)
                )?;
        }

        return Ok(());
    }

    if is_zip {
        // Re-open file for zip
        let file = std::fs::File
            ::open(archive_path)
            .map_err(|e|
                format!("Failed to open archive for zip {}: {}", archive_path.display(), e)
            )?;

        let mut archive = zip::ZipArchive
            ::new(file)
            .map_err(|e| format!("Failed to read zip archive {}: {}", archive_path.display(), e))?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to access zip archive entry {}: {}", i, e))?;

            let enclosed = match file.enclosed_name() {
                Some(p) => p.to_owned(),
                None => {
                    continue;
                }
            };

            if
                enclosed.is_absolute() ||
                enclosed.components().any(|c| matches!(c, std::path::Component::ParentDir))
            {
                continue;
            }

            let outpath = dest.join(enclosed);
            if file.name().ends_with('/') || file.is_dir() {
                std::fs
                    ::create_dir_all(&outpath)
                    .map_err(|e| format!("Failed to create dir {}: {}", outpath.display(), e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs
                        ::create_dir_all(p)
                        .map_err(|e| format!("Failed to create parent {}: {}", p.display(), e))?;
                }
                let mut outfile = std::fs::File
                    ::create(&outpath)
                    .map_err(|e| format!("Failed to create file {}: {}", outpath.display(), e))?;
                std::io
                    ::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to write file {}: {}", outpath.display(), e))?;
            }
        }

        return Ok(());
    }

    // Unknown magic: try tar.gz first, then zip as fallback
    // Try tar.gz
    let file = std::fs::File
        ::open(archive_path)
        .map_err(|e| format!("Failed to open archive {}: {}", archive_path.display(), e))?;
    if
        let Ok(()) = (|| -> Result<(), String> {
            let gz = flate2::read::GzDecoder::new(&file);
            let mut archive = tar::Archive::new(gz);
            for entry in archive
                .entries()
                .map_err(|e| format!("Failed to read tar entries: {}", e))? {
                let mut entry = entry.map_err(|e|
                    format!("Failed to read tar archive entry: {}", e)
                )?;
                let path = match entry.path() {
                    Ok(p) => p.into_owned(),
                    Err(_) => {
                        continue;
                    }
                };
                if
                    path.is_absolute() ||
                    path.components().any(|c| matches!(c, std::path::Component::ParentDir))
                {
                    continue;
                }
                let outpath = dest.join(&path);
                if let Some(parent) = outpath.parent() {
                    std::fs
                        ::create_dir_all(parent)
                        .map_err(|e|
                            format!("Failed to create parent {}: {}", parent.display(), e)
                        )?;
                }
                entry
                    .unpack(&outpath)
                    .map_err(|e|
                        format!("Failed to unpack tar entry to {}: {}", outpath.display(), e)
                    )?;
            }
            Ok(())
        })()
    {
        return Ok(());
    }

    // Fallback to zip
    let file = std::fs::File
        ::open(archive_path)
        .map_err(|e| format!("Failed to open archive for zip {}: {}", archive_path.display(), e))?;
    let mut archive = zip::ZipArchive
        ::new(file)
        .map_err(|e| format!("Failed to read zip archive {}: {}", archive_path.display(), e))?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access zip archive entry {}: {}", i, e))?;
        let enclosed = match file.enclosed_name() {
            Some(p) => p.to_owned(),
            None => {
                continue;
            }
        };
        if
            enclosed.is_absolute() ||
            enclosed.components().any(|c| matches!(c, std::path::Component::ParentDir))
        {
            continue;
        }
        let outpath = dest.join(enclosed);
        if file.name().ends_with('/') || file.is_dir() {
            std::fs
                ::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create dir {}: {}", outpath.display(), e))?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs
                    ::create_dir_all(p)
                    .map_err(|e| format!("Failed to create parent {}: {}", p.display(), e))?;
            }
            let mut outfile = std::fs::File
                ::create(&outpath)
                .map_err(|e| format!("Failed to create file {}: {}", outpath.display(), e))?;
            std::io
                ::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file {}: {}", outpath.display(), e))?;
        }
    }

    Ok(())
}

/// Detects addon type by inspecting archive contents without extracting fully.
/// Returns one of: "bank", "plugin", "preset", "template", or "unknown".
pub fn detect_addon_type_in_archive(archive_path: &Path) -> Result<String, String> {
    use std::fs::File;

    let mut file = File::open(archive_path)
        .map_err(|e| format!("Failed to open archive {}: {}", archive_path.display(), e))?;

    let mut magic = [0u8; 4];
    let n = file
        .read(&mut magic)
        .map_err(|e| format!("Failed to read archive header {}: {}", archive_path.display(), e))?;
    file
        .seek(SeekFrom::Start(0))
        .map_err(|e| format!("Failed to seek archive {}: {}", archive_path.display(), e))?;

    let is_gzip = n >= 2 && magic[0] == 0x1f && magic[1] == 0x8b;
    let is_zip = n >= 2 && magic[0] == 0x50 && magic[1] == 0x4b;

    let check_name = |name: &str| {
        let lname = name.to_ascii_lowercase();
        if lname.ends_with("bank.toml") {
            return Some("bank".to_string());
        }
        if lname.ends_with("plugin.toml") {
            return Some("plugin".to_string());
        }
        if lname.ends_with("preset.toml") {
            return Some("preset".to_string());
        }
        if lname.ends_with("template.toml") {
            return Some("template".to_string());
        }
        None
    };

    if is_gzip {
        // Decompress to temp and inspect inner archive
        let mut gz = flate2::read::GzDecoder::new(file);
        let mut named = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        std::io::copy(&mut gz, &mut named)
            .map_err(|e| format!("Failed to decompress gzip to temp: {}", e))?;
        named.as_file_mut().seek(SeekFrom::Start(0)).map_err(|e| format!("Failed to seek temp file: {}", e))?;

        // Inspect inner magic
        let mut head = [0u8; 4];
        let m = named.as_file_mut().read(&mut head).map_err(|e| format!("Failed to read temp archive header: {}", e))?;
        named.as_file_mut().seek(SeekFrom::Start(0)).map_err(|e| format!("Failed to rewind temp file: {}", e))?;

        let inner_is_zip = m >= 2 && head[0] == 0x50 && head[1] == 0x4b;
        if inner_is_zip {
            let tmp_file = named.reopen().map_err(|e| format!("Failed to reopen temp file for zip: {}", e))?;
            let mut archive = zip::ZipArchive::new(tmp_file).map_err(|e| format!("Failed to read zip archive inside gzip: {}", e))?;
            for i in 0..archive.len() {
                if let Ok(file) = archive.by_index(i) {
                    if let Some(enclosed) = file.enclosed_name() {
                        let s = enclosed.to_string_lossy().to_string();
                        if let Some(t) = check_name(&s) {
                            return Ok(t);
                        }
                    }
                }
            }
            return Ok("unknown".to_string());
        }

        // treat as tar
        let tmp_file = named.reopen().map_err(|e| format!("Failed to reopen temp file for tar: {}", e))?;
        let mut archive = tar::Archive::new(tmp_file);
        if let Ok(entries) = archive.entries() {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(path) = entry.path() {
                    let s = path.to_string_lossy().to_string();
                    if let Some(t) = check_name(&s) {
                        return Ok(t);
                    }
                }
            }
        }
        return Ok("unknown".to_string());
    }

    if is_zip {
        let file = std::fs::File::open(archive_path).map_err(|e| format!("Failed to open archive for zip {}: {}", archive_path.display(), e))?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip archive {}: {}", archive_path.display(), e))?;
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                if let Some(enclosed) = file.enclosed_name() {
                    let s = enclosed.to_string_lossy().to_string();
                    if let Some(t) = check_name(&s) {
                        return Ok(t);
                    }
                }
            }
        }
        return Ok("unknown".to_string());
    }

    // fallback: try tar.gz read attempt
    if let Ok(f) = std::fs::File::open(archive_path) {
        let gz = flate2::read::GzDecoder::new(f);
        let mut archive = tar::Archive::new(gz);
        if let Ok(entries) = archive.entries() {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(path) = entry.path() {
                    let s = path.to_string_lossy().to_string();
                    if let Some(t) = check_name(&s) {
                        return Ok(t);
                    }
                }
            }
        }
    }

    Ok("unknown".to_string())
}

/// Removes the temporary folder located at `.deva/tmp` inside the project root if it exists.
/// Returns Ok(()) even if the folder did not exist. Returns Err(String) on filesystem errors.
pub fn clear_tmp_folder() -> Result<(), String> {
    match crate::path::ensure_deva_dir() {
        Ok(deva_dir) => {
            let tmp_dir = deva_dir.join("tmp");
            if tmp_dir.exists() {
                std::fs::remove_dir_all(&tmp_dir)
                    .map_err(|e| format!("Failed to remove tmp dir '{}': {}", tmp_dir.display(), e))?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}
