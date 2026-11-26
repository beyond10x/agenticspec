// The overview's figures: averages over the sessions of a time slice, the same figure for the
// slice before it, and a trend of the figure over the slice. The server sums the figures per
// bucket (the `figures` view); this divides. Pure; tested in stats.test.ts.

import type { OverviewFiguresRow, SessionListRow } from './generated/types.ts';
import { money } from './format.ts';

export type Slice = '24h' | '72h' | '7d' | '30d';

export const SLICES: { id: Slice; label: string; ms: number; buckets: number }[] = [
  { id: '24h', label: '24 h', ms: 24 * 3_600_000, buckets: 12 },
  { id: '72h', label: '72 h', ms: 72 * 3_600_000, buckets: 12 },
  { id: '7d', label: '7 d', ms: 7 * 86_400_000, buckets: 14 },
  { id: '30d', label: '30 d', ms: 30 * 86_400_000, buckets: 15 },
];

export function slice(id: Slice) {
  return SLICES.find((s) => s.id === id) ?? SLICES[1];
}

export const ms = (t: string) => Date.parse(t);

/** Wall time of a session in seconds. */
export function wallSeconds(r: Pick<SessionListRow, 'first_at' | 'last_at'>): number {
  return Math.max(0, (ms(r.last_at) - ms(r.first_at)) / 1000);
}

const HOUR = 3_600_000;

/**
 * What to ask the `figures` view for: two slices that end at the hour after `now`, in buckets of
 * whole hours, the slice before first.
 */
export function window(id: Slice, now: number): { since: number; end: number; bucketSeconds: number } {
  const s = slice(id);
  const end = Math.ceil(now / HOUR) * HOUR;
  return { since: end - 2 * s.ms, end, bucketSeconds: s.ms / s.buckets / 1000 };
}

export interface Figure {
  id: string;
  label: string;
  /** The figure over the slice; null when the slice holds no session. */
  value: number | null;
  /** The same figure over the slice before; null when it cannot stand for it. */
  previous: number | null;
  /** One value per bucket of the slice, oldest first; null for a bucket without a session. */
  trend: (number | null)[];
  /** Whether a higher value is better, for the colour of the change. */
  higherIsBetter: boolean | null;
  kind: 'count' | 'money' | 'duration' | 'share' | 'number';
  note?: string;
}

/** The sums of some buckets. */
export interface Sums {
  sessions: number;
  cost_usd: number;
  working_seconds: number;
  waiting_share: number;
  cache_hit_share: number;
  tool_calls: number;
  refused_tool_calls: number;
}

type Metric = {
  id: string;
  label: string;
  kind: Figure['kind'];
  higherIsBetter: boolean | null;
  of: (s: Sums) => number | null;
  note?: (s: Sums) => string | undefined;
};

const per = (s: Sums, v: number) => (s.sessions ? v / s.sessions : null);

export const METRICS: Metric[] = [
  { id: 'sessions', label: 'Sessions', kind: 'count', higherIsBetter: null, of: (s) => s.sessions },
  {
    id: 'cost',
    label: 'Cost per session',
    kind: 'money',
    higherIsBetter: false,
    of: (s) => per(s, s.cost_usd),
    note: (s) => (s.sessions ? `${money(s.cost_usd)} in total, at list price` : undefined),
  },
  {
    id: 'working',
    label: 'Working time',
    kind: 'duration',
    higherIsBetter: null,
    of: (s) => per(s, s.working_seconds),
    note: () => 'agent time per session',
  },
  {
    id: 'waiting',
    label: 'Waiting share',
    kind: 'share',
    higherIsBetter: false,
    of: (s) => per(s, s.waiting_share),
    note: () => 'of wall time on a human',
  },
  {
    id: 'cache',
    label: 'Cache hits',
    kind: 'share',
    higherIsBetter: true,
    of: (s) => per(s, s.cache_hit_share),
    note: () => 'of input tokens read from cache',
  },
  {
    id: 'tools',
    label: 'Tool calls',
    kind: 'number',
    higherIsBetter: null,
    of: (s) => per(s, s.tool_calls),
    note: (s) => (s.tool_calls ? `per session · ${((s.refused_tool_calls / s.tool_calls) * 100).toFixed(1)}% refused` : undefined),
  },
];

export function sum(rows: Pick<OverviewFiguresRow, keyof Sums>[]): Sums {
  const out: Sums = { sessions: 0, cost_usd: 0, working_seconds: 0, waiting_share: 0, cache_hit_share: 0, tool_calls: 0, refused_tool_calls: 0 };
  for (const r of rows) {
    for (const k of Object.keys(out) as (keyof Sums)[]) out[k] += r[k];
  }
  return out;
}

/** The figures of the slice that ends at `now`, from the buckets of [`window`]. */
export function figures(rows: OverviewFiguresRow[], id: Slice, now: number): Figure[] {
  const s = slice(id);
  const w = window(id, now);
  const mid = w.since + s.ms;
  const width = w.bucketSeconds * 1000;
  const current = rows.filter((r) => ms(r.start) >= mid && ms(r.start) < w.end);
  const before = rows.filter((r) => ms(r.start) >= w.since && ms(r.start) < mid);
  const buckets: OverviewFiguresRow[][] = Array.from({ length: s.buckets }, () => []);
  for (const r of current) {
    const i = Math.min(s.buckets - 1, Math.max(0, Math.floor((ms(r.start) - mid) / width)));
    buckets[i].push(r);
  }
  const cur = sum(current);
  const prev = sum(before);
  // The store keeps the sessions of its sync window only, so the slice before is often cut off.
  // A change is said only when that slice holds enough sessions to stand for it.
  const comparable = prev.sessions >= Math.max(5, Math.ceil(cur.sessions / 4));
  return METRICS.map((m) => ({
    id: m.id,
    label: m.label,
    kind: m.kind,
    higherIsBetter: m.higherIsBetter,
    value: cur.sessions || m.kind === 'count' ? m.of(cur) : null,
    previous: comparable ? m.of(prev) : null,
    trend: buckets.map((b) => {
      const x = sum(b);
      return x.sessions || m.kind === 'count' ? m.of(x) : null;
    }),
    note: m.note?.(cur),
  }));
}

/** The change from `previous` to `value` as a signed share; null when it cannot be said. */
export function change(value: number | null, previous: number | null): number | null {
  if (value === null || previous === null || previous === 0) return null;
  return (value - previous) / Math.abs(previous);
}
