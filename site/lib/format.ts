function pad(n: number): string {
  return String(n).padStart(2, '0');
}

export function fmtKm(km: number | null | undefined, digits = 1): string {
  if (km == null || !Number.isFinite(km)) return '—';
  return km.toFixed(digits);
}

export function fmtKmUnit(km: number | null | undefined, digits = 1): string {
  if (km == null || !Number.isFinite(km)) return '—';
  return `${km.toFixed(digits)} км`;
}

export function fmtDuration(s: number | null | undefined): string {
  if (s == null || !Number.isFinite(s) || s < 0) return '—';
  const total = Math.round(s);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const sec = total % 60;
  return h > 0 ? `${h}:${pad(m)}:${pad(sec)}` : `${m}:${pad(sec)}`;
}

export function fmtPace(sPerKm: number | null | undefined): string {
  if (sPerKm == null || !Number.isFinite(sPerKm) || sPerKm <= 0) return '—';
  const total = Math.round(sPerKm);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${pad(s)}`;
}

export function paceSPerKm(timeS: number | null | undefined, distanceKm: number | null | undefined): number | null {
  if (timeS == null || distanceKm == null || !Number.isFinite(timeS) || !Number.isFinite(distanceKm)) return null;
  if (timeS <= 0 || distanceKm <= 0) return null;
  return timeS / distanceKm;
}

export function fmtInt(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return '—';
  return Math.round(n).toLocaleString('ru-RU');
}

export function fmtPct(n: number | null | undefined): string {
  if (n == null || !Number.isFinite(n)) return '—';
  return `${Math.round(n)}%`;
}
