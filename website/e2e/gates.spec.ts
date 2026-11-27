// Hard gates 1 to 5 of the ui-iteration-loop on every docs task, gating viewport and theme: the
// screen-only user plays the card (L3), the layout contract runs at every state and axe-core on
// every page the run opens (L2). The light-theme runs write the task's metrics to
// target/site-ux/metrics for the ratchet. A gate that fails on the current site is marked
// test.fail with its finding (ux/site/findings) in debt.ts: the debt is recorded, and the
// test turns red the day the defect is gone, so the mark is taken out with the fix.

import {mkdirSync, writeFileSync} from 'node:fs';
import {expect, test} from '@playwright/test';
import {Bot, settle, shiftObserver, where} from './bot';
import {expectedFail} from './debt';
import {axeLines, axeOnce, contract} from './layout';
import {truth, target} from './scenarios';
import {adapter, card, OUT, open, sitePath} from './site';

for (const name of adapter.scenarios) {
  for (const vp of adapter.gating_viewports) {
    for (const theme of adapter.themes) {
      const title = `${name} ${vp.width}x${vp.height} ${theme}`;
      test(title, async ({page}) => {
        const finding = expectedFail(title);
        test.fail(!!finding, finding);
        const c = card(name);
        await page.setViewportSize(vp);
        await page.addInitScript(shiftObserver);
        const problems: string[] = [];
        const axed = new Set<string>();
        const check = async (label: string) => {
          for (const p of await contract(page)) problems.push(`${label}: ${p}`);
          if (!axed.has(sitePath(page))) {
            axed.add(sitePath(page));
            for (const p of axeLines(await axeOnce(page, theme))) problems.push(`${label} (${sitePath(page)}): ${p}`);
          }
        };
        await open(page, c.start, theme);
        const bot = new Bot(page, check);
        await settle(page);
        await check('start');
        let error = '';
        try {
          for (const s of c.steps) await bot.step(s);
          await bot.reach(target(page, c), 'the target');
        } catch (e) {
          error = e instanceof Error ? e.message : String(e);
        }
        // A step that changed the page in place (a list, a field, a button) leaves a state of its own to scan.
        if (c.steps.some((s) => !('click' in s) || s.click !== 'link')) {
          for (const p of axeLines(await axeOnce(page, theme, `end-${name}`))) problems.push(`completion (${sitePath(page)}): ${p}`);
        }
        const seen = await where(target(page, c));
        const wrong = await truth(page, c);
        if (theme === adapter.metric_viewport_theme) {
          mkdirSync(`${OUT}metrics`, {recursive: true});
          writeFileSync(
            `${OUT}metrics/${name}@${vp.width}x${vp.height}.json`,
            `${JSON.stringify({scenario: name, viewport: vp, metrics: await bot.metrics(), log: bot.log, error}, null, 2)}\n`,
          );
        }
        // Gate 4: the screen-only user completes the task. Gate 3: the end state is on the screen and right.
        expect.soft(error, bot.log.join('\n')).toBe('');
        expect.soft(seen, 'completion_miss: the target is not fully in view').toBe(1);
        expect.soft(wrong, 'false_complete: the target does not show the truth').toBeNull();
        // Gates 1, 2 and 5.
        expect.soft(problems, bot.log.join('\n')).toEqual([]);
      });
    }
  }
}
