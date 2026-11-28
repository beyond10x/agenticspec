import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

// The gallery of the block panels: a page per block, a panel per view, each over its fixture.
const fixtures = path.resolve(import.meta.dirname, '../../fixtures');
const screens = path.resolve(import.meta.dirname, '../../../../target/screens');

interface Fixture {
  view: string;
  rows: Record<string, unknown>[];
}

const views: Fixture[] = readdirSync(fixtures)
  .filter((f) => f.endsWith('.json'))
  .map((f) => JSON.parse(readFileSync(path.join(fixtures, f), 'utf8')) as Fixture);

/** The views of a block page: the panels generator names a page by its block. */
function viewsOf(block: string): Fixture[] {
  return views.filter((v) => v.view.split('.')[1] === block);
}

/** The row key of a fixture row: a panel keys its rows by the view's first field. */
function keyOf(row: Record<string, unknown>): string {
  return String(Object.values(row)[0]);
}

interface Link {
  hash: string;
  label: string;
}

/**
 * The block pages, read from the navigation of the generated app when the tests are collected, so
 * that a page gets tests of its own. Each link's address is the route's: `#/` for the home page,
 * `#/<name, its dots as slashes>` for every other page. The first test holds this list equal to the
 * links the gallery shows.
 */
const app = readFileSync(path.resolve(import.meta.dirname, '../src/generated/app.ts'), 'utf8');
const home = /\bhome: '([^']+)'/.exec(app)?.[1];
const pages: Link[] = [...app.matchAll(/\{ name: '([^']+)', label: '([^']+)' \}/g)].map(([, name, label]) => ({
  hash: name === home ? '#/' : `#/${name.replaceAll('.', '/')}`,
  label,
}));

async function open(page: Page): Promise<Link[]> {
  await page.goto(`${process.env.GALLERY_URL ?? ''}?still#/`);
  await expect(page.locator('body[data-ready="true"]')).toBeAttached();
  const links = await page
    .locator('nav[aria-label="Pages"] a')
    .evaluateAll((as) => as.map((a) => ({ hash: new URL((a as HTMLAnchorElement).href).hash, label: (a.textContent ?? '').trim() })));
  expect(links.length).toBeGreaterThanOrEqual(20);
  return links;
}

/** Opens a page of the gallery and waits until every panel has read its rows. */
async function visit(page: Page, link: Link): Promise<void> {
  await page.evaluate((h) => (location.hash = h), link.hash);
  await expect(page.locator('h1')).toHaveText(link.label);
  await expect(page.locator('[data-panel="rows"]').first()).toBeAttached();
  await expect(page.locator('[data-panel="rows"]:not([data-state="ready"])')).toHaveCount(0);
}

function blockOf(link: Link): string {
  return link.hash.replace(/^#\/?/, '') || 'delegation';
}

test('every panel of every block page shows the rows of its fixture', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  const links = await open(page);
  expect(links, 'the pages each axe test is made for').toEqual(pages);
  let panels = 0;
  for (const link of links) {
    await visit(page, link);
    const expected = viewsOf(blockOf(link));
    expect(expected.length, link.label).toBeGreaterThan(0);
    await expect(page.locator('[data-panel="rows"]'), link.label).toHaveCount(expected.length);
    const keys = await page.locator('tr[data-row]').evaluateAll((rs) => rs.map((r) => r.getAttribute('data-row')));
    for (const v of expected) {
      for (const row of v.rows) expect(keys, `${link.label}: ${v.view}`).toContain(keyOf(row));
    }
    panels += expected.length;
  }
  expect(panels).toBe(views.length);
});

for (const size of [
  { width: 1366, height: 768 },
  { width: 390, height: 844 },
]) {
  for (const theme of ['light', 'dark'] as const) {
    // One test per block page, so that no test runs axe on more than one page.
    test.describe(`at ${size.width}x${size.height}, ${theme}`, () => {
      for (const link of pages) {
        test(`${link.label}: no serious or critical axe finding and no horizontal overflow`, async ({ page }) => {
          await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
          await page.setViewportSize(size);
          await page.goto(`${process.env.GALLERY_URL ?? ''}?still${link.hash}`);
          await expect(page.locator('body[data-ready="true"]')).toBeAttached();
          await expect(page.locator(`html[data-theme="${theme}"]`)).toBeAttached();
          await visit(page, link);
          const result = await new AxeBuilder({ page }).analyze();
          const bad = result.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
          expect(bad.map((v) => `${link.label}: ${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
          const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
          expect(overflow, link.label).toBeLessThanOrEqual(0);
        });
      }
    });
  }
}

test('the page list of the axe tests is read from the generated app', () => {
  expect(home).toBeTruthy();
  expect(pages.length).toBeGreaterThanOrEqual(20);
  expect(new Set(pages.map((p) => p.hash)).size).toBe(pages.length);
});

test('on a narrow screen the Pages button opens the navigation and a link closes it', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await open(page);
  const nav = page.getByRole('navigation', { name: 'Pages' });
  await expect(nav).toBeHidden();
  const menu = page.getByRole('button', { name: 'Pages' });
  await menu.click();
  await expect(menu).toHaveAttribute('aria-expanded', 'true');
  await expect(nav).toBeVisible();
  await nav.getByRole('link', { name: 'Harness' }).click();
  await expect(page.locator('h1')).toHaveText('Harness');
  await expect(nav).toBeHidden();
  await expect(page.locator('nav[aria-label="Pages"] a[aria-current="page"]')).toHaveText('Harness');
});

test('screenshots of the gallery and some panels in both themes', async ({ page }) => {
  test.skip(!process.env.SCREENS, 'SCREENS=1 writes the screenshots');
  for (const theme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme: theme, reducedMotion: 'reduce' });
    await page.setViewportSize({ width: 1366, height: 768 });
    const links = await open(page);
    for (const block of ['Delegation', 'Harness', 'Architecture']) {
      const link = links.find((l) => l.label === block);
      if (!link) continue;
      await visit(page, link);
      await page.screenshot({ path: path.join(screens, `views-${block.toLowerCase()}-${theme}.png`) });
    }
    const harness = links.find((l) => l.label === 'Harness');
    if (harness) {
      await visit(page, harness);
      await page.locator('[data-panel="tool_calls"]').screenshot({ path: path.join(screens, `views-panel-tool-calls-${theme}.png`) });
    }
    await page.setViewportSize({ width: 390, height: 844 });
    await page.getByRole('button', { name: 'Pages' }).click();
    await page.screenshot({ path: path.join(screens, `views-narrow-menu-${theme}.png`) });
  }
});
