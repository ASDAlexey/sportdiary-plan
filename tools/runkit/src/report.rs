use crate::dates::{
    day_name_ru, format_date_ru, format_hms, format_km, format_pace, iso_week_id, monday_of,
    plural_ru,
};
use crate::model::{Athlete, Plan, PlanItem, Races, Workout};
use crate::store::{load_all, write_if_changed};
use anyhow::Result;
use chrono::{Datelike, Duration, NaiveDate};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct ReportOutcome {
    pub dashboard_written: bool,
    pub week_files_written: Vec<String>,
}

pub fn run(root: &Path, today: NaiveDate) -> Result<ReportOutcome> {
    let loaded = load_all(root)?;
    let workouts: Vec<&Workout> = loaded.workouts.iter().map(|(_, w)| w).collect();
    let plans: BTreeMap<String, &Plan> = loaded
        .plans
        .iter()
        .map(|(_, p)| (p.week.clone(), p))
        .collect();

    let mut week_ids: BTreeSet<String> = workouts.iter().map(|w| iso_week_id(w.date)).collect();
    for id in plans.keys() {
        week_ids.insert(id.clone());
    }

    let reports_dir = root.join("reports");
    let mut week_files_written = Vec::new();
    for week_id in &week_ids {
        let Some(monday) = monday_from_week_id(week_id) else {
            eprintln!("warning: skipping week {week_id:?}: not an ISO week id");
            continue;
        };
        let week_workouts: Vec<&Workout> = workouts
            .iter()
            .copied()
            .filter(|w| iso_week_id(w.date) == *week_id)
            .collect();
        let content =
            render_week_file(week_id, monday, plans.get(week_id).copied(), &week_workouts);
        let path = reports_dir.join(format!("week-{week_id}.md"));
        if write_if_changed(&path, &content)? {
            week_files_written.push(format!("reports/week-{week_id}.md"));
        }
    }

    let athlete = loaded.athlete.as_ref().map(|(_, a)| a);
    let races = loaded.races.as_ref().map(|(_, r)| r);
    let dashboard = render_dashboard(today, &workouts, &plans, athlete, races);
    let dashboard_written = write_if_changed(&reports_dir.join("DASHBOARD.md"), &dashboard)?;

    Ok(ReportOutcome {
        dashboard_written,
        week_files_written,
    })
}

fn monday_from_week_id(week_id: &str) -> Option<NaiveDate> {
    let bytes = week_id.as_bytes();
    if bytes.len() != 8 || bytes[4] != b'-' || bytes[5] != b'W' {
        return None;
    }
    let year: i32 = week_id[0..4].parse().ok()?;
    let week: u32 = week_id[6..8].parse().ok()?;
    let jan4 = NaiveDate::from_ymd_opt(year, 1, 4)?;
    let jan4_monday = monday_of(jan4);
    let monday = jan4_monday + Duration::weeks(week as i64 - jan4.iso_week().week() as i64);
    if iso_week_id(monday) == week_id {
        Some(monday)
    } else {
        None
    }
}

struct DayRow {
    date: NaiveDate,
    title: String,
    kind: String,
    planned_km: Option<f64>,
    done: bool,
    distance_km: Option<f64>,
    time_s: Option<u32>,
    pace: Option<f64>,
    rpe: Option<String>,
    status: String,
}

fn workout_rows(workout: &Workout) -> DayRow {
    DayRow {
        date: workout.date,
        title: workout.title.clone(),
        kind: kind_word(workout.kind.as_str()),
        planned_km: None,
        done: true,
        distance_km: Some(workout.totals.distance_km),
        time_s: Some(workout.totals.moving_time_s),
        pace: workout.avg_pace_s_per_km(),
        rpe: workout.rpe.map(|r| r.to_string()),
        status: status_word(workout.status.as_str()),
    }
}

fn status_word(status: &str) -> String {
    let ru = match status {
        "done" => "выполнено",
        "planned" => "запланировано",
        "partial" => "частично",
        "moved" => "перенесено",
        "skipped" => "пропущено",
        other => other,
    };
    ru.to_string()
}

fn kind_word(kind: &str) -> String {
    let ru = match kind {
        "easy" => "кросс",
        "long" => "длинная",
        "session" => "тренировка",
        "hills" => "горки",
        "race" => "забег",
        "strength" => "силовая",
        "rest" => "отдых",
        "park-5k" => "парковый забег 5 км",
        "road-5k" => "ровный курс 5 км",
        other => other,
    };
    ru.to_string()
}

fn priority_word(priority: &str) -> String {
    match priority {
        "benchmark" => "бенчмарк".to_string(),
        other => other.to_string(),
    }
}

fn combine_rpse(rpse: Vec<Option<u8>>) -> Option<String> {
    let values: Vec<u8> = rpse.into_iter().flatten().collect();
    if values.is_empty() {
        None
    } else if values.windows(2).all(|w| w[0] == w[1]) {
        Some(values[0].to_string())
    } else {
        Some(
            values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("/"),
        )
    }
}

fn match_rows<'a>(week_id: &str, plan: Option<&'a Plan>, workouts: &[&'a Workout]) -> Vec<DayRow> {
    let Some(plan) = plan else {
        return workouts.iter().map(|w| workout_rows(w)).collect();
    };

    let mut matched: Vec<Vec<&Workout>> = vec![Vec::new(); plan.items.len()];
    let prefix = format!("{week_id}/");
    let mut remaining: Vec<&Workout> = Vec::new();

    for w in workouts {
        let by_ref = w
            .plan_ref
            .as_ref()
            .and_then(|r| r.strip_prefix(&prefix))
            .and_then(|id| plan.items.iter().position(|i| i.id == id));
        match by_ref {
            Some(idx) => matched[idx].push(w),
            None => remaining.push(w),
        }
    }

    let mut leftover: Vec<&Workout> = Vec::new();
    for w in remaining {
        let by_date = plan
            .items
            .iter()
            .enumerate()
            .find(|(idx, item)| item.date == w.date && matched[*idx].is_empty())
            .map(|(idx, _)| idx);
        match by_date {
            Some(idx) => matched[idx].push(w),
            None => leftover.push(w),
        }
    }

    let mut rows = Vec::new();
    for (idx, item) in plan.items.iter().enumerate() {
        let ws = &matched[idx];
        let distance: f64 = ws.iter().map(|w| w.totals.distance_km).sum();
        let time: u32 = ws.iter().map(|w| w.totals.moving_time_s).sum();
        let pace = (distance > 0.0).then_some(time as f64 / distance);
        let title = if item.title.is_empty() {
            ws.first().map(|w| w.title.clone()).unwrap_or_default()
        } else {
            item.title.clone()
        };
        rows.push(DayRow {
            date: item.date,
            title,
            kind: kind_word(item.kind.as_str()),
            planned_km: Some(item.planned_distance_km()),
            done: !ws.is_empty(),
            distance_km: (!ws.is_empty()).then_some(distance),
            time_s: (!ws.is_empty()).then_some(time),
            pace,
            rpe: combine_rpse(ws.iter().map(|w| w.rpe).collect()),
            status: status_word(item.status.as_str()),
        });
    }
    rows.extend(leftover.into_iter().map(workout_rows));
    rows.sort_by_key(|r| r.date);
    rows
}

fn render_plan_table(
    week_id: &str,
    monday: NaiveDate,
    plan: Option<&Plan>,
    workouts: &[&Workout],
) -> String {
    let mut out = String::new();
    out.push_str("| День | Название | Вид | План, км | Выполнено | Дистанция, км | Время | Темп | RPE | Статус |\n");
    out.push_str("| --- | --- | --- | ---: | --- | ---: | ---: | ---: | ---: | --- |\n");

    let mut by_date: BTreeMap<NaiveDate, Vec<DayRow>> = BTreeMap::new();
    for row in match_rows(week_id, plan, workouts) {
        by_date.entry(row.date).or_default().push(row);
    }

    for i in 0..7 {
        let date = monday + Duration::days(i);
        let rows = by_date.remove(&date);
        match rows {
            Some(rows) if !rows.is_empty() => {
                for (n, row) in rows.iter().enumerate() {
                    let day_label = if n == 0 {
                        format!("{} {}", day_name_ru(date), format_date_ru(date))
                    } else {
                        String::new()
                    };
                    push_row(&mut out, &day_label, row);
                }
            }
            _ => {
                out.push_str(&format!(
                    "| {} {} | — | — | — | — | — | — | — | — | — |\n",
                    day_name_ru(date),
                    format_date_ru(date)
                ));
            }
        }
    }
    for (date, rows) in by_date {
        for (n, row) in rows.iter().enumerate() {
            let day_label = if n == 0 {
                format!("{} {}", day_name_ru(date), format_date_ru(date))
            } else {
                String::new()
            };
            push_row(&mut out, &day_label, row);
        }
    }
    out
}

fn push_row(out: &mut String, day_label: &str, row: &DayRow) {
    let planned = row
        .planned_km
        .filter(|km| *km > 0.0)
        .map(format_km)
        .unwrap_or_else(|| "—".to_string());
    let done = if row.done { "✓" } else { "—" };
    let distance = row
        .distance_km
        .map(format_km)
        .unwrap_or_else(|| "—".to_string());
    let time = row
        .time_s
        .map(format_hms)
        .unwrap_or_else(|| "—".to_string());
    let pace = row.pace.map(format_pace).unwrap_or_else(|| "—".to_string());
    let rpe = row.rpe.clone().unwrap_or_else(|| "—".to_string());
    let title = if row.title.is_empty() {
        "—"
    } else {
        &row.title
    };
    let kind = if row.kind.is_empty() {
        "—"
    } else {
        &row.kind
    };
    out.push_str(&format!(
        "| {day_label} | {title} | {kind} | {planned} | {done} | {distance} | {time} | {pace} | {rpe} | {} |\n",
        row.status
    ));
}

fn render_week_file(
    week_id: &str,
    monday: NaiveDate,
    plan: Option<&Plan>,
    workouts: &[&Workout],
) -> String {
    let sunday = monday + Duration::days(6);
    let mut out = String::new();
    out.push_str(&format!(
        "# Неделя {} ({}–{})\n\n",
        week_id,
        format_date_ru(monday),
        format_date_ru(sunday)
    ));
    if let Some(plan) = plan {
        out.push_str(&format!("Фаза: {}\n\n", plan.phase.as_str()));
    }

    out.push_str("## План и выполнение\n\n");
    out.push_str(&render_plan_table(week_id, monday, plan, workouts));
    out.push('\n');

    let total_km: f64 = workouts.iter().map(|w| w.totals.distance_km).sum();
    let total_time: u32 = workouts.iter().map(|w| w.totals.moving_time_s).sum();
    let elevations: Vec<u32> = workouts
        .iter()
        .filter_map(|w| w.totals.elevation_m)
        .collect();
    let elevation_sum: u32 = elevations.iter().sum();
    let has_elevation = !workouts.is_empty() && elevations.len() == workouts.len();

    out.push_str("## Итоги недели\n\n");
    out.push_str(&format!("- Тренировок: {}\n", workouts.len()));
    out.push_str(&format!("- Дистанция: {} км\n", format_km(total_km)));
    out.push_str(&format!("- Время: {}\n", format_hms(total_time)));
    if total_km > 0.0 {
        out.push_str(&format!(
            "- Средний темп: {}/км\n",
            format_pace(total_time as f64 / total_km)
        ));
    } else {
        out.push_str("- Средний темп: —\n");
    }
    if has_elevation {
        out.push_str(&format!("- Набор высоты: {} м\n", elevation_sum));
    }

    let mut kinds: BTreeMap<String, (usize, f64, u32)> = BTreeMap::new();
    for w in workouts {
        let e = kinds.entry(kind_word(w.kind.as_str())).or_default();
        e.0 += 1;
        e.1 += w.totals.distance_km;
        e.2 += w.totals.moving_time_s;
    }
    if !kinds.is_empty() {
        out.push_str("\n## По видам\n\n");
        out.push_str("| Вид | Тренировки | Дистанция, км | Время |\n");
        out.push_str("| --- | ---: | ---: | ---: |\n");
        for (kind, (count, km, time)) in kinds {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                kind,
                count,
                format_km(km),
                format_hms(time)
            ));
        }
    }

    let mut notes: Vec<String> = Vec::new();
    if let Some(plan) = plan {
        if !plan.notes.trim().is_empty() {
            notes.push(format!("План: {}", plan.notes.trim()));
        }
    }
    for w in workouts {
        if !w.notes.trim().is_empty() {
            notes.push(format!(
                "{} — {}: {}",
                format_date_ru(w.date),
                w.title,
                w.notes.trim()
            ));
        }
    }
    if !notes.is_empty() {
        out.push_str("\n## Заметки\n\n");
        for n in notes {
            out.push_str(&format!("- {n}\n"));
        }
    }
    out
}

fn weekly_volume(workouts: &[&Workout], week_id: &str) -> f64 {
    workouts
        .iter()
        .filter(|w| iso_week_id(w.date) == week_id)
        .map(|w| w.totals.distance_km)
        .sum()
}

fn render_dashboard(
    today: NaiveDate,
    workouts: &[&Workout],
    plans: &BTreeMap<String, &Plan>,
    athlete: Option<&Athlete>,
    races: Option<&Races>,
) -> String {
    let mut out = String::new();
    let name = athlete.map(|a| a.name.as_str()).unwrap_or("athlete");
    out.push_str(&format!("# Беговой дашборд — {name}\n\n"));
    out.push_str(&format!("Отчёт на {}\n\n", today.format("%Y-%m-%d")));

    let current_week = iso_week_id(today);
    let current_monday = monday_of(today);
    let current_sunday = current_monday + Duration::days(6);
    let plan = plans.get(&current_week).copied();

    out.push_str(&format!(
        "## Текущая неделя {} ({}–{})\n\n",
        current_week,
        format_date_ru(current_monday),
        format_date_ru(current_sunday)
    ));

    let week_workouts: Vec<&Workout> = workouts
        .iter()
        .copied()
        .filter(|w| iso_week_id(w.date) == current_week)
        .collect();
    let done_km: f64 = week_workouts.iter().map(|w| w.totals.distance_km).sum();
    let done_time: u32 = week_workouts.iter().map(|w| w.totals.moving_time_s).sum();
    let planned_km: f64 = plan
        .map(|p| p.items.iter().map(PlanItem::planned_distance_km).sum())
        .unwrap_or(0.0);
    let planned_known = planned_km > 0.0;

    if planned_known {
        let pct = ((done_km / planned_km) * 100.0).round();
        out.push_str(&format!(
            "- План: {} км, выполнено: {} км ({:.0}%)\n",
            format_km(planned_km),
            format_km(done_km),
            pct
        ));
    } else {
        out.push_str(&format!(
            "- План: — (нет данных о плановой дистанции), выполнено: {} км\n",
            format_km(done_km)
        ));
    }
    out.push_str(&format!("- Время: {}\n", format_hms(done_time)));

    out.push_str("\n| День | План, км | Факт, км |\n");
    out.push_str("| --- | ---: | ---: |\n");
    for i in 0..7 {
        let date = current_monday + Duration::days(i);
        let day_plan: f64 = plan
            .map(|p| {
                p.items
                    .iter()
                    .filter(|item| item.date == date)
                    .map(PlanItem::planned_distance_km)
                    .sum()
            })
            .unwrap_or(0.0);
        let day_done: f64 = workouts
            .iter()
            .filter(|w| w.date == date)
            .map(|w| w.totals.distance_km)
            .sum();
        let plan_col = if day_plan > 0.0 {
            format_km(day_plan)
        } else {
            "—".to_string()
        };
        let done_col = if day_done > 0.0 {
            format_km(day_done)
        } else {
            "—".to_string()
        };
        out.push_str(&format!(
            "| {} {} | {plan_col} | {done_col} |\n",
            day_name_ru(date),
            format_date_ru(date)
        ));
    }

    let trailing: Vec<String> = (1..=4)
        .rev()
        .map(|back| {
            let monday = current_monday - Duration::weeks(back);
            iso_week_id(monday + Duration::days(3))
        })
        .collect();
    let trailing_total: f64 = trailing.iter().map(|w| weekly_volume(workouts, w)).sum();
    out.push_str(&format!(
        "\n## Объём\n\n- За предыдущие 4 недели: {} км (в среднем {} км/нед)\n",
        format_km(trailing_total),
        format_km(trailing_total / 4.0)
    ));
    out.push_str(&format!(
        "- С начала года: {} км\n",
        format_km(
            workouts
                .iter()
                .filter(|w| w.date.year() == today.year() && w.date <= today)
                .map(|w| w.totals.distance_km)
                .sum::<f64>()
        )
    ));

    out.push_str("\n## Следующий старт\n\n");
    let next = races
        .and_then(|r| {
            r.races
                .iter()
                .filter(|race| race.date >= today)
                .min_by_key(|race| race.date)
        })
        .map(|race| {
            let days = (race.date - today).num_days();
            let countdown = if days == 0 {
                "сегодня".to_string()
            } else {
                format!("через {}", plural_ru(days as u64, "день", "дня", "дней"))
            };
            format!(
                "- {} — {} ({}, приоритет: {}) — {}\n",
                race.name,
                race.date.format("%d.%m.%Y"),
                kind_word(&race.kind),
                priority_word(&race.priority),
                countdown
            )
        });
    match next {
        Some(line) => out.push_str(&line),
        None => out.push_str("- Запланированных стартов нет\n"),
    }

    out.push_str("\n## Травмы\n\n");
    let since = today - Duration::days(14);
    let mut tags: BTreeSet<String> = BTreeSet::new();
    for w in workouts
        .iter()
        .filter(|w| w.date >= since && w.date <= today)
    {
        for tag in &w.injuries {
            tags.insert(tag.clone());
        }
    }
    if tags.is_empty() {
        out.push_str("- Жалоб в тренировках за последние 14 дней нет\n");
    } else {
        out.push_str(&format!(
            "- Отметки в тренировках за последние 14 дней: {}\n",
            tags.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    let active_profile: Vec<&crate::model::Injury> = athlete
        .map(|a| a.injury_history.iter().filter(|i| i.is_active()).collect())
        .unwrap_or_default();
    if active_profile.is_empty() {
        out.push_str("- Активных отметок в профиле нет\n");
    } else {
        for injury in active_profile {
            out.push_str(&format!(
                "- {}: {} — {}\n",
                injury.region, injury.status, injury.note
            ));
        }
    }

    let mut recent: Vec<&&Workout> = workouts.iter().collect();
    recent.sort_by(|a, b| b.date.cmp(&a.date));
    out.push_str("\n## Последние тренировки\n\n");
    out.push_str("| Дата | Название | Вид | Дистанция, км | Время | Темп |\n");
    out.push_str("| --- | --- | --- | ---: | ---: | ---: |\n");
    for w in recent.iter().take(5) {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            w.date.format("%d.%m.%Y"),
            w.title,
            kind_word(w.kind.as_str()),
            format_km(w.totals.distance_km),
            format_hms(w.totals.moving_time_s),
            w.avg_pace_s_per_km()
                .map(format_pace)
                .unwrap_or_else(|| "—".to_string())
        ));
    }
    out
}
