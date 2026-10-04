import type { PlanWeek, Workout } from '@/lib/data';
import { mondayOfWeekId, weekDates } from '@/lib/dates';
import { paceSPerKm } from '@/lib/format';

export interface WeekRow {
  key: string;
  title: string;
  href: string | null;
  kind: string | null;
  status: string;
  distanceKm: number | null;
  paceS: number | null;
  meta: string | null;
}

export interface WeekDay {
  date: string;
  rows: WeekRow[];
}

const DAY_IDS = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];

function dayIndexOfItemId(id: string): number {
  const base = id.replace(/\d+$/, '');
  const idx = DAY_IDS.indexOf(base);
  return idx;
}

export function buildWeekModel(plan: PlanWeek | null, workouts: Workout[], weekId: string): WeekDay[] {
  const monday = plan?.monday ?? mondayOfWeekId(weekId);
  const days = monday ? weekDates(monday) : [];
  if (days.length === 0) return [];

  const items = plan?.items ?? [];
  const byRef = new Map<string, Workout>();
  for (const w of workouts) {
    if (w.planRef && w.planRef.startsWith(`${weekId}/`)) {
      byRef.set(w.planRef, w);
    }
  }

  const claimed = new Set<string>();
  const rowsFor = (date: string): WeekRow[] => {
    const rows: WeekRow[] = [];
    const dayIdx = days.indexOf(date);
    const dayItems = items.filter((it) => {
      if (it.date) return it.date === date;
      return dayIndexOfItemId(it.id) === dayIdx;
    });

    for (const it of dayItems) {
      const ref = `${weekId}/${it.id}`;
      let w: Workout | null | undefined = byRef.get(ref);
      if (!w) {
        w =
          workouts.find(
            (cand) => !claimed.has(cand.slug) && !cand.planRef && cand.date === date && it.kind !== 'rest' && (cand.status ?? 'done') === 'done',
          ) ?? null;
      }
      if (w) claimed.add(w.slug);
      const status = w ? 'done' : it.status;
      const metaParts: string[] = [];
      if (it.movedTo) metaParts.push(`перенос → ${it.movedTo}`);
      if (!w && it.structure.length > 0) metaParts.push(`структура: ${it.structure.length} сегм.`);
      rows.push({
        key: it.id,
        title: it.title,
        href: w ? `/workouts/${w.slug}` : null,
        kind: it.kind,
        status,
        distanceKm: w?.totals.distanceKm ?? null,
        paceS: w ? paceSPerKm(w.totals.movingTimeS, w.totals.distanceKm) : null,
        meta: metaParts.length > 0 ? metaParts.join(' · ') : null,
      });
    }

    for (const w of workouts) {
      if (w.date !== date || claimed.has(w.slug)) continue;
      claimed.add(w.slug);
      const extra = w.planRef && !w.planRef.startsWith(`${weekId}/`) ? `план: ${w.planRef}` : null;
      rows.push({
        key: w.slug,
        title: w.title,
        href: `/workouts/${w.slug}`,
        kind: w.kind,
        status: w.status ?? 'done',
        distanceKm: w.totals.distanceKm,
        paceS: paceSPerKm(w.totals.movingTimeS, w.totals.distanceKm),
        meta: extra,
      });
    }

    return rows;
  };

  return days.map((date) => ({ date, rows: rowsFor(date) }));
}

export function weekModelStats(days: WeekDay[]) {
  let planned = 0;
  let done = 0;
  let km = 0;
  for (const d of days) {
    for (const r of d.rows) {
      if (r.kind === 'rest') continue;
      planned += 1;
      if (r.status === 'done') done += 1;
      if (r.status === 'done' && r.distanceKm != null) km += r.distanceKm;
    }
  }
  return { planned, done, km };
}
