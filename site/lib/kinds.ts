export const KINDS = ['easy', 'long', 'session', 'hills', 'race', 'strength', 'rest'] as const;

export type Kind = (typeof KINDS)[number];

export function kindKey(kind: string | null | undefined): string {
  return kind ?? 'other';
}

export const KIND_LABELS: Record<string, string> = {
  easy: 'Лёгкая',
  long: 'Длинная',
  session: 'Качество',
  hills: 'Горки',
  race: 'Гонка',
  strength: 'Силовая',
  rest: 'Отдых',
  other: 'Прочее',
};

export function kindLabel(kind: string | null | undefined): string {
  return KIND_LABELS[kindKey(kind)] ?? kindKey(kind);
}

export const KIND_COLORS: Record<string, string> = {
  easy: '#2a78d6',
  long: '#eb6834',
  session: '#1baf7a',
  hills: '#eda100',
  race: '#b3221f',
  strength: '#4a3aa7',
  rest: '#898781',
  other: '#898781',
};

export const STATUSES = ['planned', 'done', 'partial', 'moved', 'skipped'] as const;

export const STATUS_LABELS: Record<string, string> = {
  planned: 'Запланировано',
  done: 'Выполнено',
  partial: 'Частично',
  moved: 'Перенесено',
  skipped: 'Пропущено',
};

export function statusLabel(status: string | null | undefined): string {
  const key = status ?? 'planned';
  return STATUS_LABELS[key] ?? key;
}

export const HR_ZONE_ORDER = [
  'recovery',
  'aerobic_endurance',
  'aerobic_power',
  'threshold',
  'anaerobic_endurance',
  'anaerobic_power',
] as const;

export const HR_ZONE_LABELS: Record<string, string> = {
  recovery: 'Z1 Восстановление',
  aerobic_endurance: 'Z2 Аэробная выносливость',
  aerobic_power: 'Z3 Аэробная мощность',
  threshold: 'Z4 Порог',
  anaerobic_endurance: 'Z5 Анаэробная выносливость',
  anaerobic_power: 'Z6 Анаэробная мощность',
};

export const HR_ZONE_RAMP = ['#86b6ef', '#6e9cd5', '#5683ba', '#3d69a0', '#255085', '#0d366b'];

export const PHASES = ['base', 'build', 'peak', 'sharpen', 'race', 'deload'] as const;

export const PHASE_LABELS: Record<string, string> = {
  base: 'База',
  build: 'Развитие',
  peak: 'Пик',
  sharpen: 'Подводка',
  race: 'Старт',
  deload: 'Разгрузка',
};

export function phaseLabel(phase: string | null | undefined): string {
  if (!phase) return '';
  return PHASE_LABELS[phase] ?? phase;
}
