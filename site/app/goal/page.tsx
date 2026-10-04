import { Markdown } from '@/lib/markdown';
import { Card, EmptyState, PageHeader } from '@/components/ui';
import { loadGoalDoc, loadRaces } from '@/lib/data';
import { fmtDayYear } from '@/lib/dates';
import { fmtDuration } from '@/lib/format';

export const metadata = { title: 'Цель' };

export default function GoalPage() {
  const goalDoc = loadGoalDoc();
  const { results } = loadRaces();

  return (
    <>
      <PageHeader title="Цель" subtitle="5 км быстрее 20:00 — A-гонка 25 апреля 2027" />
      {goalDoc == null || !goalDoc.trim() ? (
        <EmptyState text="Документ цели не найден (data/goals/5k-sub20.md)" />
      ) : (
        <Card>
          <Markdown source={goalDoc} />
        </Card>
      )}

      <Card title="История стартов">
        {results.length === 0 ? (
          <EmptyState text="Результатов пока нет" />
        ) : (
          <ul className="timeline">
            {results.map((r) => (
              <li key={`${r.date}-${r.name}`}>
                <div className="tl-date">{fmtDayYear(r.date)}</div>
                <div className="tl-name">{r.name}</div>
                {r.timeS != null ? <span className="tl-time">{fmtDuration(r.timeS)}</span> : null}
                {r.note ? <p className="tl-note">{r.note}</p> : null}
              </li>
            ))}
          </ul>
        )}
      </Card>
    </>
  );
}
