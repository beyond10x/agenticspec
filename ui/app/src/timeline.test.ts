import { describe, expect, it } from 'vitest';
import { scale } from './timeline.ts';

const MIN = 60_000;

describe('compressed time scale', () => {
  it('keeps one stretch linear', () => {
    const s = scale(Array.from({ length: 11 }, (_, i) => i * MIN), 1000);
    expect(s.breaks).toHaveLength(0);
    expect(s.x(0)).toBe(0);
    expect(s.x(5 * MIN)).toBeCloseTo(500, 6);
    expect(s.x(10 * MIN)).toBeCloseTo(1000, 6);
  });

  it('folds an idle gap into a break of fixed width', () => {
    const t = [0, 4 * MIN, 8 * MIN, 8 * MIN + 600 * MIN, 8 * MIN + 604 * MIN];
    const s = scale(t, 1000, { breakPx: 40 });
    expect(s.stretches).toHaveLength(2);
    expect(s.breaks).toHaveLength(1);
    expect(s.breaks[0].w).toBe(40);
    // The 10-hour gap takes 40 px; the two active stretches share the rest by duration.
    const [a, b] = s.stretches;
    expect(a.w / b.w).toBeCloseTo((8 + 0) / 4, 0);
    expect(s.x(t[t.length - 1])).toBeCloseTo(1000, 6);
    expect(s.x(t[2])).toBeLessThan(s.x(t[3]));
  });

  it('never puts more breaks than fit, so the axis ends at the width', () => {
    const t: number[] = [];
    for (let i = 0; i < 200; i++) t.push(i * 30 * MIN, i * 30 * MIN + MIN);
    const s = scale(t, 600, { breakPx: 36, breakShare: 0.3 });
    expect(s.breaks.length).toBeLessThanOrEqual(5);
    expect(s.x(t[t.length - 1])).toBeCloseTo(600, 6);
    const ends = s.stretches.map((x) => x.x + x.w);
    expect(Math.max(...ends)).toBeLessThanOrEqual(600 + 1e-6);
  });

  it('is monotonic over the whole session', () => {
    const t = [0, MIN, 90 * MIN, 91 * MIN, 400 * MIN, 401 * MIN, 402 * MIN];
    const s = scale(t, 800);
    let prev = -1;
    for (let x = 0; x <= 402 * MIN; x += MIN) {
      const px = s.x(x);
      expect(px).toBeGreaterThanOrEqual(prev);
      prev = px;
    }
  });

  it('labels ticks without overlap', () => {
    const s = scale([0, 3 * 3600_000], 900, { tickPx: 72 });
    for (let i = 1; i < s.ticks.length; i++) expect(s.ticks[i].x - s.ticks[i - 1].x).toBeGreaterThanOrEqual(72 * 0.8);
  });
});
