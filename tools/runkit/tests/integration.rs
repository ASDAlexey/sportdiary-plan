use chrono::NaiveDate;
use runkit::dates::parse_iso_date;
use runkit::ingest::{self, IngestArgs};
use runkit::model::Kind;
use runkit::report;
use runkit::scaffold;
use runkit::track::{compute_metrics, parse_gpx, parse_json, parse_tcx};
use runkit::validate;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn d(s: &str) -> NaiveDate {
    parse_iso_date(s).unwrap()
}

fn write_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn ingest_args(path: &Path, title: &str) -> IngestArgs {
    IngestArgs {
        path: path.to_path_buf(),
        title: Some(title.to_string()),
        title_from_time: false,
        kind: Kind::easy,
        source: None,
        date: Some(d("2026-10-05")),
        rpe: None,
        notes: None,
        plan_ref: None,
    }
}

const ATHLETE_YAML: &str = r#"name: Alexey
weekly_template:
  mon: {slot: quality, controlled: true}
  tue: {slot: secondary, controlled: true}
  wed: {slot: easy-am, controlled: true}
  thu: {slot: quality-group, controlled: false}
  fri: {slot: rest, controlled: true}
  sat: {slot: hills-group, controlled: false}
  sun: {slot: long, controlled: true}
guardrails:
  max_weekly_volume_growth: 1.10
  max_controlled_quality_days: 3
  max_strides_days: 2
injury_history:
  - {region: heel, status: active-caution, note: "tenses up on fast sprint work"}
shoes:
  - {id: lining-x9, brand: Li-Ning, model: TBD, status: current}
"#;

fn workout_yaml(date: &str, title: &str, kind: &str, km: f64, time_s: u32) -> String {
    format!(
        r#"date: {date}
title: "{title}"
kind: {kind}
source: gpx
status: done
totals:
  distance_km: {km}
  moving_time_s: {time_s}
  elevation_m: null
  avg_hr: null
  max_hr: null
structure: []
rpe: null
hr_zones_s: null
notes: ""
shoes: null
injuries: []
"#
    )
}

#[test]
fn parses_gpx_fixture_with_metrics() {
    let bytes = fs::read(fixture("sample.gpx")).unwrap();
    let activity = parse_gpx(&bytes).unwrap();
    assert_eq!(activity.points.len(), 4);
    let m = compute_metrics(&activity, None);
    assert!(
        (m.distance_km - 0.334).abs() < 0.005,
        "got {}",
        m.distance_km
    );
    assert_eq!(m.moving_time_s, 180);
    assert_eq!(m.elevation_m, Some(5));
    let expected_start = chrono::DateTime::parse_from_rfc3339("2026-10-05T06:00:00Z")
        .unwrap()
        .timestamp();
    assert_eq!(m.start_utc, Some(expected_start));
}

#[test]
fn parses_tcx_fixture_with_hr_and_summary() {
    let bytes = fs::read(fixture("sample.tcx")).unwrap();
    let activity = parse_tcx(&bytes).unwrap();
    assert_eq!(activity.points.len(), 3);
    let m = compute_metrics(&activity, None);
    assert!(
        (m.distance_km - 0.222).abs() < 0.005,
        "got {}",
        m.distance_km
    );
    assert_eq!(m.moving_time_s, 80);
    assert_eq!(m.elevation_m, Some(1));
    assert_eq!(m.avg_hr, Some(150));
    assert_eq!(m.max_hr, Some(160));
}

#[test]
fn parses_tcx_summary_without_track() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<TrainingCenterDatabase xmlns="http://www.garmin.com/xmlschemas/TrainingCenterDatabase/v2">
  <Activities>
    <Activity Sport="Running">
      <Id>2026-10-05T06:12:00Z</Id>
      <Lap StartTime="2026-10-05T06:12:00Z">
        <TotalTimeSeconds>2284</TotalTimeSeconds>
        <DistanceMeters>6010</DistanceMeters>
        <AverageHeartRateBpm><Value>138</Value></AverageHeartRateBpm>
        <MaximumHeartRateBpm><Value>151</Value></MaximumHeartRateBpm>
      </Lap>
    </Activity>
  </Activities>
</TrainingCenterDatabase>"#;
    let activity = parse_tcx(xml.as_bytes()).unwrap();
    let m = compute_metrics(&activity, None);
    assert!((m.distance_km - 6.01).abs() < 1e-9);
    assert_eq!(m.moving_time_s, 2284);
    assert_eq!(m.avg_hr, Some(138));
    assert_eq!(m.max_hr, Some(151));
    assert_eq!(m.elevation_m, None);
}

#[test]
fn parses_json_summary_fixture() {
    let bytes = fs::read(fixture("sample.json")).unwrap();
    let (activity, summary) = parse_json(&bytes).unwrap();
    assert_eq!(summary.date_hint.as_deref(), Some("2026-10-05"));
    let m = compute_metrics(&activity, summary.elevation_hint);
    assert!((m.distance_km - 8.02).abs() < 1e-9);
    assert_eq!(m.moving_time_s, 2700);
    assert_eq!(m.elevation_m, Some(32));
    assert_eq!(m.avg_hr, Some(145));
}

#[test]
fn parses_json_track_points() {
    let json = r#"{
      "distance_km": 8.0,
      "moving_time_s": 2400,
      "points": [
        {"lat": 55.0, "lon": 37.0, "ele": 10.0, "time": "2026-10-05T06:00:00Z", "hr": 140},
        {"lat": 55.001, "lon": 37.0, "ele": 12.0, "time": "2026-10-05T06:01:00Z", "hr": 150}
      ]
    }"#;
    let (activity, summary) = parse_json(json.as_bytes()).unwrap();
    let m = compute_metrics(&activity, summary.elevation_hint);
    assert!(
        (m.distance_km - 0.111).abs() < 0.005,
        "got {}",
        m.distance_km
    );
    assert_eq!(m.moving_time_s, 60);
    assert_eq!(m.elevation_m, Some(2));
    assert_eq!(m.avg_hr, Some(145));
    assert_eq!(m.max_hr, Some(150));
}

#[test]
fn ingest_writes_metrics_only_yaml() {
    let root = TempDir::new().unwrap();
    let mut args = ingest_args(&fixture("sample.gpx"), "Morning hills");
    args.kind = Kind::hills;
    args.rpe = Some(5);
    args.notes = Some("soft flat loop".to_string());
    let workout = ingest::run(root.path(), &args).unwrap();

    assert_eq!(workout.date, d("2026-10-05"));
    assert_eq!(workout.kind, Kind::hills);
    let target = root
        .path()
        .join("data/workouts/2026-10-05-morning-hills.yaml");
    let yaml = fs::read_to_string(&target).unwrap();
    assert!(yaml.contains("kind: hills"));
    assert!(yaml.contains("distance_km: 0.33"));
    assert!(yaml.contains("moving_time_s: 180"));
    assert!(yaml.contains("elevation_m: 5"));
    assert!(yaml.contains("rpe: 5"));
    assert!(yaml.contains("notes: soft flat loop"));
    assert!(
        !yaml.contains("55.00"),
        "coordinates must not be stored: {yaml}"
    );
    assert!(
        !yaml.contains("37.0"),
        "coordinates must not be stored: {yaml}"
    );
    assert!(
        !yaml.contains("2026-10-05T"),
        "point timestamps must not be stored: {yaml}"
    );
}

#[test]
fn ingest_replaces_same_workout_atomically() {
    let root = TempDir::new().unwrap();
    let args = ingest_args(&fixture("sample.gpx"), "Morning hills");
    ingest::run(root.path(), &args).unwrap();
    let args2 = ingest_args(&fixture("sample.tcx"), "Morning hills");
    ingest::run(root.path(), &args2).unwrap();
    let dir: Vec<_> = fs::read_dir(root.path().join("data/workouts"))
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(dir, vec!["2026-10-05-morning-hills.yaml".to_string()]);
    let yaml = fs::read_to_string(
        root.path()
            .join("data/workouts/2026-10-05-morning-hills.yaml"),
    )
    .unwrap();
    assert!(yaml.contains("moving_time_s: 80"));
}

#[test]
fn ingest_requires_title_or_title_from_time() {
    let root = TempDir::new().unwrap();
    let mut args = ingest_args(&fixture("sample.gpx"), "");
    args.title = None;
    let err = ingest::run(root.path(), &args).unwrap_err().to_string();
    assert!(err.contains("--title"), "got: {err}");
}

#[test]
fn report_renders_dashboard_and_week_files() {
    let root = TempDir::new().unwrap();
    write_file(root.path(), "data/athlete/alexey.yaml", ATHLETE_YAML);
    write_file(
        root.path(),
        "data/races/races.yaml",
        r#"races:
  - {date: 2026-10-25, name: "October park race", kind: park-5k, priority: benchmark}
results: []
"#,
    );
    write_file(
        root.path(),
        "data/plan/2026-W40.yaml",
        r#"week: 2026-W40
monday: 2026-09-28
phase: base
items:
  - id: mon
    date: 2026-09-28
    kind: session
    title: "6x3min steady"
    structure:
      - {name: warmup, distance_km: 2.0}
      - {name: "3min", repeat: 6, distance_km: 1.0}
      - {name: cooldown, distance_km: 1.5}
    status: done
    notes: ""
  - id: tue
    date: 2026-09-29
    kind: easy
    title: "Easy shakeout"
    structure:
      - {name: run, distance_km: 8.0}
    status: planned
    notes: ""
  - id: sun
    date: 2026-10-04
    kind: long
    title: ""
    structure: []
    status: planned
    notes: ""
notes: "first structured week"
"#,
    );
    let mut w1 = workout_yaml("2026-09-21", "Long slow run", "long", 10.0, 3600);
    w1 = w1.replace("notes: \"\"", "notes: \"week 39 volume\"");
    write_file(
        root.path(),
        "data/workouts/2026-09-21-long-slow-run.yaml",
        &w1,
    );
    let mut w2 = workout_yaml("2026-09-28", "6x3min steady", "session", 9.5, 3000);
    w2 = w2.replace("source: gpx", "source: gpx\nplan_ref: 2026-W40/mon");
    write_file(
        root.path(),
        "data/workouts/2026-09-28-6x3min-steady.yaml",
        &w2,
    );
    write_file(
        root.path(),
        "data/workouts/2026-10-01-easy-recovery.yaml",
        &workout_yaml("2026-10-01", "Easy recovery", "easy", 6.5, 2400),
    );
    let mut w4 = workout_yaml("2026-10-03", "Easy return run", "easy", 6.01, 2284);
    w4 = w4.replace("injuries: []", "injuries: [heel-tightness]");
    write_file(
        root.path(),
        "data/workouts/2026-10-03-easy-return-run.yaml",
        &w4,
    );

    let today = d("2026-10-04");
    report::run(root.path(), today).unwrap();

    let dashboard = fs::read_to_string(root.path().join("reports/DASHBOARD.md")).unwrap();
    assert!(
        dashboard.contains("Текущая неделя 2026-W40"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("План: 17.5 км, выполнено: 22.01 км"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("Следующий старт"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("через 21 день"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("October park race"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("heel-tightness"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("С начала года: 32.01 км"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("Последние тренировки"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("За предыдущие 4 недели: 10.0 км"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("6x3min steady"),
        "dashboard: {dashboard}"
    );

    let week = fs::read_to_string(root.path().join("reports/week-2026-W40.md")).unwrap();
    assert!(
        week.contains("# Неделя 2026-W40 (28.09–04.10)"),
        "week: {week}"
    );
    assert!(week.contains("Фаза: base"), "week: {week}");
    assert!(
        week.contains("| Пн 28.09 | 6x3min steady | тренировка | 9.5 | ✓ | 9.5 |"),
        "week: {week}"
    );
    assert!(
        week.contains("| Вт 29.09 | Easy shakeout | кросс | 8.0 | — |"),
        "week: {week}"
    );
    assert!(
        week.contains("| Чт 01.10 | Easy recovery | кросс | — | ✓ | 6.5 |"),
        "week: {week}"
    );
    assert!(week.contains("Итоги недели"), "week: {week}");
    assert!(week.contains("По видам"), "week: {week}");
    assert!(week.contains("Заметки"), "week: {week}");
    assert!(week.contains("План: first structured week"), "week: {week}");
    assert!(week.contains("- Дистанция: 22.01 км"), "week: {week}");

    assert!(root.path().join("reports/week-2026-W39.md").exists());
    let week39 = fs::read_to_string(root.path().join("reports/week-2026-W39.md")).unwrap();
    assert!(week39.contains("21.09–27.09"), "week39: {week39}");
    assert!(week39.contains("week 39 volume"), "week39: {week39}");

    let before = fs::read_to_string(root.path().join("reports/DASHBOARD.md")).unwrap();
    report::run(root.path(), today).unwrap();
    let after = fs::read_to_string(root.path().join("reports/DASHBOARD.md")).unwrap();
    assert_eq!(before, after, "report must be idempotent");
}

#[test]
fn report_tolerates_empty_data_dirs() {
    let root = TempDir::new().unwrap();
    report::run(root.path(), d("2026-10-04")).unwrap();
    let dashboard = fs::read_to_string(root.path().join("reports/DASHBOARD.md")).unwrap();
    assert!(
        dashboard.contains("Запланированных стартов нет"),
        "dashboard: {dashboard}"
    );
    assert!(
        dashboard.contains("Жалоб в тренировках за последние 14 дней нет"),
        "dashboard: {dashboard}"
    );
}

#[test]
fn validate_accepts_good_data() {
    let root = TempDir::new().unwrap();
    write_file(root.path(), "data/athlete/alexey.yaml", ATHLETE_YAML);
    write_file(
        root.path(),
        "data/races/races.yaml",
        "races:\n  - {date: 2026-10-25, name: \"October park race\", kind: park-5k, priority: benchmark}\nresults: []\n",
    );
    write_file(
        root.path(),
        "data/workouts/2026-09-28-6x3min-steady.yaml",
        &workout_yaml("2026-09-28", "6x3min steady", "session", 9.5, 3000),
    );
    write_file(
        root.path(),
        "data/plan/2026-W40.yaml",
        "week: 2026-W40\nmonday: 2026-09-28\nphase: base\nitems:\n  - id: mon\n    date: 2026-09-28\n    kind: session\n    title: \"6x3min steady\"\n    structure: []\n    status: done\n    notes: \"\"\nnotes: \"\"\n",
    );
    let v = validate::run(root.path()).unwrap();
    assert!(v.errors.is_empty(), "errors: {:#?}", v.errors);
}

#[test]
fn validate_reports_schema_errors() {
    let root = TempDir::new().unwrap();
    write_file(root.path(), "data/athlete/alexey.yaml", ATHLETE_YAML);
    write_file(
        root.path(),
        "data/workouts/2026-09-28-tempo-run.yaml",
        &workout_yaml("2026-09-28", "Tempo run", "tempo", 8.0, 2400),
    );

    let mut bad_zones = workout_yaml("2026-09-29", "Zone run", "easy", 8.0, 2400);
    bad_zones = bad_zones.replace("hr_zones_s: null", "hr_zones_s: [100, 100, 100, 100, 100]");
    write_file(
        root.path(),
        "data/workouts/2026-09-29-zone-run.yaml",
        &bad_zones,
    );

    let mut bad_ref = workout_yaml("2026-09-30", "Ghost run", "easy", 8.0, 2400);
    bad_ref = bad_ref.replace("source: gpx", "source: gpx\nplan_ref: 2026-W99/mon");
    write_file(
        root.path(),
        "data/workouts/2026-09-30-ghost-run.yaml",
        &bad_ref,
    );

    let mut bad_rpe = workout_yaml("2026-10-01", "Hard run", "easy", 8.0, 2400);
    bad_rpe = bad_rpe.replace("rpe: null", "rpe: 12");
    write_file(
        root.path(),
        "data/workouts/2026-10-01-hard-run.yaml",
        &bad_rpe,
    );

    write_file(
        root.path(),
        "data/plan/2026-W41.yaml",
        "week: 2026-W41\nmonday: 2026-10-07\nphase: base\nitems:\n  - id: mon\n    date: 2026-10-05\n    kind: easy\n    title: \"\"\n    structure: []\n    status: moved\n    notes: \"\"\nnotes: \"\"\n",
    );

    let v = validate::run(root.path()).unwrap();
    assert!(!v.errors.is_empty());
    let joined = v.errors.join("\n");
    assert!(joined.contains("unknown variant `tempo`"), "got: {joined}");
    assert!(
        joined.contains("hr_zones_s must have 6 entries"),
        "got: {joined}"
    );
    assert!(joined.contains("2026-W99"), "got: {joined}");
    assert!(joined.contains("rpe"), "got: {joined}");
    assert!(joined.contains("not a Monday"), "got: {joined}");
    assert!(
        joined.contains("status moved but no moved_to"),
        "got: {joined}"
    );
}

#[test]
fn validate_warns_on_guardrails() {
    let root = TempDir::new().unwrap();
    write_file(root.path(), "data/athlete/alexey.yaml", ATHLETE_YAML);

    let base = "2026-08-31 2026-09-07 2026-09-14".split_whitespace();
    for date in base {
        let slug = format!("steady-{date}");
        let yaml = workout_yaml(date, &slug, "easy", 40.0, 12000);
        write_file(
            root.path(),
            &format!("data/workouts/{date}-{slug}.yaml"),
            &yaml,
        );
    }
    let spike = workout_yaml("2026-09-21", "Big week", "easy", 70.0, 21000);
    write_file(
        root.path(),
        "data/workouts/2026-09-21-big-week.yaml",
        &spike,
    );

    let mut s1 = workout_yaml("2026-09-28", "Strides am", "easy", 5.0, 1500);
    s1 = s1.replace("notes: \"\"", "notes: \"6 strides after\"");
    write_file(root.path(), "data/workouts/2026-09-28-strides-am.yaml", &s1);
    write_file(
        root.path(),
        "data/workouts/2026-09-29-strides-pm.yaml",
        &workout_yaml("2026-09-29", "Strides pm", "easy", 5.0, 1500),
    );
    write_file(
        root.path(),
        "data/workouts/2026-09-30-strides-again.yaml",
        &workout_yaml("2026-09-30", "Strides again", "easy", 5.0, 1500),
    );

    write_file(
        root.path(),
        "data/plan/2026-W41.yaml",
        r#"week: 2026-W41
monday: 2026-10-05
phase: build
items:
  - {id: mon, date: 2026-10-05, kind: session, title: "intervals", structure: [], status: planned, notes: ""}
  - {id: tue, date: 2026-10-06, kind: session, title: "tempo", structure: [], status: planned, notes: ""}
  - {id: wed, date: 2026-10-07, kind: session, title: "fartlek", structure: [], status: planned, notes: ""}
  - {id: fri, date: 2026-10-09, kind: race, title: "test 5k", structure: [], status: planned, notes: ""}
notes: ""
"#,
    );

    let v = validate::run(root.path()).unwrap();
    assert!(v.errors.is_empty(), "errors: {:#?}", v.errors);
    let joined = v.warnings.join("\n");
    assert!(
        joined.contains("volume guardrail"),
        "expected volume warning, got: {joined}"
    );
    assert!(
        joined.contains("strides guardrail"),
        "expected strides warning, got: {joined}"
    );
    assert!(
        joined.contains("quality guardrail"),
        "expected quality warning, got: {joined}"
    );
}

#[test]
fn plan_new_scaffolds_from_weekly_template() {
    let root = TempDir::new().unwrap();
    write_file(root.path(), "data/athlete/alexey.yaml", ATHLETE_YAML);

    scaffold::run(
        root.path(),
        d("2026-10-05"),
        runkit::model::Phase::base,
        false,
    )
    .unwrap();
    let yaml = fs::read_to_string(root.path().join("data/plan/2026-W41.yaml")).unwrap();
    assert!(yaml.contains("week: 2026-W41"), "yaml: {yaml}");
    assert!(yaml.contains("monday: 2026-10-05"), "yaml: {yaml}");
    assert!(yaml.contains("id: sun"), "yaml: {yaml}");
    assert!(yaml.contains("date: 2026-10-11"), "yaml: {yaml}");
    assert_eq!(yaml.matches("status: planned").count(), 7);
    assert!(yaml.contains("notes: 'slot: quality'"), "yaml: {yaml}");
    assert!(yaml.contains("kind: hills"), "yaml: {yaml}");
    assert!(yaml.contains("kind: rest"), "yaml: {yaml}");
    assert!(yaml.contains("kind: long"), "yaml: {yaml}");

    let err = scaffold::run(
        root.path(),
        d("2026-10-05"),
        runkit::model::Phase::base,
        false,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("--force"), "got: {err}");
    scaffold::run(
        root.path(),
        d("2026-10-05"),
        runkit::model::Phase::base,
        true,
    )
    .unwrap();
}

#[test]
fn plan_new_requires_monday() {
    let root = TempDir::new().unwrap();
    write_file(root.path(), "data/athlete/alexey.yaml", ATHLETE_YAML);
    let err = scaffold::run(
        root.path(),
        d("2026-10-07"),
        runkit::model::Phase::base,
        false,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("Monday"), "got: {err}");
}

#[test]
fn plan_new_requires_athlete_profile() {
    let root = TempDir::new().unwrap();
    let err = scaffold::run(
        root.path(),
        d("2026-10-05"),
        runkit::model::Phase::base,
        false,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("athlete"), "got: {err}");
}
