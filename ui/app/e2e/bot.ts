// The screen-only user (L3): deterministic code, no model. It uses only what is on the screen:
// a control it may press is found by role and name, is inside the viewport and is not covered at
// its centre. A click, a key press, typing into a field and one scroll of the page are one action
// each. At every look it notes where its target was: 1 in view, 2 partly in view, 3 out of view.

import type { Locator, Page } from '@playwright/test';

export type Glance = 1 | 2 | 3;

/** A search waits this long after the last key before it asks (pages/Overview.vue: 180 ms). */
export const TYPING_PAUSE_MS = 250;

export interface Metrics {
  'screen_only.actions': number;
  'screen_only.looks': number;
  'screen_only.steers': number;
  'screen_only.time_to_act_steps.max': number;
  'glance_cost.median': number;
  'glance_cost.p95': number;
  'reading_load.max': number;
  'reading_load.median': number;
  'layout_shift.score': number;
}

/** Counts the layout shifts of the page; read with `layoutShift`. */
export function shiftObserver() {
  const w = window as unknown as { __shift: number };
  w.__shift = 0;
  new PerformanceObserver((list) => {
    for (const e of list.getEntries() as unknown as { value: number; hadRecentInput: boolean }[]) {
      if (!e.hadRecentInput) w.__shift += e.value;
    }
  }).observe({ type: 'layout-shift', buffered: true });
}

export async function settle(page: Page): Promise<void> {
  await page.evaluate(
    () =>
      new Promise<void>((done) => {
        void document.fonts.ready.then(() => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(done, 40))));
      }),
  );
}

/** Where `l` is on the screen. */
export async function where(l: Locator): Promise<Glance> {
  if ((await l.count()) === 0) return 3;
  return l.first().evaluate((el): Glance => {
    const r = el.getBoundingClientRect();
    const vw = document.documentElement.clientWidth;
    const vh = window.innerHeight;
    if (r.width === 0 || r.height === 0) return 3;
    const any = r.right > 0 && r.bottom > 0 && r.left < vw && r.top < vh;
    if (!any) return 3;
    const inside = r.left >= -0.5 && r.top >= -0.5 && r.right <= vw + 0.5 && r.bottom <= vh + 0.5;
    if (!inside) return 2;
    const hit = document.elementFromPoint((r.left + r.right) / 2, (r.top + r.bottom) / 2);
    const group = el instanceof SVGElement ? el.parentElement : null;
    const shown = !!hit && (hit === el || el.contains(hit) || (!!group && group.contains(hit)));
    return shown ? 1 : 2;
  });
}

/** The words of the text that is on the screen now. */
export async function visibleWords(page: Page): Promise<number> {
  return page.evaluate(() => {
    const vw = document.documentElement.clientWidth;
    const vh = window.innerHeight;
    const walk = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    let words = 0;
    const range = document.createRange();
    for (let n = walk.nextNode(); n; n = walk.nextNode()) {
      const text = n.textContent?.trim();
      if (!text) continue;
      const parent = n.parentElement;
      if (!parent) continue;
      const style = getComputedStyle(parent);
      if (style.visibility === 'hidden' || style.display === 'none' || Number(style.opacity) === 0) continue;
      range.selectNodeContents(n);
      const r = range.getBoundingClientRect();
      if (r.width === 0 || r.bottom <= 0 || r.top >= vh || r.right <= 0 || r.left >= vw) continue;
      words += text.split(/\s+/).length;
    }
    return words;
  });
}

function median(xs: number[]): number {
  if (!xs.length) return 0;
  const s = xs.slice().sort((a, b) => a - b);
  const m = Math.floor(s.length / 2);
  return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
}

function p95(xs: number[]): number {
  if (!xs.length) return 0;
  const s = xs.slice().sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.ceil(0.95 * s.length) - 1)];
}

export type StateHook = (label: string) => Promise<void>;

export class Bot {
  actions = 0;
  looks = 0;
  steers = 0;
  private sinceAct = 0;
  private toAct: number[] = [];
  private glances: number[] = [];
  private reading: number[] = [];
  log: string[] = [];

  constructor(
    readonly page: Page,
    private readonly onState: StateHook = async () => {},
  ) {}

  /** Looks for `l` and notes where it is. */
  async look(l: Locator, what: string): Promise<Glance> {
    await settle(this.page);
    const g = await where(l);
    this.looks += 1;
    this.sinceAct += 1;
    this.glances.push(g);
    this.reading.push(await visibleWords(this.page));
    this.log.push(`look ${what}: ${g}`);
    return g;
  }

  /** Scrolls until `l` is in view, one action per scroll of at most 80 % of the viewport. */
  async reach(l: Locator, what: string): Promise<void> {
    let g = await this.look(l, what);
    for (let i = 0; g !== 1 && i < 12; i++) {
      if ((await l.count()) === 0) throw new Error(`the screen shows no ${what}`);
      const box = await l.first().evaluate((el) => {
        const r = el.getBoundingClientRect();
        return { top: r.top, bottom: r.bottom, vh: window.innerHeight };
      });
      const step = Math.round(box.vh * 0.8);
      const need = box.bottom > box.vh ? Math.min(step, box.bottom - box.vh + 24) : box.top < 0 ? -Math.min(step, -box.top + 72) : 0;
      if (need === 0) break;
      await this.page.mouse.wheel(0, need);
      this.actions += 1;
      this.log.push(`scroll ${need}`);
      await this.onState(`scroll toward ${what}`);
      g = await this.look(l, what);
    }
    if (g !== 1) throw new Error(`${what} is not fully in view and not covered (glance ${g})`);
  }

  private async act(l: Locator, what: string, fn: () => Promise<void>, cost = 1): Promise<void> {
    await this.reach(l, what);
    this.toAct.push(this.sinceAct);
    this.sinceAct = 0;
    this.actions += cost;
    this.log.push(`act ${what}`);
    await fn();
    await settle(this.page);
    await this.onState(`after ${what}`);
  }

  click(l: Locator, what: string): Promise<void> {
    return this.act(l, what, () => l.first().click());
  }

  /** A click into the field and the typing: two actions. */
  type(l: Locator, text: string, what: string): Promise<void> {
    return this.act(l, what, async () => {
      await l.first().fill(text);
      await this.page.waitForTimeout(TYPING_PAUSE_MS);
    }, 2);
  }

  async layoutShift(): Promise<number> {
    return this.page.evaluate(() => (window as unknown as { __shift: number }).__shift ?? 0);
  }

  async metrics(): Promise<Metrics> {
    return {
      'screen_only.actions': this.actions,
      'screen_only.looks': this.looks,
      'screen_only.steers': this.steers,
      'screen_only.time_to_act_steps.max': Math.max(0, ...this.toAct),
      'glance_cost.median': median(this.glances),
      'glance_cost.p95': p95(this.glances),
      'reading_load.max': Math.max(0, ...this.reading),
      'reading_load.median': median(this.reading),
      'layout_shift.score': Math.round((await this.layoutShift()) * 10_000) / 10_000,
    };
  }
}
