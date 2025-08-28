use super::settings::get_devalang_homedir;
use crate::config::driver::ProjectConfig;
use crate::core::{
    parser::statement::{Statement, StatementKind},
    store::global::GlobalStore,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    fs,
    io::Write,
    path::Path,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatsCounts {
    pub nb_files: usize,
    pub nb_modules: usize,
    pub nb_lines: usize,
    pub nb_banks: usize,
    pub nb_plugins: usize,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatsFeatures {
    pub uses_imports: bool,
    pub uses_functions: bool,
    pub uses_groups: bool,
    pub uses_automations: bool,
    pub uses_loops: bool,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatsAudio {
    pub avg_bpm: Option<u32>,
    pub has_synths: bool,
    pub has_samples: bool,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ProjectStats {
    pub counts: StatsCounts,
    pub features: StatsFeatures,
    pub audio: StatsAudio,
}

impl ProjectStats {
    pub fn new() -> Self {
        ProjectStats {
            counts: StatsCounts {
                nb_files: 0,
                nb_modules: 0,
                nb_lines: 0,
                nb_banks: 0,
                nb_plugins: 0,
            },
            features: StatsFeatures {
                uses_imports: false,
                uses_functions: false,
                uses_groups: false,
                uses_automations: false,
                uses_loops: false,
            },
            audio: StatsAudio {
                avg_bpm: None,
                has_synths: false,
                has_samples: false,
            },
        }
    }

    // Returns the in-memory stats if available, otherwise loads from file,
    // otherwise returns a default struct.
    pub fn get() -> Result<Self, String> {
        if let Some(stats) = get_memory_stats() {
            return Ok(stats);
        }
        if let Some(stats) = load_from_file() {
            return Ok(stats);
        }
        Ok(Self::new())
    }

    // Saves stats to stats.json under the devalang home directory.
    pub fn persist(&self) -> Result<(), String> {
        save_to_file(self)
    }
}

// ----------------------------
// Storage helpers (memory/file)
// ----------------------------

static STATS_MEMORY: OnceLock<Mutex<ProjectStats>> = OnceLock::new();

fn stats_file_path() -> PathBuf {
    get_devalang_homedir().join("stats.json")
}

pub fn set_memory_stats(stats: ProjectStats) {
    let m = STATS_MEMORY.get_or_init(|| Mutex::new(ProjectStats::new()));
    if let Ok(mut guard) = m.lock() {
        *guard = stats;
    }
}

pub fn get_memory_stats() -> Option<ProjectStats> {
    let m = STATS_MEMORY.get_or_init(|| Mutex::new(ProjectStats::new()));
    if let Ok(guard) = m.lock() {
        // If it's the default value (all zero/false/None), still return it; caller can decide.
        return Some(guard.clone());
    }
    None
}

pub fn load_from_file() -> Option<ProjectStats> {
    let path = stats_file_path();
    if !path.exists() {
        return None;
    }
    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<ProjectStats>(&content) {
            Ok(stats) => Some(stats),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub fn save_to_file(stats: &ProjectStats) -> Result<(), String> {
    let path = stats_file_path();
    let dir = path
        .parent()
        .unwrap_or(&get_devalang_homedir())
        .to_path_buf();
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create stats dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(stats)
        .map_err(|e| format!("failed to serialize stats: {}", e))?;
    let mut file =
        fs::File::create(&path).map_err(|e| format!("failed to create stats file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("failed to write stats file: {}", e))?;
    Ok(())
}

// Compute stats from modules and store
pub fn compute_from(
    statements_by_module: &HashMap<String, Vec<Statement>>,
    global_store: &GlobalStore,
    config: &Option<ProjectConfig>,
    _output_dir: Option<&str>,
) -> ProjectStats {
    let mut stats = ProjectStats::new();

    // Counts
    stats.counts.nb_modules = statements_by_module.len();
    stats.counts.nb_files = stats.counts.nb_modules; // number of source files loaded

    let mut total_lines = 0usize;
    for (path, _stmts) in statements_by_module.iter() {
        let p = Path::new(path);
        if let Ok(content) = fs::read_to_string(p) {
            total_lines += content.lines().count();
        }
    }
    stats.counts.nb_lines = total_lines;

    // Banks/Plugins from config
    if let Some(cfg) = config {
        stats.counts.nb_banks = cfg.banks.as_ref().map(|v| v.len()).unwrap_or(0);
        stats.counts.nb_plugins = cfg.plugins.as_ref().map(|v| v.len()).unwrap_or(0);
    }

    // Features and audio
    let mut bpm_sum: f32 = 0.0;
    let mut bpm_count: usize = 0;

    fn visit(
        stmts: &[Statement],
        acc: &mut ProjectStats,
        bpm_sum: &mut f32,
        bpm_count: &mut usize,
    ) {
        for s in stmts {
            match &s.kind {
                StatementKind::Import { .. } => acc.features.uses_imports = true,
                StatementKind::Function { body, .. } => {
                    acc.features.uses_functions = true;
                    visit(body, acc, bpm_sum, bpm_count);
                }
                StatementKind::Group => {
                    acc.features.uses_groups = true;
                    if let Some(body) = extract_body_block(&s.value) {
                        visit(body, acc, bpm_sum, bpm_count);
                    }
                }
                StatementKind::Automate { .. } => acc.features.uses_automations = true,
                StatementKind::Loop => {
                    acc.features.uses_loops = true;
                    if let Some(body) = extract_body_block(&s.value) {
                        visit(body, acc, bpm_sum, bpm_count);
                    }
                }
                StatementKind::Synth => acc.audio.has_synths = true,
                StatementKind::Tempo => {
                    if let crate::core::shared::value::Value::Number(bpm) = &s.value {
                        *bpm_sum += *bpm as f32;
                        *bpm_count += 1;
                    }
                }
                StatementKind::Trigger { entity, .. } => {
                    let e = entity.to_lowercase();
                    if e.ends_with(".wav") || e.ends_with(".mp3") || e.contains("devalang://bank/")
                    {
                        acc.audio.has_samples = true;
                    }
                }
                _ => {}
            }
        }
    }

    fn extract_body_block(v: &crate::core::shared::value::Value) -> Option<&[Statement]> {
        use crate::core::shared::value::Value;
        if let Value::Map(map) = v {
            if let Some(Value::Block(stmts)) = map.get("body") {
                return Some(stmts.as_slice());
            }
        }
        None
    }

    for (_path, stmts) in statements_by_module.iter() {
        visit(stmts, &mut stats, &mut bpm_sum, &mut bpm_count);
    }

    if !stats.audio.has_samples {
        for (_k, v) in global_store.variables.variables.iter() {
            if let crate::core::shared::value::Value::String(s) = v {
                if s.starts_with("devalang://bank/") {
                    stats.audio.has_samples = true;
                    break;
                }
            }
        }
    }

    if bpm_count > 0 {
        stats.audio.avg_bpm = Some((bpm_sum / bpm_count as f32).round() as u32);
    }

    stats
}
