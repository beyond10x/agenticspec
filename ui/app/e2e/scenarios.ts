// The scenarios of ux/adapter.json, played by the screen-only user. Each one starts on its page,
// does its task through visible controls and returns its target: the row or panel that shows the
// answer. `truth` says whether the target shows the scenario's truth (ux/scenarios/<name>).

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Locator, Page } from '@playwright/test';
import type { Bot } from './bot.ts';

export const UX = fileURLToPath(new URL('../../../ux/', import.meta.url));
export const OUT = fileURLToPath(new URL('../../../target/ux/', import.meta.url));

export interface Viewport {
  width: number;
  height: number;
}

export interface AlwaysVisible {
  id: string;
  role: 'banner' | 'searchbox' | 'group';
  name?: string;
  pages: string[];
}

export interface Adapter {
  scenarios: string[];
  gating_viewports: Viewport[];
  measured_viewports: Viewport[];
  themes: ('light' | 'dark')[];
  always_visible: AlwaysVisible[];
  metric_viewport_theme: 'light' | 'dark';
  axe_tags: string[];
}

export const adapter: Adapter = JSON.parse(readFileSync(`${UX}adapter.json`, 'utf8'));

interface Card {
  name: string;
  start: string;
  input: Record<string, string>;
  truth: Record<string, unknown>;
}

export function card(name: string): Card {
  return JSON.parse(readFileSync(`${UX}scenarios/${name}/scenario.json`, 'utf8'));
}

export interface Scenario {
  /** The task, done through visible controls. */
  act: (bot: Bot, c: Card) => Promise<void>;
  /** The row or panel that shows the answer. */
  target: (page: Page, c: Card) => Locator;
  /** Whether the screen shows the scenario's truth. */
  truth: (page: Page, c: Card) => Promise<boolean>;
}

const sessionRow = (page: Page, title: string) => page.getByRole('row').filter({ hasText: title });
const tileValue = (page: Page, label: string) =>
  page
    .locator('[data-ux-region="tiles"] .tile')
    .filter({ has: page.locator('.label', { hasText: new RegExp(`^${label}$`) }) })
    .locator('.value');

export const SCENARIOS: Record<string, Scenario> = {
  'find-session-by-search': {
    act: (bot, c) => bot.type(bot.page.getByRole('searchbox', { name: 'Search sessions' }), c.input.search, 'the search box'),
    target: (page, c) => sessionRow(page, (c.truth.rows as string[])[0]),
    async truth(page, c) {
      const titles = (await page.locator('tbody tr .title').allTextContents()).map((t) => t.trim());
      return JSON.stringify(titles) === JSON.stringify(c.truth.rows);
    },
  },

  'slice-kpi-averages': {
    act: (bot, c) =>
      bot.click(bot.page.getByRole('group', { name: 'Time slice' }).getByRole('button', { name: c.input.slice }), `the ${c.input.slice} slice`),
    target: (page) => page.locator('[data-ux-region="tiles"]'),
    async truth(page, c) {
      const pressed = await page.getByRole('group', { name: 'Time slice' }).getByRole('button', { name: c.input.slice }).getAttribute('aria-pressed');
      if (pressed !== 'true') return false;
      // The figures of 24 h equal those of the default 72 h on the fixtures; the heading tells them apart.
      if (!((await page.locator('h1 + .sub').textContent()) ?? '').includes(c.truth.heading as string)) return false;
      for (const [label, value] of Object.entries(c.truth.tiles as Record<string, string>)) {
        if ((await tileValue(page, label).textContent())?.trim() !== value) return false;
      }
      return true;
    },
  },

  'failing-check': {
    act: (bot, c) => bot.click(sessionRow(bot.page, c.input.session), `the row of ${c.input.session}`),
    target: (page, c) => page.locator('[data-ux-region="checks"] li').filter({ hasText: c.truth.check as string }),
    async truth(page, c) {
      const first = (await page.locator('[data-ux-region="checks"] li').first().textContent()) ?? '';
      return first.includes(c.truth.check as string) && first.includes(c.truth.status as string);
    },
  },

  'agent-lane': {
    async act(bot, c) {
      await bot.click(sessionRow(bot.page, c.input.session), `the row of ${c.input.session}`);
      await bot.click(bot.page.getByRole('button', { name: `Show only ${c.input.agent}` }), `the lane of ${c.input.agent}`);
    },
    target: (page, c) => page.getByRole('button', { name: `Show only ${c.input.agent}` }),
    async truth(page, c) {
      const lane = page.getByRole('button', { name: `Show only ${c.input.agent}` });
      if ((await lane.getAttribute('aria-pressed')) !== 'true') return false;
      const text = (await page.locator('[data-ux-region="timeline"]').textContent()) ?? '';
      return text.includes(c.truth.agent as string) && text.includes(c.truth.cost as string);
    },
  },

  'embed-session': {
    act: async () => {},
    target: (page, c) => page.getByRole('group', { name: c.truth.timeline as string }),
    async truth(page, c) {
      const banner = await page.getByRole('banner').count();
      const body = (await page.locator('main').textContent()) ?? '';
      return banner === 0 && body.includes(c.truth.cost as string);
    },
  },
};

/** The page kind of a URL path, for the always-visible elements. */
export function pageKind(path: string): string {
  if (path === '/' || path === '') return 'overview';
  if (path.startsWith('/session/')) return 'session';
  return 'embed';
}
