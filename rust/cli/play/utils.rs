use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn snapshot_files<P: AsRef<Path>>(dir: P) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(duration) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        map.insert(entry.path().display().to_string(), duration.as_secs());
                    }
                }
            }
        }
    }
    map
}

pub fn files_changed(old: &HashMap<String, u64>, new: &HashMap<String, u64>) -> bool {
    old != new
}
