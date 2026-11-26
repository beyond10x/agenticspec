// Hard gates 1 to 5 of the ui-iteration-loop on every scenario, gating viewport and theme: the
// screen-only user plays the scenario (L3) and the layout contract and axe-core run at every state
// (L2). The light-theme runs write the scenario's metrics to target/ux/metrics for the ratchet.

import { mkdirSync, writeFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';
import { Bot, settle, shiftObserver, where } from './bot.ts';
import { open } from './harness.ts';
import { axe, contract } from './layout.ts';
import { adapter, card, OUT, SCENARIOS } from './scenarios.ts';

for (const name of adapter.scenarios) {
  for (const vp of adapter.gating_viewports) {
    for (const theme of adapter.themes) {
      test(`${name} ${vp.width}x${vp.height} ${theme}`, async ({ page }) => {
        const c = card(name);
        await page.setViewportSize(vp);
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.addInitScript(shiftObserver);
        const problems: string[] = [];
        const check = async (label: string) => {
          for (const p of await contract(page)) problems.push(`${label}: ${p}`);
          for (const p of await axe(page)) problems.push(`${label}: ${p}`);
        };
        await open(page, c.start, theme);
        const bot = new Bot(page, check);
        await settle(page);
        await check('start');
        const s = SCENARIOS[name];
        await s.act(bot, c);
        const target = s.target(page, c);
        await bot.reach(target, 'the target').catch(() => {});
        const seen = await where(target);
        const right = await s.truth(page, c);
        // Gate 3: the end state is on the screen and is the right one.
        expect.soft(seen, 'completion_miss: the target is not fully in view').toBe(1);
        expect.soft(right, 'false_complete: the target does not show the truth').toBe(true);
        // Gates 1, 2 and 5.
        expect.soft(problems, bot.log.join('\n')).toEqual([]);
        if (theme === adapter.metric_viewport_theme) {
          mkdirSync(`${OUT}metrics`, { recursive: true });
          writeFileSync(
            `${OUT}metrics/${name}@${vp.width}x${vp.height}.json`,
            `${JSON.stringify({ scenario: name, viewport: vp, metrics: await bot.metrics(), log: bot.log }, null, 2)}\n`,
          );
        }
      });
    }
  }
}
