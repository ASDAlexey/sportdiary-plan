import fs from 'node:fs';
import path from 'node:path';
import { parse as parseYaml } from 'yaml';

const DATA_DIR = path.join(process.cwd(), '..', 'data');

export interface Segment {
  name?: string | null;
  repeat?: number | null;
  distanceKm?: number | null;
  timeS?: number | null;
  paceSPerKm?: number | null;
  rest?: string | null;
  note?: string | null;
}

export interface WorkoutTotals {
  distanceKm: number | null;
  movingTimeS: number | null;
  elevationM: number | null;
  avgHr: number | null;
  maxHr: number | null;
}

export interface Workout {
  slug: string;
  date: string;
  title: string;
  kind: string | null;
  source: string | null;
  status: string | null;
  planRef: string | null;
  totals: WorkoutTotals;
  structure: Segment[];
  rpe: number | null;
  hrZonesS: number[] | null;
  notes: string | null;
  shoes: string | null;
  injuries: string[];
}

export interface PlanItem {
  id: string;
  date: string | null;
  kind: string | null;
  title: string;
  status: string;
  movedTo: string | null;
  structure: Segment[];
}

export interface PlanWeek {
  week: string;
  monday: string | null;
  phase: string | null;
  items: PlanItem[];
  notes: string | null;
}

export interface RaceEntry {
  date: string;
  name: string;
  kind: string | null;
  priority: string | null;
  target: string | null;
}

export interface RaceResult {
  date: string;
  name: string;
  kind: string | null;
  timeS: number | null;
  note: string | null;
}

export interface Athlete {
  name: string | null;
  restingHr: number | null;
  maxHr: number | null;
  thresholdHr: number | null;
  thresholdPaceSPerKm: number | null;
  hrZones: Record<string, { low: number; high: number }> | null;
  shoes: { id: string; brand: string | null; model: string | null; status: string | null }[];
}

function num(v: unknown): number | null {
  return typeof v === 'number' && Number.isFinite(v) ? v : null;
}

function str(v: unknown): string | null {
  if (typeof v === 'string') return v.trim() || null;
  if (typeof v === 'number') return String(v);
  return null;
}

function isoDate(v: unknown): string | null {
  if (v instanceof Date) return v.toISOString().slice(0, 10);
  if (typeof v === 'string') {
    const m = /^(\d{4}-\d{2}-\d{2})/.exec(v.trim());
    if (m) return m[1];
  }
  return null;
}

function strArr(v: unknown): string[] {
  return Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string' && !!x.trim()) : [];
}

function readText(rel: string): string | null {
  try {
    return fs.readFileSync(path.join(DATA_DIR, rel), 'utf8');
  } catch {
    return null;
  }
}

function listFiles(rel: string, exts: string[]): string[] {
  try {
    return fs
      .readdirSync(path.join(DATA_DIR, rel), { withFileTypes: true })
      .filter((e) => e.isFile() && exts.some((ext) => e.name.endsWith(ext)))
      .map((e) => e.name)
      .sort();
  } catch {
    return [];
  }
}

function parseSegment(v: unknown): Segment {
  const o = (v ?? {}) as Record<string, unknown>;
  return {
    name: str(o.name),
    repeat: num(o.repeat),
    distanceKm: num(o.distance_km),
    timeS: num(o.time_s),
    paceSPerKm: num(o.pace_s_per_km),
    rest: str(o.rest),
    note: str(o.note),
  };
}

function parseSegments(v: unknown): Segment[] {
  return Array.isArray(v) ? v.map(parseSegment) : [];
}

let workoutsCache: Workout[] | null = null;

export function loadWorkouts(): Workout[] {
  if (workoutsCache) return workoutsCache;
  const workouts: Workout[] = [];
  for (const file of listFiles('workouts', ['.yaml', '.yml'])) {
    const raw = readText(path.join('workouts', file));
    if (raw == null) continue;
    let doc: Record<string, unknown>;
    try {
      doc = (parseYaml(raw) ?? {}) as Record<string, unknown>;
    } catch {
      continue;
    }
    const date = isoDate(doc.date);
    if (!date) continue;
    const totalsRaw = (doc.totals ?? {}) as Record<string, unknown>;
    const zonesRaw = doc.hr_zones_s;
    workouts.push({
      slug: file.replace(/\.ya?ml$/, ''),
      date,
      title: str(doc.title) ?? file.replace(/\.ya?ml$/, ''),
      kind: str(doc.kind),
      source: str(doc.source),
      status: str(doc.status),
      planRef: str(doc.plan_ref),
      totals: {
        distanceKm: num(totalsRaw.distance_km),
        movingTimeS: num(totalsRaw.moving_time_s),
        elevationM: num(totalsRaw.elevation_m),
        avgHr: num(totalsRaw.avg_hr),
        maxHr: num(totalsRaw.max_hr),
      },
      structure: parseSegments(doc.structure),
      rpe: num(doc.rpe),
      hrZonesS: Array.isArray(zonesRaw) ? zonesRaw.map((z) => num(z) ?? 0) : null,
      notes: str(doc.notes),
      shoes: str(doc.shoes),
      injuries: strArr(doc.injuries),
    });
  }
  workouts.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : b.slug.localeCompare(a.slug)));
  workoutsCache = workouts;
  return workouts;
}

export function isCounted(w: Workout): boolean {
  return (!w.status || w.status === 'done') && w.totals.distanceKm != null;
}

let plansCache: Map<string, PlanWeek> | null = null;

export function loadPlans(): Map<string, PlanWeek> {
  if (plansCache) return plansCache;
  const map = new Map<string, PlanWeek>();
  for (const file of listFiles('plan', ['.yaml', '.yml'])) {
    const raw = readText(path.join('plan', file));
    if (raw == null) continue;
    let doc: Record<string, unknown>;
    try {
      doc = (parseYaml(raw) ?? {}) as Record<string, unknown>;
    } catch {
      continue;
    }
    const week = str(doc.week) ?? file.replace(/\.ya?ml$/, '');
    const itemsRaw = Array.isArray(doc.items) ? doc.items : [];
    const items: PlanItem[] = itemsRaw.map((it, i) => {
      const o = (it ?? {}) as Record<string, unknown>;
      return {
        id: str(o.id) ?? `item-${i + 1}`,
        date: isoDate(o.date),
        kind: str(o.kind),
        title: str(o.title) ?? '—',
        status: str(o.status) ?? 'planned',
        movedTo: str(o.moved_to),
        structure: parseSegments(o.structure),
      };
    });
    map.set(week, {
      week,
      monday: isoDate(doc.monday),
      phase: str(doc.phase),
      items,
      notes: str(doc.notes),
    });
  }
  plansCache = map;
  return map;
}

export function loadPlan(weekId: string): PlanWeek | null {
  return loadPlans().get(weekId) ?? null;
}

let racesCache: { races: RaceEntry[]; results: RaceResult[] } | null = null;

export function loadRaces(): { races: RaceEntry[]; results: RaceResult[] } {
  if (racesCache) return racesCache;
  const raw = readText(path.join('races', 'races.yaml'));
  let doc: Record<string, unknown> = {};
  if (raw != null) {
    try {
      doc = (parseYaml(raw) ?? {}) as Record<string, unknown>;
    } catch {
      doc = {};
    }
  }
  const races: RaceEntry[] = (Array.isArray(doc.races) ? doc.races : [])
    .map((r): RaceEntry | null => {
      const o = (r ?? {}) as Record<string, unknown>;
      const date = isoDate(o.date);
      if (!date) return null;
      return {
        date,
        name: str(o.name) ?? '—',
        kind: str(o.kind),
        priority: str(o.priority),
        target: str(o.target),
      };
    })
    .filter((r): r is RaceEntry => r !== null)
    .sort((a, b) => a.date.localeCompare(b.date));
  const results: RaceResult[] = (Array.isArray(doc.results) ? doc.results : [])
    .map((r): RaceResult | null => {
      const o = (r ?? {}) as Record<string, unknown>;
      const date = isoDate(o.date);
      if (!date) return null;
      return {
        date,
        name: str(o.name) ?? '—',
        kind: str(o.kind),
        timeS: num(o.time_s),
        note: str(o.note),
      };
    })
    .filter((r): r is RaceResult => r !== null)
    .sort((a, b) => b.date.localeCompare(a.date));
  racesCache = { races, results };
  return racesCache;
}

let athleteCache: Athlete | null = null;
let athleteLoaded = false;

export function loadAthlete(): Athlete | null {
  if (athleteLoaded) return athleteCache;
  athleteLoaded = true;
  const file = listFiles('athlete', ['.yaml', '.yml'])[0];
  if (!file) return null;
  const raw = readText(path.join('athlete', file));
  if (raw == null) return null;
  try {
    const doc = (parseYaml(raw) ?? {}) as Record<string, unknown>;
    const zones: Record<string, { low: number; high: number }> = {};
    const zonesRaw = doc.hr_zones;
    if (zonesRaw && typeof zonesRaw === 'object' && !Array.isArray(zonesRaw)) {
      for (const [key, val] of Object.entries(zonesRaw as Record<string, unknown>)) {
        const o = (val ?? {}) as Record<string, unknown>;
        const low = num(o.low);
        const high = num(o.high);
        if (low != null || high != null) zones[key] = { low: low ?? 0, high: high ?? 0 };
      }
    }
    const shoes = (Array.isArray(doc.shoes) ? doc.shoes : []).map((s) => {
      const o = (s ?? {}) as Record<string, unknown>;
      return {
        id: str(o.id) ?? '',
        brand: str(o.brand),
        model: str(o.model),
        status: str(o.status),
      };
    });
    athleteCache = {
      name: str(doc.name),
      restingHr: num(doc.resting_hr),
      maxHr: num(doc.max_hr),
      thresholdHr: num(doc.threshold_hr),
      thresholdPaceSPerKm: num(doc.threshold_pace_s_per_km),
      hrZones: Object.keys(zones).length > 0 ? zones : null,
      shoes,
    };
    return athleteCache;
  } catch {
    return null;
  }
}

export function loadGoalDoc(): string | null {
  return readText(path.join('goals', '5k-sub20.md'));
}

export function dataDirExists(): boolean {
  try {
    return fs.statSync(DATA_DIR).isDirectory();
  } catch {
    return false;
  }
}
