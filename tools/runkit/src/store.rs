use crate::model::{Athlete, Plan, Races, Workout};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Loaded {
    pub workouts: Vec<(PathBuf, Workout)>,
    pub plans: Vec<(PathBuf, Plan)>,
    pub athlete: Option<(PathBuf, Athlete)>,
    pub races: Option<(PathBuf, Races)>,
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("yaml") | Some("yml")
            )
        })
        .collect();
    files.sort();
    files
}

pub fn load_workouts(root: &Path) -> Vec<(PathBuf, Result<Workout>)> {
    yaml_files(&root.join("data/workouts"))
        .into_iter()
        .map(|path| {
            let parsed = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))
                .and_then(|text| {
                    serde_norway::from_str(&text)
                        .with_context(|| format!("failed to parse {}", path.display()))
                });
            (path, parsed)
        })
        .collect()
}

pub fn load_plans(root: &Path) -> Vec<(PathBuf, Result<Plan>)> {
    yaml_files(&root.join("data/plan"))
        .into_iter()
        .map(|path| {
            let parsed = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))
                .and_then(|text| {
                    serde_norway::from_str(&text)
                        .with_context(|| format!("failed to parse {}", path.display()))
                });
            (path, parsed)
        })
        .collect()
}

pub fn load_athlete(root: &Path) -> Result<Option<(PathBuf, Athlete)>> {
    let files = yaml_files(&root.join("data/athlete"));
    let Some(path) = files.into_iter().next() else {
        return Ok(None);
    };
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let athlete: Athlete = serde_norway::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some((path, athlete)))
}

pub fn load_races(root: &Path) -> Result<Option<(PathBuf, Races)>> {
    let files = yaml_files(&root.join("data/races"));
    let Some(path) = files.into_iter().next() else {
        return Ok(None);
    };
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let races: Races = serde_norway::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some((path, races)))
}

pub fn load_all(root: &Path) -> Result<Loaded> {
    let workouts = load_workouts(root)
        .into_iter()
        .filter_map(|(p, r)| match r {
            Ok(w) => Some((p, w)),
            Err(e) => {
                eprintln!("warning: skipping unparsable workout file: {e:#}");
                None
            }
        })
        .collect();
    let plans = load_plans(root)
        .into_iter()
        .filter_map(|(p, r)| match r {
            Ok(w) => Some((p, w)),
            Err(e) => {
                eprintln!("warning: skipping unparsable plan file: {e:#}");
                None
            }
        })
        .collect();
    Ok(Loaded {
        workouts,
        plans,
        athlete: load_athlete(root)?,
        races: load_races(root)?,
    })
}

pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let tmp = path.with_extension("yaml.tmp");
    fs::write(&tmp, content).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to rename {}", path.display()))?;
    Ok(())
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if let Ok(existing) = fs::read_to_string(path) {
        if existing == content {
            return Ok(false);
        }
    }
    atomic_write(path, content)?;
    Ok(true)
}
