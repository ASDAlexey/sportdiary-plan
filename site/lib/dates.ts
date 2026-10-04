const DAY_MS = 86_400_000;

function pad(n: number): string {
  return String(n).padStart(2, '0');
}

export function parseISO(iso: string): Date {
  return new Date(`${iso}T12:00:00Z`);
}

export function toISO(d: Date): string {
  return `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}`;
}

export function todayISO(): string {
  const n = new Date();
  return `${n.getFullYear()}-${pad(n.getMonth() + 1)}-${pad(n.getDate())}`;
}

export function isoWeek(date: Date): { year: number; week: number } {
  const d = new Date(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate(), 12));
  const dayNum = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() + 4 - dayNum);
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
  const week = Math.ceil(((d.getTime() - yearStart.getTime()) / DAY_MS + 1) / 7);
  return { year: d.getUTCFullYear(), week };
}

export function isoWeekIdOf(iso: string): string {
  const { year, week } = isoWeek(parseISO(iso));
  return `${year}-W${pad(week)}`;
}

export function mondayOfISO(iso: string): string {
  const d = parseISO(iso);
  const dayNum = d.getUTCDay() || 7;
  d.setUTCDate(d.getUTCDate() - (dayNum - 1));
  return toISO(d);
}

export function mondayOfWeekId(weekId: string): string {
  const m = /^(\d{4})-W(\d{2})$/.exec(weekId);
  if (!m) return '';
  const year = Number(m[1]);
  const week = Number(m[2]);
  const jan4 = new Date(Date.UTC(year, 0, 4, 12));
  const jan4Day = jan4.getUTCDay() || 7;
  const week1Monday = new Date(jan4);
  week1Monday.setUTCDate(jan4.getUTCDate() - (jan4Day - 1));
  const d = new Date(week1Monday);
  d.setUTCDate(d.getUTCDate() + (week - 1) * 7);
  return toISO(d);
}

export function weekDates(mondayIso: string): string[] {
  const d = parseISO(mondayIso);
  return Array.from({ length: 7 }, (_, i) => {
    const x = new Date(d);
    x.setUTCDate(x.getUTCDate() + i);
    return toISO(x);
  });
}

export function weekIdAtOffset(weekId: string, offset: number): string {
  const monday = mondayOfWeekId(weekId);
  if (!monday) return weekId;
  const d = parseISO(monday);
  d.setUTCDate(d.getUTCDate() + offset * 7);
  return isoWeekIdOf(toISO(d));
}

export function addDaysISO(iso: string, days: number): string {
  const d = parseISO(iso);
  d.setUTCDate(d.getUTCDate() + days);
  return toISO(d);
}

export function daysBetween(fromIso: string, toIso: string): number {
  return Math.round((parseISO(toIso).getTime() - parseISO(fromIso).getTime()) / DAY_MS);
}

const MONTHS = ['янв', 'фев', 'мар', 'апр', 'мая', 'июн', 'июл', 'авг', 'сен', 'окт', 'ноя', 'дек'];
const MONTHS_FULL = ['Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь', 'Июль', 'Август', 'Сентябрь', 'Октябрь', 'Ноябрь', 'Декабрь'];
const DAYS = ['Воскресенье', 'Понедельник', 'Вторник', 'Среда', 'Четверг', 'Пятница', 'Суббота'];
const DAYS_SHORT = ['вс', 'пн', 'вт', 'ср', 'чт', 'пт', 'сб'];

export function fmtDay(iso: string): string {
  const d = parseISO(iso);
  return `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]}`;
}

export function fmtDayYear(iso: string): string {
  const d = parseISO(iso);
  return `${d.getUTCDate()} ${MONTHS[d.getUTCMonth()]} ${d.getUTCFullYear()}`;
}

export function dayName(iso: string): string {
  return DAYS[parseISO(iso).getUTCDay()];
}

export function dayShort(iso: string): string {
  return DAYS_SHORT[parseISO(iso).getUTCDay()];
}

export function fmtMonthYear(month: string): string {
  const y = Number(month.slice(0, 4));
  const m = Number(month.slice(5, 7));
  return `${MONTHS_FULL[m - 1] ?? month} ${y}`;
}

export function weekRangeLabel(weekId: string): string {
  const monday = mondayOfWeekId(weekId);
  if (!monday) return weekId;
  const end = addDaysISO(monday, 6);
  const a = parseISO(monday);
  const b = parseISO(end);
  if (a.getUTCMonth() === b.getUTCMonth()) {
    return `${a.getUTCDate()}–${b.getUTCDate()} ${MONTHS[b.getUTCMonth()]} ${b.getUTCFullYear()}`;
  }
  if (a.getUTCFullYear() === b.getUTCFullYear()) {
    return `${a.getUTCDate()} ${MONTHS[a.getUTCMonth()]} – ${b.getUTCDate()} ${MONTHS[b.getUTCMonth()]} ${b.getUTCFullYear()}`;
  }
  return `${fmtDayYear(monday)} – ${fmtDayYear(end)}`;
}

export function pluralRu(n: number, one: string, few: string, many: string): string {
  const abs = Math.abs(n) % 100;
  const d = abs % 10;
  if (abs > 10 && abs < 20) return many;
  if (d > 1 && d < 5) return few;
  if (d === 1) return one;
  return many;
}

export function fmtDaysIn(n: number): string {
  if (n === 0) return 'сегодня';
  if (n === 1) return 'завтра';
  const word = pluralRu(n, 'день', 'дня', 'дней');
  return n < 0 ? `${-n} ${word} назад` : `через ${n} ${word}`;
}
