import { Card, EmptyState, PageHeader } from '@/components/ui';
import { loadRaces } from '@/lib/data';
import { daysBetween, fmtDayYear, fmtDaysIn, todayISO } from '@/lib/dates';

export const metadata = { title: 'Гонки' };

const PRIORITY_LABELS: Record<string, string> = {
  A: 'A-гонка',
  B: 'B-гонка',
  benchmark: 'Бенчмарк',
};

function PriorityChip({ priority }: { priority: string | null }) {
  if (!priority) return <span>—</span>;
  const label = PRIORITY_LABELS[priority] ?? priority;
  if (priority === 'A') return <span className="chip chip-race">{label}</span>;
  return <span className="chip">{label}</span>;
}

export default function RacesPage() {
  const today = todayISO();
  const { races } = loadRaces();
  const upcoming = races.filter((r) => r.date >= today);
  const past = races.filter((r) => r.date < today).reverse();

  return (
    <>
      <PageHeader title="Гонки" subtitle={`Календарь стартов · сегодня ${fmtDayYear(today)}`} />

      <Card title="Предстоящие старты">
        {upcoming.length === 0 ? (
          <EmptyState text="Запланированных стартов нет" />
        ) : (
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Дата</th>
                  <th>Старт</th>
                  <th>Тип</th>
                  <th>Приоритет</th>
                  <th>Цель</th>
                  <th>Осталось</th>
                </tr>
              </thead>
              <tbody>
                {upcoming.map((r) => (
                  <tr key={`${r.date}-${r.name}`}>
                    <td className="num">{fmtDayYear(r.date)}</td>
                    <td className="title-cell">{r.name}</td>
                    <td>{r.kind ?? '—'}</td>
                    <td>
                      <PriorityChip priority={r.priority} />
                    </td>
                    <td>{r.target ?? '—'}</td>
                    <td>{fmtDaysIn(daysBetween(today, r.date))}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {past.length > 0 ? (
        <Card title="Прошедшие">
          <div className="table-scroll">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Дата</th>
                  <th>Старт</th>
                  <th>Тип</th>
                  <th>Приоритет</th>
                  <th>Цель</th>
                </tr>
              </thead>
              <tbody>
                {past.map((r) => (
                  <tr key={`${r.date}-${r.name}`}>
                    <td className="num">{fmtDayYear(r.date)}</td>
                    <td className="title-cell">{r.name}</td>
                    <td>{r.kind ?? '—'}</td>
                    <td>
                      <PriorityChip priority={r.priority} />
                    </td>
                    <td>{r.target ?? '—'}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      ) : null}
    </>
  );
}
