use crate::dates::{local_date_of_utc, parse_iso_date};
use crate::model::{Kind, Totals, Workout, WorkoutStatus};
use crate::slug::slugify;
use crate::store::atomic_write;
use crate::track::{compute_metrics, parse_gpx, parse_json, parse_tcx};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{NaiveDate, Timelike};
use std::path::{Path, PathBuf};

pub struct IngestArgs {
    pub path: PathBuf,
    pub title: Option<String>,
    pub title_from_time: bool,
    pub kind: Kind,
    pub source: Option<String>,
    pub date: Option<NaiveDate>,
    pub rpe: Option<u8>,
    pub notes: Option<String>,
    pub plan_ref: Option<String>,
}

pub fn run(root: &Path, args: &IngestArgs) -> Result<Workout> {
    let bytes = std::fs::read(&args.path)
        .with_context(|| format!("failed to read {}", args.path.display()))?;
    let ext = args
        .path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let (activity, elevation_hint, date_hint) = match ext.as_str() {
        "gpx" => (parse_gpx(&bytes)?, None, None),
        "tcx" => (parse_tcx(&bytes)?, None, None),
        "json" => {
            let (activity, summary) = parse_json(&bytes)?;
            (activity, summary.elevation_hint, summary.date_hint)
        }
        other => bail!("unsupported file extension {other:?}; expected .gpx, .tcx or .json"),
    };
    let metrics = compute_metrics(&activity, elevation_hint);

    let date = match args.date {
        Some(d) => d,
        None => date_hint
            .as_deref()
            .and_then(|d| parse_iso_date(d).ok())
            .or_else(|| metrics.start_utc.map(local_date_of_utc))
            .ok_or_else(|| anyhow!("no date in file; pass --date YYYY-MM-DD explicitly"))?,
    };

    let title = match (&args.title, args.title_from_time) {
        (Some(t), _) => t.clone(),
        (None, true) => title_from_start(metrics.start_utc)?,
        (None, false) => bail!("--title is required unless --title-from-time is set"),
    };

    let source = args.source.clone().unwrap_or_else(|| ext.clone());

    let workout = Workout {
        date,
        title: title.clone(),
        kind: args.kind,
        source,
        status: WorkoutStatus::done,
        plan_ref: args.plan_ref.clone(),
        totals: Totals {
            distance_km: metrics.distance_km,
            moving_time_s: metrics.moving_time_s,
            elevation_m: metrics.elevation_m,
            avg_hr: metrics.avg_hr,
            max_hr: metrics.max_hr,
        },
        structure: Vec::new(),
        rpe: args.rpe,
        hr_zones_s: None,
        notes: args.notes.clone().unwrap_or_default(),
        shoes: None,
        injuries: Vec::new(),
    };

    let mut yaml = serde_norway::to_string(&workout).context("failed to serialize workout")?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    let filename = format!("{}-{}.yaml", date, slugify(&title));
    let target = root.join("data/workouts").join(filename);
    atomic_write(&target, &yaml)?;

    let pace = if metrics.distance_km > 0.0 {
        crate::dates::format_pace(metrics.moving_time_s as f64 / metrics.distance_km)
    } else {
        "—".to_string()
    };
    println!(
        "wrote {} ({:.2} km, {}, {})",
        target.display(),
        metrics.distance_km,
        crate::dates::format_hms(metrics.moving_time_s),
        pace
    );
    Ok(workout)
}

fn title_from_start(start_utc: Option<i64>) -> Result<String> {
    let epoch = start_utc.ok_or_else(|| {
        anyhow!("--title-from-time needs a start timestamp; pass --title instead")
    })?;
    let hour = chrono::DateTime::from_timestamp(epoch, 0)
        .map(|dt| dt.with_timezone(&chrono::Local).hour())
        .ok_or_else(|| anyhow!("invalid start timestamp"))?;
    Ok(match hour {
        5..=11 => "Morning run",
        12..=16 => "Afternoon run",
        17..=22 => "Evening run",
        _ => "Night run",
    }
    .to_string())
}
