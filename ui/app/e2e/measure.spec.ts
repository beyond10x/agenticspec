// The Measure step of a round (not part of task check): every scenario at every gating and
// measured viewport in both themes, played by the screen-only user, with the layout contract and
// axe-core at every moment and a screenshot 0.75 s after it. Writes target/ux/measure/*.json and
// target/ux/measure/shots/; e2e/report.spec.ts turns them into the round's report.

import { mkdirSync, writeFileSync } from 'node:fs';
import { test } from '@playwright/test';
import { Bot, settle, shiftObserver, where } from './bot.ts';
import { open } from './harness.ts';
import { axe, contract } from './layout.ts';
import { adapter, card, OUT, SCENARIOS } from './scenarios.ts';

export interface Moment {
  label: string;
  shot: string;
  problems: string[];
}

export interface Run {
  scenario: string;
  viewport: { width: number; height: number };
  theme: 'light' | 'dark';
  gating: boolean;
  moments: Moment[];
  metrics: Record<string, number>;
  completion: { seen: number; right: boolean; error: string | null };
  log: string[];
}

const DIR = `${OUT}measure/`;
const viewports = [
  ...adapter.gating_viewports.map((v) => ({ ...v, gating: true })),
  ...adapter.measured_viewports.map((v) => ({ ...v, gating: false })),
];

for (const name of adapter.scenarios) {
  for (const vp of viewports) {
    for (const theme of adapter.themes) {
      const id = `${name}-${vp.width}x${vp.height}-${theme}`;
      test(id, async ({ page }) => {
        const c = card(name);
        mkdirSync(`${DIR}shots`, { recursive: true });
        await page.setViewportSize({ width: vp.width, height: vp.height });
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.addInitScript(shiftObserver);
        const moments: Moment[] = [];
        const moment = async (label: string) => {
          const problems = [...(await contract(page)), ...(await axe(page))];
          await page.waitForTimeout(750);
          const shot = `${id}-${String(moments.length).padStart(2, '0')}.jpg`;
          await page.screenshot({ path: `${DIR}shots/${shot}`, type: 'jpeg', quality: 80 });
          moments.push({ label, shot, problems });
        };
        await open(page, c.start, theme);
        const bot = new Bot(page, moment);
        await settle(page);
        await moment('start');
        const s = SCENARIOS[name];
        let error: string | null = null;
        try {
          await s.act(bot, c);
          await bot.reach(s.target(page, c), 'the target');
        } catch (e) {
          error = String(e instanceof Error ? e.message : e);
        }
        const target = s.target(page, c);
        const completion = { seen: await where(target), right: await s.truth(page, c).catch(() => false), error };
        await moment('completion');
        const run: Run = {
          scenario: name,
          viewport: { width: vp.width, height: vp.height },
          theme,
          gating: vp.gating,
          moments,
          metrics: { ...(await bot.metrics()) },
          completion,
          log: bot.log,
        };
        writeFileSync(`${DIR}${id}.json`, `${JSON.stringify(run, null, 2)}\n`);
      });
    }
  }
}
