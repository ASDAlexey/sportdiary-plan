use runkit::coros;
use std::fs;
use tempfile::tempdir;

fn record(id: &str) -> String {
    format!(
        r#"[{{"id":"{id}","date":"2026-10-03","sport_type":100,"distance_km":6.01,"moving_time_s":2284,"avg_hr":145}}]"#
    )
}

#[test]
fn sync_preserves_manual_fields_and_is_idempotent() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("data/workouts")).unwrap();
    let target = root.path().join("data/workouts/2026-10-03-existing.yaml");
    fs::write(&target, "date: 2026-10-03\ntitle: Возвращение\nkind: race\nsource: manual\nstatus: done\ntotals:\n  distance_km: 6.01\n  moving_time_s: 2284\nstructure: []\nrpe: 6\nnotes: Моя заметка\nshoes: lining-x9\ninjuries: [heel-tightness]\n").unwrap();
    let input = root.path().join("input.json");
    fs::write(&input, record("123")).unwrap();
    coros::run(root.path(), &input).unwrap();
    let first = fs::read_to_string(&target).unwrap();
    assert!(first.contains("kind: race"));
    assert!(first.contains("Моя заметка"));
    assert!(first.contains("lining-x9"));
    assert!(first.contains("heel-tightness"));
    assert!(first.contains("avg_hr: 145"));
    coros::run(root.path(), &input).unwrap();
    assert_eq!(fs::read_to_string(target).unwrap(), first);
    assert_eq!(
        fs::read_dir(root.path().join("data/workouts"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn rejects_coordinates_and_duplicate_ids_without_writing() {
    let root = tempdir().unwrap();
    let input = root.path().join("input.json");
    fs::write(
        &input,
        record("123").replace("\"avg_hr\":145", "\"avg_hr\":145,\"latitude\":1"),
    )
    .unwrap();
    assert!(coros::run(root.path(), &input).is_err());
    let single = record("123");
    let object = &single[1..single.len() - 1];
    fs::write(&input, format!("[{object},{object}]")).unwrap();
    assert!(coros::run(root.path(), &input).is_err());
    assert!(!root.path().join("data/workouts").exists());
}

#[test]
fn skips_nonrunning_and_keeps_same_day_runs_separate() {
    let root = tempdir().unwrap();
    let input = root.path().join("input.json");
    let first = record("1");
    let second = record("2");
    let third = record("3").replace("\"sport_type\":100", "\"sport_type\":200");
    fs::write(
        &input,
        format!(
            "[{},{},{}]",
            &first[1..first.len() - 1],
            &second[1..second.len() - 1],
            &third[1..third.len() - 1]
        ),
    )
    .unwrap();
    coros::run(root.path(), &input).unwrap();
    assert_eq!(
        fs::read_dir(root.path().join("data/workouts"))
            .unwrap()
            .count(),
        2
    );
    assert!(!root
        .path()
        .join("data/workouts/2026-10-03-coros-3.yaml")
        .exists());
}
