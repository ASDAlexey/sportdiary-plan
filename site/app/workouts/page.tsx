import WorkoutsExplorer, { type WorkoutListItem } from './WorkoutsExplorer';
import { EmptyState, PageHeader } from '@/components/ui';
import { isCounted, loadWorkouts } from '@/lib/data';
import { pluralRu } from '@/lib/dates';
import { fmtKm, fmtKmUnit, paceSPerKm } from '@/lib/format';

export const metadata = { title: 'Тренировки' };

export default function WorkoutsPage() {
  const workouts = loadWorkouts();
  const items: WorkoutListItem[] = workouts.map((w) => ({
    slug: w.slug,
    date: w.date,
    title: w.title,
    kind: w.kind,
    status: w.status,
    distanceKm: w.totals.distanceKm,
    timeS: w.totals.movingTimeS,
    paceS: paceSPerKm(w.totals.movingTimeS, w.totals.distanceKm),
  }));
  const totalKm = workouts.filter(isCounted).reduce((s, w) => s + (w.totals.distanceKm ?? 0), 0);

  return (
    <>
      <PageHeader
        title="Тренировки"
        subtitle={items.length > 0 ? `${items.length} ${pluralRu(items.length, 'запись', 'записи', 'записей')} · всего ${fmtKmUnit(totalKm)}` : undefined}
      />
      {items.length === 0 ? (
        <EmptyState text="Тренировок пока нет — данные появятся после первого лога" />
      ) : (
        <WorkoutsExplorer items={items} />
      )}
    </>
  );
}
