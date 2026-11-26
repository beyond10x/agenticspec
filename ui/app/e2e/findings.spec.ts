// One regression test per finding of ux/findings (its regression_asset), each written to fail on
// the state the finding was seen on and to pass after its change.

import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { settle, shiftObserver } from './bot.ts';
import { open, type Theme } from './harness.ts';
import { contract } from './layout.ts';

const SESSION = '/session/ses-demo-0001';
const DESK = { width: 1440, height: 900 };

async function violations(page: Page, rules: string[]): Promise<string[]> {
  const r = await new AxeBuilder({ page }).withRules(rules).analyze();
  return r.violations.flatMap((v) => v.nodes.map((n) => `${v.id}: ${n.target.join(' ')}`));
}

async function at(page: Page, path: string, theme: Theme): Promise<void> {
  await page.setViewportSize(DESK);
  await open(page, path, theme);
  await settle(page);
}

test('UXF-001 the event filter chips carry only the attributes of a radio', async ({ page }) => {
  await at(page, SESSION, 'light');
  expect(await violations(page, ['aria-allowed-attr'])).toEqual([]);
});

test('UXF-002 the timeline holds its lane buttons in a group, not an image', async ({ page }) => {
  await at(page, SESSION, 'light');
  expect(await violations(page, ['nested-interactive'])).toEqual([]);
  await expect(page.getByRole('button', { name: 'Show only Sub-agent 1' })).toBeVisible();
});

for (const theme of ['light', 'dark'] as const) {
  test(`UXF-003 muted notes reach 4.5:1 in the ${theme} theme`, async ({ page }) => {
    await at(page, '/', theme);
    const found = await violations(page, ['color-contrast']);
    await page.getByRole('row').filter({ hasText: 'Port the store' }).click();
    await settle(page);
    await page.locator('[data-ux-region="agents"]').getByRole('row').filter({ hasText: 'Sub-agent 1' }).click();
    await settle(page);
    found.push(...(await violations(page, ['color-contrast'])));
    expect(found).toEqual([]);
  });
}

test('UXF-004 no figure note is cut off without its full value', async ({ page }) => {
  for (const vp of [DESK, { width: 1280, height: 720 }]) {
    await page.setViewportSize(vp);
    await open(page, '/', 'light');
    await settle(page);
    expect((await contract(page)).filter((p) => p.startsWith('text cut off'))).toEqual([]);
  }
});

test('UXF-005 the embed page does not jump while it loads', async ({ page }) => {
  await page.addInitScript(shiftObserver);
  await at(page, '/embed/session/ses-demo-0001', 'light');
  await page.waitForTimeout(300);
  const shift = await page.evaluate(() => (window as unknown as { __shift: number }).__shift);
  expect(shift).toBeLessThanOrEqual(0.01);
});
