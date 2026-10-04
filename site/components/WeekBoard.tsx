import Link from 'next/link';
import { KindChip, StatusChip } from '@/components/Chips';
import type { WeekDay } from '@/lib/week';
import { dayName, fmtDay } from '@/lib/dates';
import { fmtKm, fmtPace } from '@/lib/format';

export default function WeekBoard({ days, todayIso }: { days: WeekDay[]; todayIso: string }) {
  if (days.length === 0) {
    return <div className="empty">Неделя не определена</div>;
  }
  return (
    <div className="wk">
      {days.map((d) => {
        const isToday = d.date === todayIso;
        return (
          <div key={d.date}>
            <div className={`wk-day ${isToday ? 'today' : ''}`}>
              <span>{isToday ? `Сегодня · ${dayName(d.date)}` : dayName(d.date)}</span>
              <span className="wk-date">{fmtDay(d.date)}</span>
            </div>
            {d.rows.length === 0 ? (
              <div className="wk-empty-day">—</div>
            ) : (
              d.rows.map((r) => (
                <div key={r.key} className="wk-row">
                  <div className="wk-main">
                    <div className="wk-title">
                      {r.href ? <Link href={r.href}>{r.title}</Link> : r.title}
                    </div>
                    <div className="wk-chips">
                      <KindChip kind={r.kind} />
                      <StatusChip status={r.status} />
                    </div>
                    {r.meta ? <div className="wk-meta">{r.meta}</div> : null}
                  </div>
                  <div className="wk-num">{r.distanceKm != null ? `${fmtKm(r.distanceKm)} км` : '—'}</div>
                  <div className="wk-num">{r.paceS != null ? `${fmtPace(r.paceS)} /км` : '—'}</div>
                </div>
              ))
            )}
          </div>
        );
      })}
    </div>
  );
}
