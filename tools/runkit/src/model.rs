use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[allow(non_camel_case_types)]
pub enum Kind {
    easy,
    long,
    session,
    hills,
    race,
    strength,
    rest,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::easy => "easy",
            Kind::long => "long",
            Kind::session => "session",
            Kind::hills => "hills",
            Kind::race => "race",
            Kind::strength => "strength",
            Kind::rest => "rest",
        }
    }

    pub fn is_quality(&self) -> bool {
        matches!(self, Kind::session | Kind::hills | Kind::race)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum WorkoutStatus {
    done,
    planned,
    skipped,
    moved,
}

impl WorkoutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkoutStatus::done => "done",
            WorkoutStatus::planned => "planned",
            WorkoutStatus::skipped => "skipped",
            WorkoutStatus::moved => "moved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum PlanItemStatus {
    planned,
    done,
    partial,
    moved,
    skipped,
}

impl PlanItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PlanItemStatus::planned => "planned",
            PlanItemStatus::done => "done",
            PlanItemStatus::partial => "partial",
            PlanItemStatus::moved => "moved",
            PlanItemStatus::skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[allow(non_camel_case_types)]
pub enum Phase {
    base,
    build,
    peak,
    sharpen,
    race,
    deload,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::base => "base",
            Phase::build => "build",
            Phase::peak => "peak",
            Phase::sharpen => "sharpen",
            Phase::race => "race",
            Phase::deload => "deload",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Totals {
    pub distance_km: f64,
    pub moving_time_s: u32,
    pub elevation_m: Option<u32>,
    pub avg_hr: Option<u16>,
    pub max_hr: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance_km: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_s: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace_s_per_km: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rest: Option<String>,
}

impl Segment {
    pub fn distance_km(&self) -> f64 {
        self.distance_km.unwrap_or(0.0) * self.repeat.unwrap_or(1) as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workout {
    pub date: NaiveDate,
    pub title: String,
    pub kind: Kind,
    pub source: String,
    pub status: WorkoutStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_ref: Option<String>,
    pub totals: Totals,
    #[serde(default)]
    pub structure: Vec<Segment>,
    #[serde(default)]
    pub rpe: Option<u8>,
    #[serde(default)]
    pub hr_zones_s: Option<Vec<u32>>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub shoes: Option<String>,
    #[serde(default)]
    pub injuries: Vec<String>,
}

impl Workout {
    pub fn avg_pace_s_per_km(&self) -> Option<f64> {
        if self.totals.distance_km > 0.0 {
            Some(self.totals.moving_time_s as f64 / self.totals.distance_km)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: String,
    pub date: NaiveDate,
    pub kind: Kind,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub structure: Vec<Segment>,
    pub status: PlanItemStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moved_to: Option<String>,
    #[serde(default)]
    pub notes: String,
}

impl PlanItem {
    pub fn planned_distance_km(&self) -> f64 {
        self.structure.iter().map(Segment::distance_km).sum()
    }

    pub fn has_strides(&self) -> bool {
        mentions_strides(&self.title) || self.structure.iter().any(mentions_strides_seg)
    }
}

fn mentions_strides_seg(s: &Segment) -> bool {
    mentions_strides(&s.name) || s.rest.as_deref().is_some_and(mentions_strides)
}

pub fn mentions_strides(text: &str) -> bool {
    let t = text.to_lowercase();
    t.contains("strid") || t.contains("страйд") || t.contains("ускорен")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub week: String,
    pub monday: NaiveDate,
    pub phase: Phase,
    pub items: Vec<PlanItem>,
    #[serde(default)]
    pub notes: String,
}

impl Plan {
    pub fn item(&self, id: &str) -> Option<&PlanItem> {
        self.items.iter().find(|i| i.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklySlot {
    pub slot: String,
    #[serde(default)]
    pub controlled: bool,
}

fn default_growth() -> f64 {
    1.10
}

fn default_quality_days() -> u32 {
    3
}

fn default_strides_days() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardrails {
    #[serde(default = "default_growth")]
    pub max_weekly_volume_growth: f64,
    #[serde(default = "default_quality_days")]
    pub max_controlled_quality_days: u32,
    #[serde(default = "default_strides_days")]
    pub max_strides_days: u32,
}

impl Default for Guardrails {
    fn default() -> Self {
        Guardrails {
            max_weekly_volume_growth: default_growth(),
            max_controlled_quality_days: default_quality_days(),
            max_strides_days: default_strides_days(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Injury {
    pub region: String,
    pub status: String,
    #[serde(default)]
    pub note: String,
}

impl Injury {
    pub fn is_active(&self) -> bool {
        self.status.starts_with("active")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Athlete {
    pub name: String,
    #[serde(default)]
    pub weekly_template: BTreeMap<String, WeeklySlot>,
    #[serde(default)]
    pub guardrails: Guardrails,
    #[serde(default)]
    pub injury_history: Vec<Injury>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Race {
    pub date: NaiveDate,
    pub name: String,
    pub kind: String,
    pub priority: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceResult {
    pub date: NaiveDate,
    pub name: String,
    pub kind: String,
    pub time_s: u32,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Races {
    #[serde(default)]
    pub races: Vec<Race>,
    #[serde(default)]
    pub results: Vec<RaceResult>,
}
