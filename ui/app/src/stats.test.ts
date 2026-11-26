import { describe, expect, it } from 'vitest';
import type { OverviewFiguresRow } from './generated/types.ts';
import { change, figures, sum, window } from './stats.ts';
import { answer, fixtures, FIXTURE_NOW } from './testing/fixtures.ts';

const rows = fixtures.sessions;

function buckets(slice: '24h' | '72h' | '7d' | '30d', now: number): OverviewFiguresRow[] {
  const w = window(slice, now);
  return answer(`/view/views/figures?since=${new Date(w.since).toISOString()}&bucket_seconds=${w.bucketSeconds}`)
    .rows as OverviewFiguresRow[];
}

describe('overview figures', () => {
  it('asks for two slices of whole-hour buckets ending at the next hour', () => {
    const w = window('72h', Date.parse('2026-11-20T10:20:00Z'));
    expect(new Date(w.end).toISOString()).toBe('2026-11-20T11:00:00.000Z');
    expect(w.end - w.since).toBe(2 * 72 * 3_600_000);
    expect(w.bucketSeconds % 3600).toBe(0);
    for (const s of ['24h', '7d', '30d'] as const) expect(window(s, 0).bucketSeconds % 3600).toBe(0);
  });

  it('averages the sums of the slice', () => {
    const f = figures(buckets('72h', FIXTURE_NOW), '72h', FIXTURE_NOW);
    const byId = Object.fromEntries(f.map((x) => [x.id, x]));
    expect(byId.sessions.value).toBe(rows.length);
    const meanCost = rows.reduce((a, r) => a + r.cost_usd, 0) / rows.length;
    expect(byId.cost.value).toBeCloseTo(meanCost, 9);
    expect(byId.cost.note).toMatch(/in total/);
    expect(byId.cache.value).toBeCloseTo(rows.reduce((a, r) => a + r.cache_hit_share, 0) / rows.length, 9);
    expect(byId.sessions.trend).toHaveLength(12);
    expect(byId.sessions.trend.reduce((a, v) => (a ?? 0) + (v ?? 0), 0)).toBe(rows.length);
  });

  it('says no change when the slice before is too thin to compare', () => {
    const f = figures(buckets('72h', FIXTURE_NOW), '72h', FIXTURE_NOW);
    expect(f.every((x) => x.previous === null)).toBe(true);
  });

  it('compares with the slice before when it holds enough sessions', () => {
    const w = window('72h', FIXTURE_NOW);
    const earlier: OverviewFiguresRow = {
      bucket_id: String(w.since),
      start: new Date(w.since).toISOString(),
      bucket_seconds: w.bucketSeconds,
      sessions: 6,
      cost_usd: 6,
      working_seconds: 0,
      waiting_share: 0,
      cache_hit_share: 0,
      tool_calls: 0,
      refused_tool_calls: 0,
    };
    const f = figures([earlier, ...buckets('72h', FIXTURE_NOW)], '72h', FIXTURE_NOW);
    const cost = f.find((x) => x.id === 'cost')!;
    expect(cost.previous).toBe(1);
    expect(change(cost.value, cost.previous)).toBeCloseTo((cost.value! - 1) / 1, 9);
  });

  it('sums every figure of the buckets', () => {
    const s = sum(buckets('7d', FIXTURE_NOW));
    expect(s.sessions).toBe(rows.length);
    expect(s.tool_calls).toBe(rows.reduce((a, r) => a + r.tool_calls, 0));
  });
});
