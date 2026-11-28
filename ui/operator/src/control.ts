// What the pages across parts share: the events of the level's channel as the Events page lists
// them, and a prediction as text.

/** One event of the level's channel, as it arrived. */
export interface LoggedEvent extends Record<string, unknown> {
  key: string;
  id: string;
  event: string;
  data: string;
}

/** The simulator's predicted value of a KPI, four significant digits; a dash when none. */
export function predicted(value: number | null): string {
  if (value === null || !Number.isFinite(value)) return 'no prediction of this revision';
  return String(Number(value.toPrecision(4)));
}

/** The rows of a view counted by the value of `field`: `Open 2 · Drained 1`. */
export function byValue(rows: Record<string, unknown>[], field: string | null): string {
  if (!field) return '';
  const counts = new Map<string, number>();
  for (const r of rows) {
    const v = String(r[field] ?? 'none');
    counts.set(v, (counts.get(v) ?? 0) + 1);
  }
  return [...counts.entries()].map(([v, n]) => `${v} ${n}`).join(' · ');
}
