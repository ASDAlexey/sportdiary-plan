'use client';

import Link from 'next/link';
import { useMemo, useState } from 'react';
import { KindChip, StatusChip } from '@/components/Chips';
import { fmtMonthYear } from '@/lib/dates';
import { fmtDuration, fmtKm, fmtPace } from '@/lib/format';
import { KINDS, kindLabel } from '@/lib/kinds';

export interface WorkoutListItem {
  slug: string;
  date: string;
  title: string;
  kind: string | null;
  status: string | null;
  distanceKm: number | null;
  timeS: number | null;
  paceS: number | null;
}

export default function WorkoutsExplorer({ items }: { items: WorkoutListItem[] }) {
  const [selected, setSelected] = useState<string>('all');

  const kindCounts = useMemo(() => {
    const counts = new Map<string, number>();
    for (const it of items) {
      const key = it.kind ?? 'other';
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return [...counts.entries()].sort((a, b) => {
      const ia = KINDS.indexOf(a[0] as (typeof KINDS)[number]);
      const ib = KINDS.indexOf(b[0] as (typeof KINDS)[number]);
      return (ia === -1 ? 99 : ia) - (ib === -1 ? 99 : ib);
    });
  }, [items]);

  const filtered = selected === 'all' ? items : items.filter((it) => (it.kind ?? 'other') === selected);

  const months = useMemo(() => {
    const map = new Map<string, WorkoutListItem[]>();
    for (const it of filtered) {
      const m = it.date.slice(0, 7);
      const list = map.get(m) ?? [];
      list.push(it);
      map.set(m, list);
    }
    return [...map.entries()].sort((a, b) => b[0].localeCompare(a[0]));
  }, [filtered]);

  return (
    <div>
      <div className="filter-row" role="group" aria-label="Фильтр по типу">
        <button type="button" className={selected === 'all' ? 'filter-btn active' : 'filter-btn'} onClick={() => setSelected('all')}>
          Все · {items.length}
        </button>
        {kindCounts.map(([kind, count]) => (
          <button
            key={kind}
            type="button"
            className={selected === kind ? 'filter-btn active' : 'filter-btn'}
            onClick={() => setSelected(kind)}
          >
            {kindLabel(kind)} · {count}
          </button>
        ))}
      </div>

      {months.length === 0 ? (
        <div className="empty">Нет тренировок выбранного типа</div>
      ) : (
        months.map(([month, list]) => (
          <section key={month}>
            <h2 className="month-head">
              {fmtMonthYear(month)} · {list.length}
            </h2>
            <div className="table-scroll">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Дата</th>
                    <th>Название</th>
                    <th>Тип</th>
                    <th>Статус</th>
                    <th className="num">Км</th>
                    <th className="num">Время</th>
                    <th className="num">Темп</th>
                  </tr>
                </thead>
                <tbody>
                  {list.map((w) => (
                    <tr key={w.slug}>
                      <td className="num">{w.date}</td>
                      <td className="title-cell">
                        <Link href={`/workouts/${w.slug}`}>{w.title}</Link>
                      </td>
                      <td>
                        <KindChip kind={w.kind} />
                      </td>
                      <td>
                        <StatusChip status={w.status} />
                      </td>
                      <td className="num">{fmtKm(w.distanceKm)}</td>
                      <td className="num">{fmtDuration(w.timeS)}</td>
                      <td className="num">{w.paceS != null ? `${fmtPace(w.paceS)} /км` : '—'}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>
        ))
      )}
    </div>
  );
}
