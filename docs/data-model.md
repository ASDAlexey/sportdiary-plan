# Контракт данных

Простой текст на первом месте, дружелюбный к git. YAML везде: читаемые диффы, правится и рукой,
и агентом, парсится из Rust и JS. Этот файл — контракт между данными, CLI `runkit`, сайтом на
Next.js и любым агентом (Claude Code, Codex, GLM), правящим репозиторий. Меняется только вместе
с `tools/runkit` и `site/`.

## Раскладка

```
data/
  athlete/alexey.yaml      # профиль: цели, пульсовые зоны, шаблон недели, травмы, обувь
  workouts/*.yaml          # один файл на тренировку (выполненную или записанную вручную)
  plan/<isoweek>.yaml      # один файл на плановую неделю, напр. 2026-W41.yaml
  goals/5k-sub20.md        # документ цели: фазы, история, решения
  races/races.yaml         # календарь гонок + результаты
  health/                  # только локально (gitignored): сон/HRV
raw/                       # только локально (gitignored): GPX/TCX/FIT с треками
reports/                   # ГЕНЕРИРУЕТСЯ `runkit report` — руками не править
```

Правило приватности: в `data/` — только метрики. Без GPS-треков, без координат, без точных точек
старта. Сырые файлы с треками остаются в `raw/` (gitignored). Данные здоровья — в `data/health/`
(gitignored).

Пульсовые зоны — шестизонная модель Coros, один и тот же порядок везде:

| # | Ключ | Зона |
|---|---|---|
| 1 | `recovery` | восстановление |
| 2 | `aerobic_endurance` | аэробная выносливость |
| 3 | `aerobic_power` | аэробная мощность |
| 4 | `threshold` | ПАНО |
| 5 | `anaerobic_endurance` | анаэробная выносливость |
| 6 | `anaerobic_power` | анаэробная мощность |

## Тренировка — `data/workouts/<дата>-<slug>.yaml`

`<дата>` — `YYYY-MM-DD` (локальная дата пробежки), `<slug>` — короткий ascii-слаг заголовка.
Повторный импорт той же тренировки атомарно заменяет файл.

```yaml
date: 2026-10-05
title: "6x3min ровно"
kind: session        # easy | long | session | hills | race | strength | rest
source: coros-mcp    # coros-mcp | coros-hub | strava | gpx | tcx | json | manual
status: done         # done | planned | skipped | moved
plan_ref: 2026-W41/mon   # необязательно: id планового item, который закрывает тренировка
totals:
  distance_km: 10.4
  moving_time_s: 3180     # секунды
  elevation_m: 15         # набор, метры; может быть null
  avg_hr: 156             # необязательно
  max_hr: 174             # необязательно
structure: []             # необязательно, сегменты по порядку:
  # - name: warmup
  #   distance_km: 2.0
  #   time_s: 720
  # - name: "3min"        # повторяемый сегмент
  #   repeat: 6
  #   time_s: 190
  #   pace_s_per_km: 250
  #   rest: "90s jog"
  # - name: cooldown
  #   distance_km: 2.2
rpe: 6                    # субъективно 1-10, необязательно
hr_zones_s: [1200, 900, 600, 400, 80, 0]   # секунды по зонам 1-6, порядок выше; необязательно
notes: ""                 # свободный текст, по-русски
shoes: lining-x9          # id из athlete.shoes, необязательно
injuries: []              # метки вроде heel-tightness; пусто, если чисто
```

Никто не хранит производное: темп = `moving_time_s / distance_km`; неделя = ISO-неделя даты
(`2026-W41`).

## План — `data/plan/<isoweek>.yaml`

Один файл на неделю, ключ — id ISO-недели, `monday` — её понедельник. У items стабильные id:
`mon`, `tue`, `wed`, `thu`, `fri`, `sat`, `sun`; вторая тренировка в день — `wed2`.

```yaml
week: 2026-W41
monday: 2026-10-05
phase: base          # base | build | peak | sharpen | race | deload
items:
  - id: mon
    date: 2026-10-05
    kind: session
    title: "6x3min ровно"
    structure: []           # тот же формат сегментов, что у тренировки
    status: planned         # planned | done | partial | moved | skipped
    moved_to: 2026-W42      # только при status: moved
notes: ""
```

Тренировка закрывает плановый item через `plan_ref: "<week>/<item-id>"`.

## Спортсмен — `data/athlete/alexey.yaml`

```yaml
name: Alexey
resting_hr: 59
max_hr: 208
threshold_hr: 187                 # Coros ПАНО (LTHR)
threshold_pace_s_per_km: 260      # 4:20/км
hr_zones:                         # 6 зон Coros, уд/мин
  recovery:            {low: 0,   high: 149}
  aerobic_endurance:   {low: 150, high: 168}
  aerobic_power:       {low: 169, high: 178}
  threshold:           {low: 179, high: 191}
  anaerobic_endurance: {low: 192, high: 198}
  anaerobic_power:     {low: 199, high: 208}
weekly_template:            # скелет недели; агент управляет слотами с controlled: true
  mon: {slot: quality, controlled: true}
  tue: {slot: secondary, controlled: true}
  wed: {slot: easy-am, controlled: true}
  thu: {slot: quality-group, controlled: false}   # групповая, содержание заранее неизвестно
  fri: {slot: rest, controlled: true}
  sat: {slot: hills-group, controlled: false}     # групповые горки 400-600 м
  sun: {slot: long, controlled: true}
guardrails:
  max_weekly_volume_growth: 1.10   # к среднему за предыдущие 4 недели
  max_controlled_quality_days: 3
  max_strides_days: 2              # осторожность с пяткой: strides по мягкому, не в максимум
injury_history: [...]
shoes: [...]
```

## Гонки — `data/races/races.yaml`

```yaml
races:
  - {date: 2026-10-25, name: "Октябрьский парковый забег", kind: park-5k, priority: benchmark}
  - {date: 2027-02-28, name: "Сочи, конец февраля", kind: road, priority: B}
  - {date: 2027-04-25, name: "Апрельский парковый забег", kind: park-5k, priority: A, target: sub-20}
results:
  - {date: 2026-09-27, name: "Сентябрьский парковый", kind: park-5k, time_s: 1284, note: "21:24 на ~5,1 км"}
```

## Сгенерированные отчеты — `reports/`

`runkit report` перезаписывает `reports/DASHBOARD.md` (текущее состояние: план-vs-факт недели,
тренд объема, обратный отсчет до гонки, флаги травм) и `reports/week-<isoweek>.md` по каждой
неделе. Идемпотентно: два запуска подряд не меняют ничего. Агентам не править руками —
перезапускать инструмент. Тексты отчетов — по-русски.
