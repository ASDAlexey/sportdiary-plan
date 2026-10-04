use crate::dates::{iso_week_id, DAY_IDS};
use crate::model::{Kind, Phase, Plan, PlanItem, PlanItemStatus};
use crate::store::{atomic_write, load_athlete};
use anyhow::{bail, Context, Result};
use chrono::{Datelike, Duration, NaiveDate};
use std::path::Path;

fn slot_kind(slot: &str) -> Kind {
    if slot.contains("hill") {
        Kind::hills
    } else if slot.contains("quality") {
        Kind::session
    } else if slot.contains("rest") {
        Kind::rest
    } else if slot.contains("long") {
        Kind::long
    } else {
        Kind::easy
    }
}

pub fn run(root: &Path, monday: NaiveDate, phase: Phase, force: bool) -> Result<()> {
    if monday.weekday() != chrono::Weekday::Mon {
        bail!(
            "{} is a {}; --monday must be a Monday",
            monday.format("%Y-%m-%d"),
            monday.weekday()
        );
    }
    let athlete = load_athlete(root)
        .context("failed to load athlete profile")?
        .map(|(_, a)| a)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "athlete profile not found under {}/data/athlete; the weekly_template is required to scaffold a plan",
                root.display()
            )
        })?;

    let week = iso_week_id(monday);
    let target = root.join("data/plan").join(format!("{week}.yaml"));
    if target.exists() && !force {
        bail!(
            "{} already exists; use --force to overwrite",
            target.display()
        );
    }

    let items = DAY_IDS
        .iter()
        .enumerate()
        .map(|(i, day)| {
            let slot = athlete
                .weekly_template
                .get(*day)
                .map(|s| s.slot.clone())
                .unwrap_or_else(|| "easy".to_string());
            PlanItem {
                id: day.to_string(),
                date: monday + Duration::days(i as i64),
                kind: slot_kind(&slot),
                title: String::new(),
                structure: Vec::new(),
                status: PlanItemStatus::planned,
                moved_to: None,
                notes: format!("slot: {slot}"),
            }
        })
        .collect();

    let plan = Plan {
        week: week.clone(),
        monday,
        phase,
        items,
        notes: String::new(),
    };
    let mut yaml = serde_norway::to_string(&plan).context("failed to serialize plan")?;
    if !yaml.ends_with('\n') {
        yaml.push('\n');
    }
    atomic_write(&target, &yaml)?;
    println!("wrote {} ({} items)", target.display(), DAY_IDS.len());
    Ok(())
}
