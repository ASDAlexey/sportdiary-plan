import WeekBoard from '@/components/WeekBoard';
import { PhaseChip } from '@/components/Chips';
import { Card, EmptyState, PageHeader } from '@/components/ui';
import { loadPlan, loadWorkouts } from '@/lib/data';
import { fmtDayYear, isoWeekIdOf, todayISO, weekIdAtOffset, weekRangeLabel } from '@/lib/dates';
import { buildWeekModel, weekModelStats } from '@/lib/week';

export const metadata = { title: 'План' };

export default function PlanPage() {
  const today = todayISO();
  const currentWeek = isoWeekIdOf(today);
  const nextWeek = weekIdAtOffset(currentWeek, 1);
  const workouts = loadWorkouts();

  const boards = [currentWeek, nextWeek].map((weekId) => {
    const plan = loadPlan(weekId);
    const days = buildWeekModel(plan, workouts, weekId);
    return { weekId, plan, days, stats: weekModelStats(days) };
  });

  const anyPlan = boards.some((b) => b.plan != null);

  return (
    <>
      <PageHeader title="План" subtitle={`Текущая неделя ${currentWeek} и следующая · сегодня ${fmtDayYear(today)}`} />
      {!anyPlan ? (
        <EmptyState text="Файлов плана пока нет — добавьте data/plan/<неделя>.yaml" />
      ) : (
        boards.map((b, idx) => (
          <Card
            key={b.weekId}
            title={
              <>
                Неделя {b.weekId} {idx === 0 ? <span className="sub">· текущая</span> : <span className="sub">· следующая</span>}
              </>
            }
            actions={
              <span className="sub">
                {weekRangeLabel(b.weekId)} {b.plan?.phase ? <PhaseChip phase={b.plan.phase} /> : null}
              </span>
            }
          >
            {b.plan == null ? (
              <EmptyState text={`Нет файла плана ${b.weekId} (data/plan/${b.weekId}.yaml)`} />
            ) : (
              <>
                <div className="card-actions" style={{ marginBottom: 10 }}>
                  Выполнено {b.stats.done} из {b.stats.planned} · {b.stats.km.toFixed(1)} км
                </div>
                <WeekBoard days={b.days} todayIso={today} />
                {b.plan.notes ? (
                  <div className="board-note">
                    <strong>Заметки недели.</strong> {b.plan.notes}
                  </div>
                ) : null}
              </>
            )}
          </Card>
        ))
      )}
    </>
  );
}
