import { fmtDuration, fmtPct } from '@/lib/format';
import { HR_ZONE_LABELS, HR_ZONE_ORDER, HR_ZONE_RAMP } from '@/lib/kinds';
import styles from './chart.module.css';

export default function HrZonesBars({ seconds }: { seconds: number[] }) {
  const zones = HR_ZONE_ORDER.slice(0, Math.max(1, seconds.length));
  const values = zones.map((_, i) => seconds[i] ?? 0);
  const total = values.reduce((a, b) => a + b, 0);
  const maxVal = Math.max(...values, 1);

  if (total <= 0) {
    return <div className="empty">Нет данных о зонах пульса</div>;
  }

  return (
    <div>
      <div className={styles.zoneList}>
        {zones.map((z, i) => {
          const pct = total > 0 ? (values[i] / total) * 100 : 0;
          return (
            <div key={z} className={styles.zoneRow}>
              <span className={styles.zoneLabel}>{HR_ZONE_LABELS[z] ?? z}</span>
              <span className={styles.zoneTrack}>
                <span
                  className={styles.zoneBar}
                  style={{ width: `${Math.max((values[i] / maxVal) * 100, values[i] > 0 ? 2 : 0)}%`, background: HR_ZONE_RAMP[i] ?? HR_ZONE_RAMP[HR_ZONE_RAMP.length - 1] }}
                />
              </span>
              <span className={styles.zoneValue}>
                {fmtDuration(values[i])} · {fmtPct(pct)}
              </span>
            </div>
          );
        })}
      </div>
      <p className={styles.caption}>Всего в движении по зонам: {fmtDuration(total)}</p>
    </div>
  );
}
