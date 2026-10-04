import Link from 'next/link';
import { notFound } from 'next/navigation';
import HrZonesBars from '@/components/HrZonesBars';
import { InjuriesChips, KindChip, StatusChip } from '@/components/Chips';
import { Card, PageHeader } from '@/components/ui';
import { loadAthlete, loadWorkouts, type Segment } from '@/lib/data';
import { dayName, fmtDayYear } from '@/lib/dates';
import { fmtDuration, fmtInt, fmtKm, fmtPace, paceSPerKm } from '@/lib/format';

export const dynamicParams = false;

export function generateStaticParams() {
  const params = loadWorkouts().map((w) => ({ slug: w.slug }));
  if (params.length > 0) return params;
  return [{ slug: 'none' }];
}

function segmentPace(s: Segment): number | null {
  return s.paceSPerKm ?? paceSPerKm(s.timeS, s.distanceKm);
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const workout = loadWorkouts().find((w) => w.slug === slug);
  return { title: workout ? workout.title : 'Тренировка' };
}

export default async function WorkoutPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const workout = loadWorkouts().find((w) => w.slug === slug);
  if (!workout) notFound();

  const athlete = loadAthlete();
  const shoe = athlete?.shoes.find((s) => s.id === workout.shoes) ?? null;
  const shoeLabel = shoe ? [shoe.brand, shoe.model].filter(Boolean).join(' ') : workout.shoes;
  const pace = paceSPerKm(workout.totals.movingTimeS, workout.totals.distanceKm);

  return (
    <>
      <Link href="/workouts" className="back-link">
        ← Все тренировки
      </Link>
      <PageHeader
        title={workout.title}
        subtitle={
          <>
            {dayName(workout.date)}, {fmtDayYear(workout.date)}
            {workout.source ? ` · источник: ${workout.source}` : ''}
          </>
        }
      />

      <div className="wk-chips" style={{ marginTop: -8, marginBottom: 16 }}>
        <KindChip kind={workout.kind} />
        <StatusChip status={workout.status} />
        <InjuriesChips injuries={workout.injuries} />
      </div>

      <div className="totals-grid">
        <div className="stat-tile">
          <div className="stat-label">Дистанция</div>
          <div className="stat-value">{fmtKm(workout.totals.distanceKm)}</div>
        </div>
        <div className="stat-tile">
          <div className="stat-label">Время в движении</div>
          <div className="stat-value">{fmtDuration(workout.totals.movingTimeS)}</div>
        </div>
        <div className="stat-tile">
          <div className="stat-label">Темп</div>
          <div className="stat-value">{pace != null ? `${fmtPace(pace)} /км` : '—'}</div>
        </div>
        <div className="stat-tile">
          <div className="stat-label">Набор высоты</div>
          <div className="stat-value">{workout.totals.elevationM != null ? `${fmtInt(workout.totals.elevationM)} м` : '—'}</div>
        </div>
        <div className="stat-tile">
          <div className="stat-label">Средний пульс</div>
          <div className="stat-value">{workout.totals.avgHr != null ? `${fmtInt(workout.totals.avgHr)} уд/мин` : '—'}</div>
        </div>
        <div className="stat-tile">
          <div className="stat-label">Максимальный пульс</div>
          <div className="stat-value">{workout.totals.maxHr != null ? `${fmtInt(workout.totals.maxHr)} уд/мин` : '—'}</div>
        </div>
      </div>

      {workout.structure.length > 0 ? (
        <Card title="Структура">
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Сегмент</th>
                  <th className="num">Повторы</th>
                  <th className="num">Км</th>
                  <th className="num">Время</th>
                  <th className="num">Темп</th>
                  <th>Отдых / заметка</th>
                </tr>
              </thead>
              <tbody>
                {workout.structure.map((s, i) => {
                  const sp = segmentPace(s);
                  return (
                    <tr key={i}>
                      <td>{s.name ?? `Сегмент ${i + 1}`}</td>
                      <td className="num">{s.repeat != null ? `×${s.repeat}` : '—'}</td>
                      <td className="num">{s.distanceKm != null ? fmtKm(s.distanceKm) : '—'}</td>
                      <td className="num">{s.timeS != null ? fmtDuration(s.timeS) : '—'}</td>
                      <td className="num">{sp != null ? `${fmtPace(sp)} /км` : '—'}</td>
                      <td>{[s.rest, s.note].filter(Boolean).join(' · ') || '—'}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </Card>
      ) : null}

      {workout.hrZonesS && workout.hrZonesS.some((v) => v > 0) ? (
        <Card title="Зоны пульса">
          <HrZonesBars seconds={workout.hrZonesS} />
        </Card>
      ) : null}

      {workout.notes ? (
        <Card title="Заметки">
          <p style={{ margin: 0, whiteSpace: 'pre-line' }}>{workout.notes}</p>
        </Card>
      ) : null}

      <dl className="meta-list">
        <div>
          <dt>RPE</dt>
          <dd>{workout.rpe != null ? fmtInt(workout.rpe) : '—'}</dd>
        </div>
        <div>
          <dt>Кроссовки</dt>
          <dd>{shoeLabel ?? '—'}</dd>
        </div>
        <div>
          <dt>Ссылка на план</dt>
          <dd>{workout.planRef ? <Link href="/plan">{workout.planRef}</Link> : '—'}</dd>
        </div>
      </dl>
    </>
  );
}
