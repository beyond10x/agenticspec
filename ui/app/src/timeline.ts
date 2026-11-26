// The time axis of a session: active stretches keep their proportion, idle gaps over a threshold
// shrink to a fixed break, so a session that ran 15 minutes in a 15-hour window still fills the
// width. Pure; tested in timeline.test.ts.

import { clock, day, shortDuration } from './format.ts';

export interface Stretch {
  from: number;
  to: number;
  x: number;
  w: number;
}

export interface Break {
  from: number;
  to: number;
  x: number;
  w: number;
}

export interface Tick {
  t: number;
  x: number;
  label: string;
}

export interface Scale {
  width: number;
  stretches: Stretch[];
  breaks: Break[];
  x(t: number): number;
  ticks: Tick[];
}

export interface ScaleOptions {
  /** An idle gap longer than this becomes a break. */
  gapMs?: number;
  /** The width of a break. */
  breakPx?: number;
  /** The least width of a stretch, so a burst of one instant stays visible. */
  minPx?: number;
  /** The least distance between two tick labels. */
  tickPx?: number;
  /** The most of the width breaks may take together. */
  breakShare?: number;
}

const STEPS = [
  1_000, 5_000, 15_000, 30_000, 60_000, 120_000, 300_000, 600_000, 900_000, 1_800_000, 3_600_000, 7_200_000,
  10_800_000, 21_600_000, 43_200_000, 86_400_000,
];

export function scale(times: number[], width: number, o: ScaleOptions = {}): Scale {
  const gapMs = o.gapMs ?? 5 * 60_000;
  const breakPx = o.breakPx ?? 36;
  const minPx = o.minPx ?? 6;
  const tickPx = o.tickPx ?? 72;
  const sorted = times.filter((t) => Number.isFinite(t)).sort((a, b) => a - b);
  if (sorted.length === 0) {
    return { width, stretches: [], breaks: [], x: () => 0, ticks: [] };
  }
  // At most a third of the width goes to breaks: the longest gaps fold, shorter ones stay time.
  const maxBreaks = Math.max(0, Math.floor((width * (o.breakShare ?? 0.3)) / breakPx));
  const gaps: number[] = [];
  for (let i = 1; i < sorted.length; i++) {
    const g = sorted[i] - sorted[i - 1];
    if (g > gapMs) gaps.push(g);
  }
  gaps.sort((a, b) => b - a);
  const threshold = gaps.length > maxBreaks ? (maxBreaks === 0 ? Infinity : gaps[maxBreaks - 1]) : gapMs;
  const fold = (g: number) => (gaps.length > maxBreaks ? g >= threshold : g > gapMs);
  const groups: { from: number; to: number }[] = [];
  let folded = 0;
  for (const t of sorted) {
    const last = groups[groups.length - 1];
    if (last && (!fold(t - last.to) || folded >= maxBreaks)) last.to = t;
    else {
      if (last) folded++;
      groups.push({ from: t, to: t });
    }
  }
  const nBreaks = groups.length - 1;
  const free = Math.max(0, width - nBreaks * breakPx - groups.length * minPx);
  const total = groups.reduce((a, g) => a + (g.to - g.from), 0);
  const stretches: Stretch[] = [];
  const breaks: Break[] = [];
  let x = 0;
  groups.forEach((g, i) => {
    const share = total > 0 ? (g.to - g.from) / total : 1 / groups.length;
    const w = minPx + share * free;
    stretches.push({ from: g.from, to: g.to, x, w });
    x += w;
    if (i < groups.length - 1) {
      breaks.push({ from: g.to, to: groups[i + 1].from, x, w: breakPx });
      x += breakPx;
    }
  });

  const at = (t: number): number => {
    if (t <= stretches[0].from) return stretches[0].x;
    for (let i = 0; i < stretches.length; i++) {
      const s = stretches[i];
      if (t <= s.to) {
        const d = s.to - s.from;
        return d > 0 ? s.x + ((t - s.from) / d) * s.w : s.x + s.w / 2;
      }
      const b = breaks[i];
      if (b && t < b.to) return b.x + ((t - b.from) / (b.to - b.from)) * b.w;
    }
    const last = stretches[stretches.length - 1];
    return last.x + last.w;
  };

  const ticks: Tick[] = [];
  let lastX = -Infinity;
  for (const s of stretches) {
    const d = s.to - s.from;
    const perMs = d > 0 ? s.w / d : 0;
    const step = perMs > 0 ? STEPS.find((st) => st * perMs >= tickPx) : undefined;
    if (!step) {
      // Too narrow for its own ticks: one label at its start when there is room.
      const px = s.x;
      if (px - lastX >= tickPx * 0.8) {
        ticks.push({ t: s.from, x: px, label: clock(s.from) });
        lastX = px;
      }
      continue;
    }
    const off = new Date(s.from).getTimezoneOffset() * 60_000;
    let t = Math.ceil((s.from - off) / step) * step + off;
    for (; t <= s.to; t += step) {
      const px = at(t);
      if (px - lastX < tickPx * 0.8) continue;
      const d = new Date(t);
      const midnight = d.getHours() === 0 && d.getMinutes() === 0 && step >= 3_600_000;
      const label = midnight ? day(t) : step < 60_000 ? `${clock(t)}:${String(d.getSeconds()).padStart(2, '0')}` : clock(t);
      ticks.push({ t, x: px, label });
      lastX = px;
    }
  }
  return { width, stretches, breaks, x: at, ticks };
}

export function breakLabel(b: Break): string {
  return shortDuration(b.to - b.from);
}
