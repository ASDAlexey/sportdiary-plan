use crate::model::{Kind, Totals, Workout, WorkoutStatus};
use crate::store::{atomic_write, load_workouts, write_if_changed};
use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: String,
    pub date: NaiveDate,
    pub sport_type: u16,
    pub distance_km: f64,
    pub moving_time_s: u32,
    pub avg_hr: Option<u16>,
    pub max_hr: Option<u16>,
    pub elevation_m: Option<u32>,
    pub title: Option<String>,
    #[serde(default)]
    pub fitness_test: bool,
}

#[derive(Default, Deserialize, Serialize)]
struct Index {
    workouts: BTreeMap<String, String>,
}

pub fn run(root: &Path, input: &Path) -> Result<()> {
    let records: Vec<Record> = serde_json::from_slice(&std::fs::read(input)?)
        .context("Не удалось прочитать записи Coros")?;
    let index_path = root.join("raw/coros-index.json");
    let mut index: Index = if index_path.exists() {
        serde_json::from_slice(&std::fs::read(&index_path)?)?
    } else {
        Index::default()
    };
    let mut existing = Vec::new();
    for (path, workout) in load_workouts(root) {
        existing.push((path, workout?));
    }
    let mut seen = BTreeSet::new();
    // Check the entire input before touching files.
    for r in &records {
        if r.id.is_empty()
            || !r
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            bail!("Недопустимый идентификатор Coros: {}", r.id);
        }
        if !seen.insert(&r.id) {
            bail!("Повторный идентификатор Coros: {}", r.id);
        }
        if !r.distance_km.is_finite() || r.distance_km < 0.0 {
            bail!("Недопустимая дистанция Coros: {}", r.id);
        }
    }
    let mut prepared = Vec::new();
    let mut claimed = BTreeSet::new();
    let mut skipped = 0;
    for r in records {
        if !(100..=103).contains(&r.sport_type) {
            skipped += 1;
            continue;
        }
        let filename = if let Some(name) = index.workouts.get(&r.id) {
            if !name.ends_with(".yaml")
                || name.contains('/')
                || name.contains('\\')
                || name.contains("..")
            {
                bail!("Некорректный путь в индексе Coros");
            }
            if !root.join("data/workouts").join(name).is_file() {
                bail!("Запись из индекса Coros отсутствует: {name}");
            }
            name.clone()
        } else {
            let matches: Vec<_> = existing
                .iter()
                .filter(|(_, w)| {
                    w.date == r.date
                        && (w.totals.distance_km - r.distance_km).abs() <= 0.015
                        && w.totals.moving_time_s.abs_diff(r.moving_time_s) <= 2
                })
                .collect();
            if matches.len() > 1 {
                bail!("Неоднозначное сопоставление Coros {}", r.id);
            }
            if let Some((path, _)) = matches.first() {
                path.file_name().unwrap().to_string_lossy().into_owned()
            } else {
                let generated = format!("{}-coros-{}.yaml", r.date, r.id);
                if root.join("data/workouts").join(&generated).exists() {
                    bail!("Файл {generated} уже существует без сопоставления Coros");
                }
                generated
            }
        };
        if !claimed.insert(filename.clone()) {
            bail!("Две активности Coros соответствуют {filename}");
        }
        if index
            .workouts
            .iter()
            .any(|(id, name)| id != &r.id && name == &filename)
        {
            bail!("Файл {filename} уже связан с другой активностью Coros");
        }
        let path = root.join("data/workouts").join(&filename);
        let mut workout = if let Some((_, w)) = existing.iter().find(|(p, _)| p == &path) {
            if w.date != r.date {
                bail!("Дата активности Coros {} изменилась", r.id);
            }
            w.clone()
        } else {
            Workout {
                date: r.date,
                title: r
                    .title
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "Беговая тренировка".into()),
                kind: if r.sport_type == 103 || r.fitness_test {
                    Kind::session
                } else {
                    Kind::easy
                },
                source: "coros-mcp".into(),
                status: WorkoutStatus::done,
                plan_ref: None,
                totals: Totals {
                    distance_km: 0.0,
                    moving_time_s: 0,
                    elevation_m: None,
                    avg_hr: None,
                    max_hr: None,
                },
                structure: vec![],
                rpe: None,
                hr_zones_s: None,
                notes: if r.sport_type == 103 || r.fitness_test {
                    String::new()
                } else {
                    "Тип тренировки не подтверждён; спокойный бег указан предварительно.".into()
                },
                shoes: None,
                injuries: vec![],
            }
        };
        workout.source = "coros-mcp".into();
        workout.totals.distance_km = r.distance_km;
        workout.totals.moving_time_s = r.moving_time_s;
        if r.avg_hr.is_some() {
            workout.totals.avg_hr = r.avg_hr;
        }
        if r.max_hr.is_some() {
            workout.totals.max_hr = r.max_hr;
        }
        if r.elevation_m.is_some() {
            workout.totals.elevation_m = r.elevation_m;
        }
        if r.fitness_test && !workout.notes.contains("fitness-test") {
            if !workout.notes.is_empty() {
                workout.notes.push(' ');
            }
            workout.notes.push_str("fitness-test: тест уровня Coros.");
        }
        index.workouts.insert(r.id, filename);
        prepared.push((path, serde_norway::to_string(&workout)?));
    }
    let mut changed = 0;
    for (path, yaml) in &prepared {
        if write_if_changed(path, yaml)? {
            changed += 1;
        }
    }
    let index_text = format!("{}\n", serde_json::to_string_pretty(&index)?);
    if std::fs::read_to_string(&index_path).ok().as_deref() != Some(&index_text) {
        atomic_write(&index_path, &index_text)?;
    }
    println!(
        "Coros: обработано {}, изменено {}, пропущено небеговых {}",
        prepared.len(),
        changed,
        skipped
    );
    Ok(())
}
