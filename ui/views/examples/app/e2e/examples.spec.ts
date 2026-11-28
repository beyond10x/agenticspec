import { readFileSync } from 'node:fs';
import path from 'node:path';
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

// The derived examples: the 37-part fixture, derived from the architecture alone, and the
// workstation, derived with its contract and its `view:` key.
const examples = path.resolve(import.meta.dirname, '../..');
const parts37: string[] = (
  JSON.parse(readFileSync(path.join(examples, 'large-37.fixtures/parts.json'), 'utf8')) as { rows: { name: string }[] }
).rows.map((r) => r.name);

interface Link {
  hash: string;
  label: string;
}

async function open(page: Page, example: string): Promise<Link[]> {
  await page.setViewportSize({ width: 1366, height: 768 });
  await page.goto(`${process.env.EXAMPLES_URL ?? ''}${example}.html?still#/`);
  await expect(page.locator('body[data-ready="true"]')).toBeAttached();
  return page
    .locator('nav[aria-label="Pages"] a')
    .evaluateAll((as) => as.map((a) => ({ hash: new URL((a as HTMLAnchorElement).href).hash, label: (a.textContent ?? '').trim() })));
}

async function overflow(page: Page): Promise<number> {
  return page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
}

test('the 37-part overview shows its parts table on the first screen and nothing runs past it sideways', async ({ page }) => {
  await open(page, 'large-37');
  const table = page.locator('[data-panel="parts"]');
  await expect(table.locator('tbody tr')).toHaveCount(37);
  const box = await table.boundingBox();
  expect(box && box.y < 768).toBe(true);
  expect(await overflow(page)).toBeLessThanOrEqual(0);
  const result = await new AxeBuilder({ page }).analyze();
  const bad = result.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
  expect(bad.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
});

test('every one of the 37 parts opens in one step from the overview, with no sideways overflow', async ({ page }) => {
  const links = await open(page, 'large-37');
  for (const name of parts37) {
    const link = links.find((l) => l.hash === `#/part.${name}` || l.hash === `#/part/${name}` || l.label === name);
    expect(link, name).toBeTruthy();
    await page.locator(`nav[aria-label="Pages"] a[href$="${link!.hash}"]`).first().click();
    await expect(page.locator('h1')).toHaveText(link!.label);
    expect(await overflow(page), name).toBeLessThanOrEqual(0);
    await page.goBack();
    await expect(page.locator('[data-panel="parts"]')).toBeVisible();
  }
});

test('the links are read in the reach table, filtered by a part', async ({ page }) => {
  await open(page, 'large-37');
  const reach = page.locator('[data-panel="reach"]');
  const all = await reach.locator('tbody tr').count();
  expect(all).toBeGreaterThan(10);
  const name = parts37[0];
  await reach.getByRole('searchbox').fill(name);
  const rows = reach.locator('tbody tr');
  await expect.poll(() => rows.count()).toBeLessThan(all);
  for (const text of await rows.allTextContents()) expect(text).toContain(name);
});

test('the graph is drawn at natural size with pan, zoom and fit', async ({ page }) => {
  await open(page, 'large-37');
  const graph = page.locator('[data-slot="draft-graph"]');
  await expect(graph.locator('svg.graph')).toBeAttached();
  await expect(graph.getByRole('button', { name: 'Zoom in' })).toBeVisible();
  await expect(graph.getByRole('button', { name: 'Zoom out' })).toBeVisible();
  await expect(graph.getByTitle('Fit the whole graph')).toBeVisible();
  await expect(graph.getByTitle('Actual size')).toBeVisible();
  expect(await overflow(page)).toBeLessThanOrEqual(0);
});

test('the workstation shows its labels, its frames and its actions', async ({ page }) => {
  const links = await open(page, 'workstation');
  expect(links.map((l) => l.label)).toContain('coding agents');
  await page.evaluate(() => (location.hash = '#/part/operator'));
  await expect(page.locator('h1')).toHaveText('operator');
  await expect(page.getByRole('group', { name: 'Actions' }).getByRole('button', { name: 'Send a message' })).toBeVisible();
  const harness = page.locator('[data-panel="area_harness"]');
  await expect(harness.getByRole('button', { name: 'Harness' })).toHaveAttribute('aria-expanded', 'false');
  await page.evaluate(() => (location.hash = '#/part/owner'));
  await expect(page.locator('h1')).toHaveText('owner');
  await expect(page.getByRole('button', { name: 'Answer a question' }).first()).toBeVisible();
  expect(await overflow(page)).toBeLessThanOrEqual(0);
});
