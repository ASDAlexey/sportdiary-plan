import { KIND_COLORS, kindKey, kindLabel, statusLabel } from '@/lib/kinds';

export function KindChip({ kind }: { kind: string | null | undefined }) {
  const key = kindKey(kind);
  const color = KIND_COLORS[key] ?? KIND_COLORS.other;
  if (key === 'race') {
    return <span className="chip chip-race">{kindLabel(kind)}</span>;
  }
  return (
    <span className="chip">
      <span className="chip-dot" style={{ background: color }} />
      {kindLabel(kind)}
    </span>
  );
}

const STATUS_CLASS: Record<string, string> = {
  done: 'st-done',
  planned: 'st-planned',
  partial: 'st-partial',
  moved: 'st-moved',
  skipped: 'st-skipped',
};

export function StatusChip({ status }: { status: string | null | undefined }) {
  const key = status ?? 'planned';
  return <span className={`chip chip-status ${STATUS_CLASS[key] ?? 'st-planned'}`}>{statusLabel(status)}</span>;
}

export function PhaseChip({ phase }: { phase: string | null | undefined }) {
  if (!phase) return null;
  return <span className="chip chip-phase">{phase}</span>;
}

export function InjuriesChips({ injuries }: { injuries: string[] }) {
  if (injuries.length === 0) return null;
  return (
    <>
      {injuries.map((tag) => (
        <span key={tag} className="chip chip-injury">
          {tag}
        </span>
      ))}
    </>
  );
}
