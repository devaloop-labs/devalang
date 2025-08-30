use include_dir::{Dir, DirEntry};
use std::{fs, path::Path};

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
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("Failed to open archive {}: {}", archive_path.display(), e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read archive {}: {}", archive_path.display(), e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to access archive entry {}: {}", i, e))?;

        let enclosed = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => {
                continue;
            }
        };

        let outpath = dest.join(enclosed);

        if file.name().ends_with('/') || file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create dir {}: {}", outpath.display(), e))?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)
                    .map_err(|e| format!("Failed to create parent {}: {}", p.display(), e))?;
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to create file {}: {}", outpath.display(), e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write file {}: {}", outpath.display(), e))?;
        }
    }

    Ok(())
}
