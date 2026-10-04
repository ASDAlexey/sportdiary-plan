use crate::dates::{iso_week_id, DAY_IDS};
use crate::model::{Athlete, Guardrails, Plan, PlanItemStatus, Workout};
use crate::store::{load_athlete, load_plans, load_races, load_workouts};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, Weekday};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct Validation {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Validation {
    fn push(&mut self, is_error: bool, msg: String) {
        if is_error {
            self.errors.push(msg);
        } else {
            self.warnings.push(msg);
        }
    }
}

fn rel_path(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| file.display().to_string())
}

fn day_id_prefix(id: &str) -> Option<&'static str> {
    DAY_IDS
        .iter()
        .find(|d| id.starts_with(*d))
        .copied()
        .filter(|d| {
            let rest = &id[d.len()..];
            rest.is_empty() || rest.chars().all(|c| c.is_ascii_digit())
        })
}

pub fn run(root: &Path) -> Result<Validation> {
    let mut v = Validation {
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let workouts = load_workouts(root);
    let plans_loaded = load_plans(root);
    let plans: Vec<&Plan> = plans_loaded
        .iter()
        .filter_map(|(_, r)| r.as_ref().ok())
        .collect();
    let plan_index: BTreeMap<&str, &Plan> = plans.iter().map(|p| (p.week.as_str(), *p)).collect();
    let athlete: Option<Athlete> = match load_athlete(root) {
        Ok(opt) => opt.map(|(_, a)| a),
        Err(e) => {
            v.push(true, format!("data/athlete: {e:#}"));
            None
        }
    };
    let guardrails = athlete
        .as_ref()
        .map(|a| a.guardrails.clone())
        .unwrap_or_default();
    let races = match load_races(root) {
        Ok(opt) => opt.map(|(_, r)| r),
        Err(e) => {
            v.push(true, format!("data/races: {e:#}"));
            None
        }
    };

    for (path, result) in &workouts {
        let rel = rel_path(root, path);
        let workout = match result {
            Ok(w) => w,
            Err(e) => {
                v.push(true, format!("{rel}: {e:#}"));
                continue;
            }
        };
        validate_workout(&mut v, &rel, workout, &plan_index);
    }

    for (path, result) in &plans_loaded {
        let rel = rel_path(root, path);
        let plan = match result {
            Ok(p) => p,
            Err(e) => {
                v.push(true, format!("{rel}: {e:#}"));
                continue;
            }
        };
        validate_plan(&mut v, &rel, plan);
    }

    if let Some(athlete) = &athlete {
        if athlete.name.trim().is_empty() {
            let rel = root.join("data/athlete").display().to_string();
            v.push(true, format!("{rel}: name must not be empty"));
        }
        let missing_days: Vec<&str> = DAY_IDS
            .iter()
            .filter(|d| !athlete.weekly_template.contains_key(**d))
            .copied()
            .collect();
        if !missing_days.is_empty() {
            v.push(
                false,
                format!(
                    "athlete weekly_template is missing days: {}",
                    missing_days.join(", ")
                ),
            );
        }
    }

    if let Some(races) = &races {
        if races.races.is_empty() && races.results.is_empty() {
            v.push(false, "data/races: no races and no results".to_string());
        }
        for (i, race) in races.races.iter().enumerate() {
            if race.name.trim().is_empty() || race.kind.trim().is_empty() {
                v.push(
                    true,
                    format!("data/races: races[{i}] must have non-empty name and kind"),
                );
            }
        }
        for (i, res) in races.results.iter().enumerate() {
            if res.time_s == 0 {
                v.push(
                    true,
                    format!("data/races: results[{i}].time_s must be positive"),
                );
            }
        }
    }

    let ok_workouts: Vec<(&Path, &Workout)> = workouts
        .iter()
        .filter_map(|(p, r)| r.as_ref().ok().map(|w| (p.as_path(), w)))
        .collect();
    check_volume_growth(&mut v, &ok_workouts, &guardrails);
    if let Some(athlete) = &athlete {
        for plan in &plans {
            check_controlled_quality(&mut v, plan, athlete, &guardrails);
        }
    }
    check_strides(&mut v, &ok_workouts, &plans, &guardrails);

    Ok(v)
}

fn validate_workout(
    v: &mut Validation,
    rel: &str,
    workout: &Workout,
    plan_index: &BTreeMap<&str, &Plan>,
) {
    if workout.title.trim().is_empty() {
        v.push(true, format!("{rel}: title must not be empty"));
    }
    if workout.source.trim().is_empty() {
        v.push(true, format!("{rel}: source must not be empty"));
    }
    if workout.totals.distance_km < 0.0 {
        v.push(
            true,
            format!("{rel}: totals.distance_km must not be negative"),
        );
    }
    if let Some(rpe) = workout.rpe {
        if !(1..=10).contains(&rpe) {
            v.push(true, format!("{rel}: rpe must be within 1..=10, got {rpe}"));
        }
    }
    if let Some(zones) = &workout.hr_zones_s {
        if zones.len() != 6 {
            v.push(
                true,
                format!(
                    "{rel}: hr_zones_s must have 6 entries (Coros zones), got {}",
                    zones.len()
                ),
            );
        }
    }
    if let Some(plan_ref) = &workout.plan_ref {
        let mut parts = plan_ref.splitn(2, '/');
        let week = parts.next().unwrap_or("");
        let item_id = parts.next().unwrap_or("");
        match plan_index.get(week) {
            None => v.push(
                true,
                format!("{rel}: plan_ref {plan_ref:?} points to unknown plan week {week:?}"),
            ),
            Some(plan) => {
                if plan.item(item_id).is_none() {
                    v.push(
                        true,
                        format!("{rel}: plan_ref {plan_ref:?} points to unknown item {item_id:?}"),
                    );
                }
            }
        }
    }
    let expected_prefix = format!("{}-", workout.date.format("%Y-%m-%d"));
    let file_name = Path::new(rel)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    if !file_name.starts_with(&expected_prefix) {
        v.push(
            true,
            format!(
                "{rel}: file name must start with {expected_prefix} to match date {}",
                workout.date.format("%Y-%m-%d")
            ),
        );
    }
}

fn validate_plan(v: &mut Validation, rel: &str, plan: &Plan) {
    if plan.monday.weekday() != Weekday::Mon {
        v.push(
            true,
            format!(
                "{rel}: monday {} is not a Monday",
                plan.monday.format("%Y-%m-%d")
            ),
        );
    }
    let computed = iso_week_id(plan.monday);
    if plan.week != computed {
        v.push(
            true,
            format!(
                "{rel}: week {:?} does not match monday {} ({computed})",
                plan.week,
                plan.monday.format("%Y-%m-%d")
            ),
        );
    }
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    for item in &plan.items {
        if !seen_ids.insert(item.id.as_str()) {
            v.push(true, format!("{rel}: duplicate item id {:?}", item.id));
        }
        if day_id_prefix(&item.id).is_none() {
            v.push(
                true,
                format!(
                    "{rel}: item id {:?} is not a day id like mon, tue, wed2",
                    item.id
                ),
            );
        }
        let offset = (item.date - plan.monday).num_days();
        if !(0..=6).contains(&offset) {
            v.push(
                true,
                format!(
                    "{rel}: item {:?} date {} is outside the week starting {}",
                    item.id,
                    item.date.format("%Y-%m-%d"),
                    plan.monday.format("%Y-%m-%d")
                ),
            );
        }
        match item.status {
            PlanItemStatus::moved => {
                if item.moved_to.is_none() {
                    v.push(
                        true,
                        format!("{rel}: item {:?} has status moved but no moved_to", item.id),
                    );
                }
            }
            _ => {
                if item.moved_to.is_some() {
                    v.push(
                        true,
                        format!(
                            "{rel}: item {:?} has moved_to but status is {}",
                            item.id,
                            item.status.as_str()
                        ),
                    );
                }
            }
        }
        for seg in &item.structure {
            if let Some(repeat) = seg.repeat {
                if repeat == 0 {
                    v.push(
                        true,
                        format!(
                            "{rel}: item {:?} segment {:?} has repeat 0",
                            item.id, seg.name
                        ),
                    );
                }
            }
        }
    }
}

fn check_volume_growth(
    v: &mut Validation,
    workouts: &[(&Path, &Workout)],
    guardrails: &Guardrails,
) {
    let mut by_week: BTreeMap<String, f64> = BTreeMap::new();
    for (_, w) in workouts {
        *by_week.entry(iso_week_id(w.date)).or_default() += w.totals.distance_km;
    }
    let week_ids: Vec<&String> = by_week.keys().collect();
    for (i, week) in week_ids.iter().enumerate() {
        let volume = by_week[*week];
        if i < 3 || volume <= 0.0 {
            continue;
        }
        let history: Vec<f64> = week_ids[..i]
            .iter()
            .rev()
            .take(4)
            .map(|w| by_week[*w])
            .filter(|km| *km > 0.0)
            .collect();
        if history.len() < 3 {
            continue;
        }
        let mean: f64 = history.iter().sum::<f64>() / history.len() as f64;
        if mean > 0.0 && volume > guardrails.max_weekly_volume_growth * mean {
            v.push(
                false,
                format!(
                    "volume guardrail: week {week} is {:.1} km, above {:.2} x trailing mean ({:.1} km over {} weeks)",
                    volume,
                    guardrails.max_weekly_volume_growth,
                    mean,
                    history.len()
                ),
            );
        }
    }
}

fn check_controlled_quality(
    v: &mut Validation,
    plan: &Plan,
    athlete: &Athlete,
    guardrails: &Guardrails,
) {
    let mut days: Vec<String> = Vec::new();
    for item in &plan.items {
        let Some(prefix) = day_id_prefix(&item.id) else {
            continue;
        };
        let controlled = athlete
            .weekly_template
            .get(prefix)
            .map(|s| s.controlled)
            .unwrap_or(false);
        if controlled && item.kind.is_quality() && !days.contains(&item.id) {
            days.push(item.id.clone());
        }
    }
    if days.len() as u32 > guardrails.max_controlled_quality_days {
        v.push(
            false,
            format!(
                "quality guardrail: plan {} has {} controlled quality days ({}), limit is {}",
                plan.week,
                days.len(),
                days.join(", "),
                guardrails.max_controlled_quality_days
            ),
        );
    }
}

fn check_strides(
    v: &mut Validation,
    workouts: &[(&Path, &Workout)],
    plans: &[&Plan],
    guardrails: &Guardrails,
) {
    let mut by_week: BTreeMap<String, BTreeSet<NaiveDate>> = BTreeMap::new();
    for (_, w) in workouts {
        if crate::model::mentions_strides(&w.title)
            || w.structure
                .iter()
                .any(|s| crate::model::mentions_strides(&s.name))
        {
            by_week
                .entry(iso_week_id(w.date))
                .or_default()
                .insert(w.date);
        }
    }
    for plan in plans {
        for item in &plan.items {
            if item.has_strides() {
                by_week
                    .entry(plan.week.clone())
                    .or_default()
                    .insert(item.date);
            }
        }
    }
    for (week, days) in by_week {
        if days.len() as u32 > guardrails.max_strides_days {
            v.push(
                false,
                format!(
                    "strides guardrail: week {week} has strides on {} days ({}), limit is {}",
                    days.len(),
                    days.iter()
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    guardrails.max_strides_days
                ),
            );
        }
    }
}

pub fn print(v: &Validation) {
    for e in &v.errors {
        println!("ERROR   {e}");
    }
    for w in &v.warnings {
        println!("WARNING {w}");
    }
    println!("{} errors, {} warnings", v.errors.len(), v.warnings.len());
}
