// The Measure step of a round (not part of task check:site): every docs task at every gating and
// measured viewport in both themes, played by the screen-only user, with the layout contract and
// axe-core at every moment and a screenshot 0.75 s after it; then every page of the sitemap in
// both themes: axe-core at every viewport (S-02, T2), the layout contract at every viewport, console errors. Writes
// target/site-ux/measure/; e2e/report.spec.ts turns it into the round's report.

import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {test} from '@playwright/test';
import {Bot, settle, shiftObserver, where} from './bot';
import {frameProblems, watchFrames} from './frames';
import {axeLines, axeOnce, contract, overflow, type AxeHit} from './layout';
import {target, truth} from './scenarios';
import {adapter, card, OUT, open, ROOT} from './site';

export interface Moment {
  label: string;
  path: string;
  shot: string;
  problems: string[];
}

export interface Run {
  scenario: string;
  viewport: {width: number; height: number};
  theme: 'light' | 'dark';
  gating: boolean;
  moments: Moment[];
  metrics: Record<string, number | null>;
  completion: {seen: number; wrong: string | null; error: string | null};
  log: string[];
}

export interface PageRun {
  path: string;
  theme: 'light' | 'dark';
  /** Serious and critical axe-core violations by viewport, `1440x900`. */
  axe: Record<string, AxeHit[]>;
  layout: Record<string, string[]>;
  errors: string[];
  frames: string[];
}

const DIR = `${OUT}measure/`;
const viewports = [
  ...adapter.gating_viewports.map((v) => ({...v, gating: true})),
  ...adapter.measured_viewports.map((v) => ({...v, gating: false})),
];

for (const name of adapter.scenarios) {
  for (const vp of viewports) {
    for (const theme of adapter.themes) {
      const id = `${name}-${vp.width}x${vp.height}-${theme}`;
      test(id, async ({page}) => {
        const c = card(name);
        mkdirSync(`${DIR}shots`, {recursive: true});
        await page.setViewportSize({width: vp.width, height: vp.height});
        await page.addInitScript(shiftObserver);
        const moments: Moment[] = [];
        const path = () => new URL(page.url()).pathname.replace(adapter.base_path, '') || '/';
        const moment = async (label: string) => {
          const problems = [...(await contract(page)), ...axeLines(await axeOnce(page, theme, label === 'completion' ? `end-${name}` : ''))];
          await page.waitForTimeout(750);
          const shot = `${id}-${String(moments.length).padStart(2, '0')}.jpg`;
          await page.screenshot({path: `${DIR}shots/${shot}`, type: 'jpeg', quality: 80});
          moments.push({label, path: path(), shot, problems});
        };
        await open(page, c.start, theme);
        const bot = new Bot(page, moment);
        await settle(page);
        await moment('start');
        let error: string | null = null;
        try {
          for (const s of c.steps) await bot.step(s);
          await bot.reach(target(page, c), 'the target');
        } catch (e) {
          error = e instanceof Error ? e.message : String(e);
        }
        const completion = {seen: await where(target(page, c)), wrong: await truth(page, c).catch((e) => String(e)), error};
        await moment('completion');
        const run: Run = {
          scenario: name,
          viewport: {width: vp.width, height: vp.height},
          theme,
          gating: vp.gating,
          moments,
          metrics: {...(await bot.metrics())},
          completion,
          log: bot.log,
        };
        writeFileSync(`${DIR}${id}.json`, `${JSON.stringify(run, null, 2)}\n`);
      });
    }
  }
}

/** Every page of the sitemap, not only the checked prefixes: the round looks at the whole site. */
function allPaths(): string[] {
  const xml = readFileSync(`${ROOT}website/build/sitemap.xml`, 'utf8');
  return [...xml.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => new URL(m[1]).pathname.replace(adapter.base_path, '') || '/').sort();
}

for (const path of allPaths()) {
  for (const theme of adapter.themes) {
    test(`sweep ${path} ${theme}`, async ({page}) => {
      mkdirSync(`${DIR}pages`, {recursive: true});
      const errors: string[] = [];
      page.on('console', (m) => {
        if (m.type() === 'error' && !m.text().includes('ERR_NETWORK_CHANGED')) errors.push(m.text().slice(0, 160));
      });
      page.on('pageerror', (e) => errors.push(e.message.slice(0, 160)));
      const statuses = watchFrames(page);
      await page.setViewportSize(adapter.gating_viewports[0]);
      await open(page, path, theme);
      await settle(page);
      const frameIssues = await frameProblems(page, statuses);
      // axe-core at every viewport in both themes (a box that scrolls on a phone only fails there),
      // the layout contract at every viewport in the metric theme.
      const axe: Record<string, AxeHit[]> = {};
      const layout: Record<string, string[]> = {};
      for (const vp of viewports) {
        await page.setViewportSize({width: vp.width, height: vp.height});
        await settle(page);
        axe[`${vp.width}x${vp.height}`] = await axeOnce(page, theme);
        if (theme === adapter.metric_viewport_theme) layout[`${vp.width}x${vp.height}`] = await overflow(page);
      }
      const run: PageRun = {path, theme, axe, layout, errors, frames: frameIssues};
      writeFileSync(`${DIR}pages/${path.replace(/[^A-Za-z0-9_-]+/g, '_')}-${theme}.json`, `${JSON.stringify(run, null, 2)}\n`);
    });
  }
}
