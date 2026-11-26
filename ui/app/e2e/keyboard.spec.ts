// Gate 5, second half: every scenario can be done with the keyboard alone (Tab, Enter, typing,
// Page Down), and ends with its target in view and showing its truth.

import { expect, test, type Locator, type Page } from '@playwright/test';
import { settle, TYPING_PAUSE_MS, where } from './bot.ts';
import { open } from './harness.ts';
import { adapter, card, SCENARIOS } from './scenarios.ts';

async function tabTo(page: Page, l: Locator, what: string): Promise<void> {
  for (let i = 0; i < 120; i++) {
    await page.keyboard.press('Tab');
    if (await l.first().evaluate((el) => el === document.activeElement)) return;
  }
  throw new Error(`Tab never reaches ${what}`);
}

async function pageDownTo(page: Page, l: Locator): Promise<void> {
  for (let i = 0; i < 12 && (await where(l)) !== 1; i++) {
    await page.keyboard.press('PageDown');
    await settle(page);
  }
}

const KEYS: Record<string, (page: Page, input: Record<string, string>) => Promise<void>> = {
  async 'find-session-by-search'(page, input) {
    await tabTo(page, page.getByRole('searchbox', { name: 'Search sessions' }), 'the search box');
    await page.keyboard.type(input.search);
    await page.waitForTimeout(TYPING_PAUSE_MS);
  },
  async 'slice-kpi-averages'(page, input) {
    await tabTo(page, page.getByRole('group', { name: 'Time slice' }).getByRole('button', { name: input.slice }), 'the slice');
    await page.keyboard.press('Enter');
  },
  async 'failing-check'(page, input) {
    await tabTo(page, page.getByRole('row').filter({ hasText: input.session }), 'the session row');
    await page.keyboard.press('Enter');
  },
  async 'agent-lane'(page, input) {
    await tabTo(page, page.getByRole('row').filter({ hasText: input.session }), 'the session row');
    await page.keyboard.press('Enter');
    await settle(page);
    await tabTo(page, page.getByRole('button', { name: `Show only ${input.agent}` }), 'the lane');
    await page.keyboard.press('Enter');
  },
  async 'embed-session'() {},
};

const vp = adapter.gating_viewports[0];

for (const name of adapter.scenarios) {
  test(`${name} by keyboard`, async ({ page }) => {
    const c = card(name);
    await page.setViewportSize(vp);
    await open(page, c.start, adapter.metric_viewport_theme);
    await settle(page);
    await KEYS[name](page, c.input);
    await settle(page);
    const s = SCENARIOS[name];
    const target = s.target(page, c);
    await pageDownTo(page, target);
    expect(await where(target), 'the target is in view').toBe(1);
    expect(await s.truth(page, c), 'the target shows the truth').toBe(true);
  });
}
