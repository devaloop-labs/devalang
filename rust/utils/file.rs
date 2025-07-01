use std::{ fs::{ self }, path::Path };
use include_dir::{ Dir, DirEntry };

pub fn copy_dir_recursive(dir: &Dir, target_root: &Path, base_path: &Path) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(subdir) => {
                copy_dir_recursive(subdir, target_root, base_path);
            }
            DirEntry::File(file) => {
                let rel_path = file.path().strip_prefix(base_path).unwrap();
                let dest_path = target_root.join(rel_path);

                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }

                fs::write(&dest_path, file.contents()).expect("Error writing file");
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
