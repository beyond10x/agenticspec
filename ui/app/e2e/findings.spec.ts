// One regression test per finding of ux/findings (its regression_asset), each written to fail on
// the state the finding was seen on and to pass after its change.

import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { settle, shiftObserver, TYPING_PAUSE_MS } from './bot.ts';
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

test('UXF-006 with a search the figures and the heading cover the matching sessions', async ({ page }) => {
  await at(page, '/', 'light');
  await page.getByRole('searchbox', { name: 'Search sessions' }).fill('planner');
  await page.waitForTimeout(TYPING_PAUSE_MS);
  await settle(page);
  const sub = (await page.locator('h1 + .sub').textContent()) ?? '';
  expect(sub).toContain('1 session matching');
  const value = (label: string) =>
    page.locator('[data-ux-region="tiles"] .tile').filter({ has: page.locator('.label', { hasText: new RegExp(`^${label}$`) }) }).locator('.value');
  await expect(value('Sessions')).toHaveText('1');
  await expect(value('Cost per session')).toHaveText('$0.38');
});

test('UXF-007 a lane keeps its name whole and its cost at 390x844', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await open(page, SESSION, 'light');
  await settle(page);
  const timeline = page.locator('[data-ux-region="timeline"]');
  await expect(timeline).toContainText('$0.07');
  const cut = await timeline.locator('text.lane-name').evaluateAll((els) =>
    els.filter((t) => {
      const r = t.getBoundingClientRect();
      const hit = t.parentElement?.querySelector('rect.lane-hit')?.getBoundingClientRect();
      return !!hit && r.right > hit.right + 0.5;
    }).map((t) => t.textContent),
  );
  expect(cut).toEqual([]);
  const size = await timeline.locator('text.lane-meta').first().evaluate((t) => parseFloat(getComputedStyle(t).fontSize));
  expect(size).toBeGreaterThanOrEqual(12);
});

test('UXF-008 Space chooses a lane like Enter and leaves the page where it is', async ({ page }) => {
  await at(page, SESSION, 'light');
  const lane = page.getByRole('button', { name: 'Show only Sub-agent 1' });
  await lane.focus();
  const y = await page.evaluate(() => window.scrollY);
  await page.keyboard.press('Space');
  await settle(page);
  await expect(lane).toHaveAttribute('aria-pressed', 'true');
  expect(await page.evaluate(() => window.scrollY)).toBe(y);
  await at(page, '/', 'light');
  await page.getByRole('row').filter({ hasText: 'Port the store' }).focus();
  await page.keyboard.press('Space');
  await settle(page);
  expect(new URL(page.url()).pathname).toBe(SESSION);
});

test('UXF-009 one name per figure, words for raw names, a singular for one', async ({ page }) => {
  await at(page, '/', 'light');
  const labels = await page.locator('[data-ux-region="tiles"] .label').allTextContents();
  expect(labels).toEqual(expect.arrayContaining(['Cache hit share', 'Waiting on a human', 'Agent working time']));
  await expect(page.locator('h1 + .sub')).toContainText('at API list price');
  await page.getByRole('searchbox', { name: 'Search sessions' }).fill('planner');
  await page.waitForTimeout(TYPING_PAUSE_MS);
  await settle(page);
  await expect(page.locator('[data-ux-region="sessions"] .hint')).toHaveText('1 session');
  await at(page, SESSION, 'light');
  await page.getByRole('radio', { name: /^All/ }).click();
  await settle(page);
  const what = (await page.locator('[data-ux-region="events"] td.what').allTextContents()).join(' | ');
  expect(what).not.toMatch(/SubAgent|\bMain\b(?! agent)|_/);
  const notes = (await page.locator('[data-ux-region="tiles"] .line').allTextContents()).join(' | ');
  expect(notes).not.toMatch(/\b1 secret exposures\b|\b1 busts\b/);
});

// UXF-010 is open: not in R-02's change set (ux/rounds/R-02/triage.md).
test.fixme('UXF-010 back to the list keeps the search and the slice', async ({ page }) => {
  await at(page, '/?q=planner&slice=24h', 'light');
  await page.getByRole('row').filter({ hasText: 'Fix the planner' }).click();
  await settle(page);
  await page.getByRole('link', { name: '← Sessions' }).click();
  await settle(page);
  await expect(page.getByRole('searchbox', { name: 'Search sessions' })).toHaveValue('planner');
  await expect(page.getByRole('group', { name: 'Time slice' }).getByRole('button', { name: '24 h' })).toHaveAttribute('aria-pressed', 'true');
});

test('UXF-011 the event log fits a phone and cuts no cell', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await open(page, SESSION, 'light');
  await settle(page);
  await page.getByRole('radio', { name: /^All/ }).click();
  await settle(page);
  const events = page.locator('[data-ux-region="events"]');
  const fits = await events.evaluate((card) => {
    const table = card.querySelector('table');
    return !!table && table.getBoundingClientRect().right <= card.getBoundingClientRect().right + 0.5;
  });
  expect(fits, 'the table ends inside its card').toBe(true);
  const cut = await events.locator('td').evaluateAll((tds) =>
    tds.filter((td) => td.scrollWidth > td.clientWidth + 1 && !td.getAttribute('title')).map((td) => td.textContent?.trim()),
  );
  expect(cut).toEqual([]);
});
