import { fmtKm } from '@/lib/format';
import styles from './chart.module.css';

export interface VolumeWeek {
  weekId: string;
  km: number;
}

function niceMax(max: number): number {
  const steps = [10, 20, 30, 40, 50, 60, 80, 100, 120, 150, 200];
  for (const s of steps) {
    if (max <= s) return s;
  }
  return Math.ceil(max / 50) * 50;
}

export default function WeeklyVolumeChart({ weeks, currentWeekId }: { weeks: VolumeWeek[]; currentWeekId: string }) {
  const maxKm = Math.max(0, ...weeks.map((w) => w.km));
  const max = niceMax(maxKm);
  const mid = Math.round(max / 2);
  const peak = weeks.reduce((acc, w) => (w.km > acc.km ? w : acc), weeks[0] ?? { weekId: '', km: 0 });
  const total = weeks.reduce((sum, w) => sum + w.km, 0);

  return (
    <figure className={styles.chart} aria-label={`Недельный объём за ${weeks.length} недель, максимум ${fmtKm(maxKm)} км`}>
      <div className={styles.plotArea}>
        <div className={styles.yAxis} aria-hidden="true">
          <span style={{ bottom: '100%' }}>{max}</span>
          <span style={{ bottom: '50%' }}>{mid}</span>
          <span style={{ bottom: '0%' }}>0</span>
        </div>
        <div className={styles.grid} aria-hidden="true">
          <span className={styles.gridTop} />
          <span className={styles.gridMid} />
          <span className={styles.gridBase} />
        </div>
        <div className={styles.cols}>
          {weeks.map((w) => {
            const isCurrent = w.weekId === currentWeekId;
            const isPeak = peak.weekId === w.weekId && w.km > 0;
            const h = max > 0 ? Math.round((w.km / max) * 100) : 0;
            return (
              <div
                key={w.weekId}
                className={`${styles.col} ${isCurrent ? styles.colCurrent : ''}`}
                tabIndex={0}
                role="img"
                aria-label={`${w.weekId}: ${fmtKm(w.km)} км`}
              >
                <div className={styles.barWrap} style={{ height: `${h}%` }}>
                  {(isCurrent || isPeak) && w.km > 0 ? (
                    <span className={styles.barLabel}>{fmtKm(w.km)}</span>
                  ) : null}
                  <div className={`${styles.bar} ${isCurrent ? styles.barCurrent : ''}`} style={{ minHeight: w.km > 0 ? 3 : 0 }} />
                </div>
                <span className={styles.tip} aria-hidden="true">
                  <b>{fmtKm(w.km)} км</b>
                  {w.weekId}
                </span>
                <span className={styles.xLab}>{w.weekId.slice(5)}</span>
              </div>
            );
          })}
        </div>
      </div>
      <figcaption className={styles.caption}>
        Недели (ISO), последняя — текущая. Всего за период: {fmtKm(total)} км.
      </figcaption>
      <details className={styles.details}>
        <summary>Показать данные таблицей</summary>
        <div className="table-scroll">
          <table className="data-table">
            <thead>
              <tr>
                <th>Неделя</th>
                <th className="num">Объём, км</th>
              </tr>
            </thead>
            <tbody>
              {weeks.map((w) => (
                <tr key={w.weekId}>
                  <td>{w.weekId}</td>
                  <td className="num">{fmtKm(w.km)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </details>
    </figure>
  );
}
