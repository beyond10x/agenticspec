// The screen-only user (L3) of the docs site: deterministic code, no model. It uses only what is on
// the screen: a control it may press is found by role and name, is inside the viewport and is not
// covered at its centre. A click, a key press, one scroll of a page or a box, and opening a menu are
// one action each; typing into a field and choosing from a list are two. At every look it notes
// where its target was: 1 in view, 2 partly in view or covered, 3 out of view.

import type {Locator, Page} from '@playwright/test';
import type {Step} from './site';

export type Glance = 1 | 2 | 3;

export interface Metrics {
  'screen_only.actions': number;
  'screen_only.looks': number;
  'screen_only.steers': number;
  'screen_only.time_to_act_steps.max': number | null;
  'glance_cost.median': number | null;
  'glance_cost.p95': number | null;
  'reading_load.max': number | null;
  'reading_load.median': number | null;
  'layout_shift.score': number;
}

/** Counts the layout shifts of the page; read with `layoutShift`. */
export function shiftObserver(): void {
  const w = window as unknown as {__shift: number};
  w.__shift = 0;
  new PerformanceObserver((list) => {
    for (const e of list.getEntries() as unknown as {value: number; hadRecentInput: boolean}[]) {
      if (!e.hadRecentInput) w.__shift += e.value;
    }
  }).observe({type: 'layout-shift', buffered: true});
}

export async function settle(page: Page): Promise<void> {
  await page.waitForLoadState('load');
  // A smooth scroll (PageDown, a jump to an anchor) may still move on a loaded machine: wait until
  // the scroll position holds for three frames, at most 1 s, so two looks at one moment agree.
  await page.evaluate(
    () =>
      new Promise<void>((done) => {
        void document.fonts.ready.then(() => {
          const t0 = performance.now();
          let last = `${scrollX},${scrollY}`;
          let still = 0;
          const tick = () => {
            const now = `${scrollX},${scrollY}`;
            still = now === last ? still + 1 : 0;
            last = now;
            if (still >= 3 || performance.now() - t0 > 1000) setTimeout(done, 60);
            else requestAnimationFrame(tick);
          };
          requestAnimationFrame(tick);
        });
      }),
  );
}

/**
 * The box of `el` a reader sees, in the page: its own box; for a control (a link, a button) whose
 * box runs past a box that clips it sideways, such as a link of the menu drawer wider than the
 * drawer, the box of its text, which is what is on the screen and what a reader clicks (S-02, T3).
 * Runs in the page; passed to `evaluate` as the source of a function.
 */
const SEEN_BOX = `(el) => {
  const r = el.getBoundingClientRect();
  if (!el.matches('a, button, [role="button"], [role="link"]')) return r;
  let clipped = false;
  for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
    if (getComputedStyle(p).overflowX === 'visible') continue;
    const b = p.getBoundingClientRect();
    if (r.left < b.left - 0.5 || r.right > b.right + 0.5) clipped = true;
  }
  if (!clipped) return r;
  const range = document.createRange();
  range.selectNodeContents(el);
  const t = range.getBoundingClientRect();
  return t.width > 0 && t.height > 0 ? t : r;
}`;

/**
 * Where `l` is on the screen. A target taller than the space under the navbar is in view when its
 * top is (S-02, T4): it can never be wholly on the screen, and its top is where a reader starts.
 */
export async function where(l: Locator): Promise<Glance> {
  if ((await l.count()) === 0) return 3;
  return l.first().evaluate((el, seenBox): Glance => {
    const seen = new Function(`return ${seenBox}`)() as (e: Element) => DOMRect;
    const r = seen(el);
    const vw = document.documentElement.clientWidth;
    const vh = window.innerHeight;
    if (r.width === 0 || r.height === 0) return 3;
    const any = r.right > 0 && r.bottom > 0 && r.left < vw && r.top < vh;
    if (!any) return 3;
    const bar = Math.max(0, document.querySelector('.navbar')?.getBoundingClientRect().bottom ?? 0);
    const tall = r.height > vh - bar;
    const insideY = tall ? r.top >= bar - 0.5 && r.top < vh : r.top >= -0.5 && r.bottom <= vh + 0.5;
    const inside = r.left >= -0.5 && r.right <= vw + 0.5 && insideY;
    if (!inside) return 2;
    // Clipped by a box that scrolls (a sidebar, a table wrapper): partly in view.
    for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
      const s = getComputedStyle(p);
      if (s.overflowX === 'visible' && s.overflowY === 'visible') continue;
      const b = p.getBoundingClientRect();
      if (r.top < b.top - 0.5 || (!tall && r.bottom > b.bottom + 0.5) || r.left < b.left - 0.5 || r.right > b.right + 0.5) return 2;
    }
    // An inline element that wraps has one box per line: the centre of the part on the screen of
    // any of them counts.
    const group = el instanceof SVGElement ? el.parentElement : null;
    const own = Array.from(el.getClientRects()).filter((q) => q.width > 0 && q.height > 0);
    const boxes = own.length && own.every((q) => q.left >= r.left - 0.5 && q.right <= r.right + 0.5) ? own : [r];
    const shown = boxes.some((q) => {
      const top = Math.max(q.top, tall ? bar : -Infinity);
      const bottom = Math.min(q.bottom, vh);
      const hit = document.elementFromPoint((q.left + q.right) / 2, (top + bottom) / 2);
      return !!hit && (hit === el || el.contains(hit) || (!!group && group.contains(hit)));
    });
    return shown ? 1 : 2;
  }, SEEN_BOX);
}

/** The point of `l` a reader clicks, relative to its box, when it differs from the default (the centre of the box it sees, T3); else undefined. */
export async function clickPoint(l: Locator): Promise<{x: number; y: number} | undefined> {
  return l.first().evaluate((el, seenBox) => {
    const seen = new Function(`return ${seenBox}`)() as (e: Element) => DOMRect;
    const r = el.getBoundingClientRect();
    const t = seen(el);
    if (Math.abs(t.left - r.left) < 0.5 && Math.abs(t.right - r.right) < 0.5 && Math.abs(t.top - r.top) < 0.5) return undefined;
    return {x: (t.left + t.right) / 2 - r.left, y: (t.top + t.bottom) / 2 - r.top};
  }, SEEN_BOX);
}

/**
 * Whether scrolling the page up or down can bring `l` into view: it is drawn, the box a reader
 * sees lies across the viewport's width (not off to a side, in a closed drawer or in a box scrolled
 * sideways), and no ancestor hides it. The bot scrolls only up and down, as a reader with a wheel
 * does.
 */
export async function reachable(l: Locator): Promise<boolean> {
  if ((await l.count()) === 0) return false;
  return l.first().evaluate((el, seenBox) => {
    const seen = new Function(`return ${seenBox}`)() as (e: Element) => DOMRect;
    const r = seen(el);
    const vw = document.documentElement.clientWidth;
    if (r.width === 0 || r.height === 0) return false;
    if (r.left < -0.5 || r.right > vw + 0.5) return false;
    for (let p: Element | null = el; p; p = p.parentElement) {
      const s = getComputedStyle(p);
      if (s.visibility === 'hidden' || s.display === 'none' || Number(s.opacity) === 0) return false;
      if (p !== el && s.overflowX !== 'visible') {
        const b = p.getBoundingClientRect();
        if (r.left < b.left - 0.5 || r.right > b.right + 0.5) return false;
      }
    }
    return true;
  }, SEEN_BOX);
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

/** A name of a card: `^…` is a pattern, anything else the exact name. */
export function nameOf(name: string): string | RegExp {
  return name.startsWith('^') ? new RegExp(name) : name;
}

/** The part of the screen a step names. */
export function region(page: Page, where: 'navbar' | 'sidebar' | 'toc' | 'main'): Locator {
  switch (where) {
    case 'navbar':
      return page.getByRole('navigation', {name: 'Main'});
    case 'sidebar':
      // Below 997 px the docs sidebar is drawn inside the menu of the navbar.
      return page.locator('nav[aria-label="Docs sidebar"], .navbar-sidebar');
    case 'toc':
      return page.locator('.table-of-contents');
    case 'main':
      return page.locator('main');
  }
}

/** Buttons that open a part of the screen a control may hide in, in the order the bot tries them. */
const OPENERS: Record<string, (string | RegExp)[]> = {
  navbar: ['Toggle navigation bar', /Back to main menu/],
  sidebar: ['Toggle navigation bar'],
  toc: ['On this page'],
  main: [],
};

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

  /** Scrolls the page, or the box the target sits in, until `l` is in view: one action per scroll of at most 80 % of it. */
  async reach(l: Locator, what: string): Promise<void> {
    let g = await this.look(l, what);
    for (let i = 0; g !== 1 && i < 12; i++) {
      if ((await l.count()) === 0) throw new Error(`not on the page: ${what}`);
      const plan = await l.first().evaluate((el) => {
        const r = el.getBoundingClientRect();
        const vh = window.innerHeight;
        const vw = document.documentElement.clientWidth;
        const bar = document.querySelector('.navbar')?.getBoundingClientRect().bottom ?? 0;
        let box: Element | null = null;
        for (let p = el.parentElement; p && p !== document.body && p !== document.documentElement; p = p.parentElement) {
          const s = getComputedStyle(p);
          if ((s.overflowY === 'auto' || s.overflowY === 'scroll') && p.scrollHeight > p.clientHeight + 1) {
            box = p;
            break;
          }
        }
        const b = box ? box.getBoundingClientRect() : {top: bar, bottom: vh, left: 0, right: vw};
        const top = Math.max(b.top, bar, 0);
        const bottom = Math.min(b.bottom, vh);
        const h = bottom - top;
        const step = Math.round(h * 0.8);
        let need = 0;
        if (r.height > h - 8) need = r.top - top - 4;
        else if (r.bottom > bottom) need = Math.min(step, r.bottom - bottom + 24);
        else if (r.top < top) need = -Math.min(step, top - r.top + 24);
        if (Math.abs(need) > step) need = Math.sign(need) * step;
        const x = (Math.max(b.left, 0) + Math.min(b.right, vw)) / 2;
        return {need: Math.round(need), x, y: (top + bottom) / 2};
      });
      if (Math.abs(plan.need) < 2) break;
      await this.page.mouse.move(plan.x, plan.y);
      await this.page.mouse.wheel(0, plan.need);
      this.actions += 1;
      this.log.push(`scroll ${plan.need}`);
      await this.page.waitForTimeout(80);
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

  /** The first match of `l` that is in view, else the first that is drawn at all, else null. */
  private async pick(l: Locator): Promise<Locator | null> {
    const n = Math.min(await l.count(), 40);
    let drawn: Locator | null = null;
    for (let i = 0; i < n; i++) {
      const one = l.nth(i);
      if ((await where(one)) === 1) return one;
      if (!drawn && (await reachable(one))) drawn = one;
    }
    return drawn;
  }

  /** Clicks a link and waits until the page shows where it leads. */
  async click(l: Locator, what: string): Promise<void> {
    const href = await l.first().getAttribute('href');
    await this.act(l, what, async () => {
      await l.first().click({position: await clickPoint(l)});
      if (href && href.startsWith('/')) {
        const want = new URL(href, this.page.url());
        await this.page.waitForURL((u) => u.pathname.replace(/\/$/, '') === want.pathname.replace(/\/$/, '') && u.hash === want.hash, {timeout: 15_000});
      }
    });
  }

  /** A click into the field and the typing: two actions. */
  type(l: Locator, text: string, what: string): Promise<void> {
    return this.act(l, what, () => l.first().fill(text), 2);
  }

  /**
   * Opening the list and choosing the option: two actions. The choice is typed into the focused
   * list, as a person's key presses are, so the browser counts what moves after it as caused by
   * input (layout_shift.score leaves it out); a label the keys cannot reach is chosen directly.
   */
  choose(l: Locator, option: string, what: string): Promise<void> {
    return this.act(l, what, async () => {
      await l.first().focus();
      await this.page.keyboard.type(option.replace(/[^\x20-\x7e].*$/, '').trim());
      const chosen = await l.first().evaluate((el) => (el as HTMLSelectElement).selectedOptions[0]?.label ?? '');
      if (chosen !== option) await l.first().selectOption({label: option});
    }, 2);
  }

  /** Plays one step of a card: finds its control on the screen, opening a menu when it hides there. */
  async step(s: Step): Promise<void> {
    const page = this.page;
    if ('select' in s) return this.choose(page.getByRole('combobox', {name: s.select}), s.option, `the list ${s.select}`);
    if ('type' in s) return this.type(page.getByRole('searchbox', {name: s.type}), s.text, `the field ${s.type}`);
    const what = `the ${s.click} ${s.name} in the ${s.in}`;
    const find = () => region(page, s.in).getByRole(s.click, {name: nameOf(s.name), exact: !s.name.startsWith('^')});
    let l = await this.pick(find());
    for (const opener of OPENERS[s.in]) {
      if (l) break;
      const b = await this.pick(page.getByRole('button', {name: opener}));
      if (!b || (await where(b)) !== 1) continue;
      this.actions += 1;
      this.log.push(`open ${String(opener)}`);
      await b.click();
      await settle(page);
      await this.onState(`after opening ${String(opener)}`);
      l = await this.pick(find());
    }
    if (!l) throw new Error((await find().count()) === 0 ? `not on the page: ${what}` : `on the page but not on the screen: ${what}`);
    if (s.click === 'link') return this.click(l, what);
    const control = l;
    return this.act(control, what, async () => control.click({position: await clickPoint(control)}));
  }

  async layoutShift(): Promise<number> {
    return this.page.evaluate(() => (window as unknown as {__shift: number}).__shift ?? 0);
  }

  /** A value the run never measured (no look, no act: it stopped first) is null, not 0 (S-02, T5). */
  async metrics(): Promise<Metrics> {
    const some = (xs: number[], f: (xs: number[]) => number) => (xs.length ? f(xs) : null);
    return {
      'screen_only.actions': this.actions,
      'screen_only.looks': this.looks,
      'screen_only.steers': this.steers,
      'screen_only.time_to_act_steps.max': some(this.toAct, (xs) => Math.max(...xs)),
      'glance_cost.median': some(this.glances, median),
      'glance_cost.p95': some(this.glances, p95),
      'reading_load.max': some(this.reading, (xs) => Math.max(...xs)),
      'reading_load.median': some(this.reading, median),
      'layout_shift.score': Math.round((await this.layoutShift()) * 10_000) / 10_000,
    };
  }
}
