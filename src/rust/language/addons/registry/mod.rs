#![cfg(feature = "cli")]

use std::collections::{HashMap, HashSet, hash_map::Entry};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct BankDefinition {
    identifier: String,
    publisher: Option<String>,
    name: String,
    root_dir: PathBuf,
    audio_root: PathBuf,
    triggers: HashMap<String, PathBuf>,
}

impl BankDefinition {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn publisher(&self) -> Option<&str> {
        self.publisher.as_deref()
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn audio_root(&self) -> &Path {
        &self.audio_root
    }

    pub fn trigger_count(&self) -> usize {
        self.triggers.len()
    }

    pub fn resolve_trigger(&self, trigger: &str) -> Option<PathBuf> {
        self.triggers.get(trigger).cloned()
    }

    fn load(identifier: &str, project_root: &Path, base_dir: &Path) -> Result<Self> {
        let manifest_path = locate_manifest(identifier, project_root, base_dir)
            .with_context(|| format!("unable to locate bank manifest for '{}'", identifier))?;

        let manifest_dir = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| project_root.to_path_buf());

        let raw = fs::read_to_string(&manifest_path)
            .with_context(|| format!("failed to read {}", manifest_path.display()))?;
        let manifest: BankManifest = toml::from_str(&raw)
            .with_context(|| format!("invalid bank manifest at {}", manifest_path.display()))?;

        let mut publisher = manifest
            .bank
            .as_ref()
            .and_then(|section| section.publisher.clone());
        let mut name = manifest
            .bank
            .as_ref()
            .and_then(|section| section.name.clone())
            .or_else(|| {
                manifest_dir
                    .file_name()
                    .map(|v| v.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| {
                identifier
                    .rsplit_once('.')
                    .map(|(_, bank)| bank.to_string())
                    .unwrap_or_else(|| identifier.to_string())
            });

        if publisher.is_none() {
            if let Some((pubr, _)) = identifier.rsplit_once('.') {
                publisher = Some(pubr.to_string());
            }
        }

        if name.is_empty() {
            name = identifier.to_string();
        }

        let audio_root = manifest
            .bank
            .as_ref()
            .and_then(|section| section.audio_path.clone())
            .map(|raw| resolve_audio_root(&manifest_dir, &raw))
            .unwrap_or_else(|| manifest_dir.join("audio"));

        let audio_root = normalize_path(&audio_root);

        let mut triggers = HashMap::new();
        for entry in manifest.triggers.into_iter() {
            let trigger_path = resolve_trigger_path(&audio_root, &manifest_dir, &entry.path);
            triggers.insert(entry.name, trigger_path);
        }

        if triggers.is_empty() {
            return Err(anyhow!(
                "bank manifest {} does not define any triggers",
                manifest_path.display()
            ));
        }

        Ok(Self {
            identifier: identifier.to_string(),
            publisher,
            name,
            root_dir: manifest_dir,
            audio_root,
            triggers,
        })
    }
}

#[derive(Default)]
pub struct BankRegistry {
    banks: HashMap<String, BankDefinition>,
}

impl BankRegistry {
    pub fn new() -> Self {
        Self {
            banks: HashMap::new(),
        }
    }

    pub fn register_bank(
        &mut self,
        alias: impl Into<String>,
        identifier: &str,
        project_root: &Path,
        base_dir: &Path,
    ) -> Result<&mut BankDefinition> {
        let alias = alias.into();
        let result = match self.banks.entry(alias.clone()) {
            Entry::Occupied(mut entry) => {
                if entry.get().identifier == identifier {
                    entry.into_mut()
                } else {
                    let bank = BankDefinition::load(identifier, project_root, base_dir)?;
                    *entry.get_mut() = bank;
                    entry.into_mut()
                }
            }
            Entry::Vacant(entry) => {
                let bank = BankDefinition::load(identifier, project_root, base_dir)?;
                entry.insert(bank)
            }
        };
        Ok(result)
    }

    pub fn resolve_trigger(&self, alias: &str, trigger: &str) -> Option<PathBuf> {
        self.banks
            .get(alias)
            .and_then(|bank| bank.resolve_trigger(trigger))
    }

    pub fn has_bank(&self, alias: &str) -> bool {
        self.banks.contains_key(alias)
    }
}

#[derive(Debug, Deserialize)]
struct BankManifest {
    #[serde(default)]
    bank: Option<BankSection>,
    #[serde(default)]
    triggers: Vec<TriggerEntry>,
}

#[derive(Debug, Deserialize)]
struct BankSection {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    publisher: Option<String>,
    #[serde(default, alias = "audio_path", alias = "audioPath")]
    audio_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TriggerEntry {
    name: String,
    path: String,
}

fn locate_manifest(identifier: &str, project_root: &Path, base_dir: &Path) -> Result<PathBuf> {
    let candidates = candidate_directories(identifier, project_root, base_dir);
    for candidate in candidates {
        let manifest_path = if candidate.is_file() {
            if let Some(name) = candidate.file_name() {
                if name == "bank.toml" {
                    candidate.clone()
                } else {
                    continue;
                }
            } else {
                continue;
            }
        } else {
            candidate.join("bank.toml")
        };

        if manifest_path.is_file() {
            return Ok(manifest_path);
        }
    }

    Err(anyhow!(
        "bank '{}' not found (searched relative to {} and project root {})",
        identifier,
        base_dir.display(),
        project_root.display()
    ))
}

fn candidate_directories(identifier: &str, project_root: &Path, base_dir: &Path) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    let normalized = identifier.replace('\\', "/");
    let identifier_path = Path::new(&normalized);

    let push_candidate = |path: PathBuf, set: &mut HashSet<String>, vec: &mut Vec<PathBuf>| {
        let key = path.to_string_lossy().to_string();
        if set.insert(key) {
            vec.push(path);
        }
    };

    if identifier_path.is_absolute() {
        push_candidate(identifier_path.to_path_buf(), &mut seen, &mut candidates);
    } else {
        push_candidate(base_dir.join(identifier_path), &mut seen, &mut candidates);
        push_candidate(
            project_root.join(identifier_path),
            &mut seen,
            &mut candidates,
        );

        if normalized.starts_with("./") || normalized.starts_with("../") {
            let joined = base_dir.join(identifier_path);
            push_candidate(normalize_path(&joined), &mut seen, &mut candidates);
        }

        if let Some((publisher, bank)) = normalized.rsplit_once('.') {
            let banks_root = project_root.join(".deva").join("banks");

            let publisher_path = publisher.replace('.', "/");
            let nested_pub: PathBuf = publisher
                .split('.')
                .fold(banks_root.clone(), |acc, part| acc.join(part));

            push_candidate(nested_pub.join(bank), &mut seen, &mut candidates);
            push_candidate(
                banks_root.join(format!("{}.{}", publisher, bank)),
                &mut seen,
                &mut candidates,
            );
            push_candidate(
                banks_root.join(&publisher).join(bank),
                &mut seen,
                &mut candidates,
            );
            push_candidate(
                banks_root.join(&publisher_path).join(bank),
                &mut seen,
                &mut candidates,
            );
        } else {
            let banks_root = project_root.join(".deva").join("banks");
            push_candidate(banks_root.join(&normalized), &mut seen, &mut candidates);
        }
    }

    candidates
}

fn resolve_audio_root(manifest_dir: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    if path.is_absolute() {
        normalize_path(path)
    } else {
        let joined = manifest_dir.join(path);
        normalize_path(&joined)
    }
}

fn resolve_trigger_path(audio_root: &Path, manifest_dir: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    if path.is_absolute() {
        normalize_path(path)
    } else if raw.starts_with("./") {
        let trimmed = raw.trim_start_matches("./");

        if !trimmed.is_empty() {
            let candidate = audio_root.join(trimmed);
            if candidate.is_file() {
                return normalize_path(&candidate);
            }
        }

        let joined = manifest_dir.join(path);
        normalize_path(&joined)
    } else if raw.starts_with("../") {
        let joined = manifest_dir.join(path);
        normalize_path(&joined)
    } else {
        let joined = audio_root.join(path);
        if joined.is_file() {
            normalize_path(&joined)
        } else {
            normalize_path(&manifest_dir.join(path))
        }
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
