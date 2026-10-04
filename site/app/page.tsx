import Link from 'next/link';
import WeeklyVolumeChart from '@/components/WeeklyVolumeChart';
import WeekBoard from '@/components/WeekBoard';
import { Card, EmptyState, PageHeader, StatTile } from '@/components/ui';
import { InjuriesChips, KindChip } from '@/components/Chips';
import { isCounted, loadPlan, loadRaces, loadWorkouts } from '@/lib/data';
import { addDaysISO, daysBetween, fmtDayYear, fmtDaysIn, isoWeekIdOf, pluralRu, todayISO, weekIdAtOffset, weekRangeLabel } from '@/lib/dates';
import { fmtDuration, fmtInt, fmtKm, fmtKmUnit, fmtPace, paceSPerKm } from '@/lib/format';
import { buildWeekModel, weekModelStats } from '@/lib/week';

export const metadata = { title: 'Дашборд' };

const INJURY_WINDOW_DAYS = 28;

export default function DashboardPage() {
  const today = todayISO();
  const currentWeek = isoWeekIdOf(today);
  const year = today.slice(0, 4);

  const workouts = loadWorkouts();
  const counted = workouts.filter(isCounted);
  const plan = loadPlan(currentWeek);
  const { races, results } = loadRaces();

  const weekDays = buildWeekModel(plan, workouts, currentWeek);
  const weekStats = weekModelStats(weekDays);

  const volumeWeeks = Array.from({ length: 12 }, (_, i) => {
    const weekId = weekIdAtOffset(currentWeek, i - 11);
    const km = counted.filter((w) => isoWeekIdOf(w.date) === weekId).reduce((s, w) => s + (w.totals.distanceKm ?? 0), 0);
    return { weekId, km: Math.round(km * 10) / 10 };
  });
  const month = today.slice(0, 7);
  const monthWorkouts = counted.filter((w) => w.date.startsWith(month) && w.date <= today);
  const monthKm = monthWorkouts.reduce((s, w) => s + (w.totals.distanceKm ?? 0), 0);
  const monthLabel = new Intl.DateTimeFormat('ru-RU', { month: 'long', year: 'numeric', timeZone: 'UTC' })
    .format(new Date(`${month}-01T00:00:00Z`));

  const ytd = counted.filter((w) => w.date.startsWith(year));
  const ytdKm = Math.round(ytd.reduce((s, w) => s + (w.totals.distanceKm ?? 0), 0) * 10) / 10;

  const upcoming = races.filter((r) => r.date >= today);
  const nextRace = upcoming[0] ?? null;
  const nextRaceDays = nextRace ? daysBetween(today, nextRace.date) : null;

  const latest = workouts.slice(0, 5);

  const injuryWindowStart = addDaysISO(today, -INJURY_WINDOW_DAYS);
  const injuryTags = new Map<string, string>();
  for (const w of workouts) {
    if (w.date < injuryWindowStart || w.date > today) continue;
    for (const tag of w.injuries) {
      if (!injuryTags.has(tag) || w.date > (injuryTags.get(tag) ?? '')) injuryTags.set(tag, w.date);
    }
  }

  return (
    <>
      <PageHeader title="Дашборд" subtitle={`Сегодня ${fmtDayYear(today)} · текущая неделя ${currentWeek}`} />

      <div className="stat-row">
        <StatTile
          label="Следующий старт"
          value={nextRace && nextRaceDays != null ? fmtDaysIn(nextRaceDays) : '—'}
          sub={nextRace ? `${fmtDayYear(nextRace.date)} · ${nextRace.name}` : 'нет запланированных стартов'}
        />
        <StatTile
          label="Объём за текущий месяц"
          value={fmtKm(monthKm)}
          unit="км"
          sub={monthLabel}
        />
        <StatTile
          label={`Пробег с начала ${year}`}
          value={fmtKm(ytdKm)}
          unit="км"
          sub={`${fmtInt(ytd.length)} ${pluralRu(ytd.length, 'тренировка', 'тренировки', 'тренировок')}`}
        />
        <StatTile
          label="Текущая неделя"
          value={fmtKm(weekStats.km)}
          unit="км"
          sub={`выполнено ${weekStats.done} из ${weekStats.planned}`}
        />
      </div>

      <Card
        title={`Неделя ${currentWeek}`}
        actions={<span className="sub">{weekRangeLabel(currentWeek)}</span>}
      >
        {plan == null && workouts.every((w) => isoWeekIdOf(w.date) !== currentWeek) ? (
          <EmptyState text="Плана на эту неделю нет, тренировки не записаны" />
        ) : (
          <>
            <WeekBoard days={weekDays} todayIso={today} />
            {plan == null ? <div className="board-note">Файла плана {currentWeek} нет — показаны только записанные тренировки</div> : null}
          </>
        )}
      </Card>

      <Card title="Недельный объём" actions={<span className="sub">последние 12 недель</span>}>
        <WeeklyVolumeChart weeks={volumeWeeks} currentWeekId={currentWeek} />
      </Card>

      <div className="grid-2">
        <Card title="Последние тренировки">
          {latest.length === 0 ? (
            <EmptyState text="Тренировок пока нет" />
          ) : (
            <div className="table-scroll">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Дата</th>
                    <th>Название</th>
                    <th>Тип</th>
                    <th className="num">Км</th>
                    <th className="num">Время</th>
                    <th className="num">Темп</th>
                  </tr>
                </thead>
                <tbody>
                  {latest.map((w) => (
                    <tr key={w.slug}>
                      <td className="num">{fmtDayYear(w.date)}</td>
                      <td className="title-cell">
                        <Link href={`/workouts/${w.slug}`}>{w.title}</Link>
                      </td>
                      <td>
                        <KindChip kind={w.kind} />
                      </td>
                      <td className="num">{fmtKm(w.totals.distanceKm)}</td>
                      <td className="num">{fmtDuration(w.totals.movingTimeS)}</td>
                      <td className="num">{fmtPace(paceSPerKm(w.totals.movingTimeS, w.totals.distanceKm))}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>

        <Card title="Флаги травм">
          {injuryTags.size === 0 ? (
            <EmptyState text={`Активных флагов за последние ${INJURY_WINDOW_DAYS} дней нет`} />
          ) : (
            <>
              <div className="wk-chips" style={{ marginBottom: 8 }}>
                <InjuriesChips injuries={[...injuryTags.keys()]} />
              </div>
              <ul className="meta-list" style={{ listStyle: 'none', margin: 0, padding: 0 }}>
                {[...injuryTags.entries()].map(([tag, date]) => (
                  <li key={tag} className="wk-meta">
                    {tag} — последний раз {fmtDayYear(date)}
                  </li>
                ))}
              </ul>
            </>
          )}
          {results.length > 0 ? (
            <div className="board-note" style={{ marginTop: 14 }}>
              Последний результат: {fmtDayYear(results[0].date)} · {results[0].name} — {fmtDuration(results[0].timeS)}
            </div>
          ) : null}
        </Card>
      </div>
    </>
  );
}
