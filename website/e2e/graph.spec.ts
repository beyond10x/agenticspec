// The rendered page of a large architecture: the graph at its natural size, panned by keyboard and
// zoomed with tools a keyboard reaches, the parts table on the first screen, and a page that does
// not overflow a phone. The 37-part fixture's page is rendered by scripts/check-site.sh into
// target/site-ux/large-37.html (agenticspec sim render), from
// crates/agenticspec-render/tests/fixtures/large-37.yaml; the other pages are the site's own.

import AxeBuilder from '@axe-core/playwright';
import {existsSync} from 'node:fs';
import {expect, test, type Frame, type Page} from '@playwright/test';
import {OUT, adapter, open} from './site';

const FIXTURE = `${OUT}large-37.html`;

async function fixture(page: Page, width: number, height: number): Promise<void> {
  expect(existsSync(FIXTURE), `${FIXTURE} is rendered by scripts/check-site.sh`).toBe(true);
  await page.setViewportSize({width, height});
  await page.goto(`file://${FIXTURE}?theme=light`, {waitUntil: 'load'});
}

/** The graph's drawn width, natural width and the drawn size of a part name, in CSS px. */
async function graph(page: Page | Frame): Promise<{drawn: number; natural: number; name: number; view: number}> {
  return page.evaluate(() => {
    const svg = document.querySelector('svg.graph') as SVGSVGElement;
    const drawn = svg.getBoundingClientRect().width;
    const natural = svg.viewBox.baseVal.width;
    const name = svg.querySelector('text.name') as SVGTextElement;
    const view = (document.querySelector('.graph-view') as HTMLElement).clientWidth;
    return {drawn, natural, name: (parseFloat(getComputedStyle(name).fontSize) * drawn) / natural, view};
  });
}

/** The architecture frame of a docs page, loaded. */
async function architecture(page: Page): Promise<Frame> {
  const frame = await (await page.locator('main iframe').first().elementHandle())?.contentFrame();
  expect(frame, 'the architecture frame').toBeTruthy();
  await frame!.waitForLoadState('load');
  await frame!.locator('svg.graph').waitFor();
  return frame!;
}

async function tabTo(page: Page, label: string, key = 'Tab'): Promise<void> {
  for (let i = 0; i < 60; i++) {
    await page.keyboard.press(key);
    if ((await page.evaluate(() => document.activeElement?.getAttribute('aria-label') ?? '')) === label) return;
  }
  throw new Error(`Tab never reaches ${label}`);
}

test('the 37-part graph is drawn at its natural size and pans to its last part', async ({page}) => {
  await fixture(page, 1280, 800);
  const g = await graph(page);
  expect(g.natural).toBeGreaterThan(g.view);
  expect(Math.abs(g.drawn - g.natural)).toBeLessThan(1);
  expect(g.name).toBeCloseTo(13.5, 1);
  const view = page.locator('.graph-view');
  await expect(view).toHaveAttribute('tabindex', '0');
  await view.focus();
  await page.keyboard.press('ArrowRight');
  await expect.poll(() => view.evaluate((v) => v.scrollLeft)).toBeGreaterThan(0);
  await view.evaluate((v) => (v.scrollLeft = v.scrollWidth));
  const last = await page.evaluate(() => {
    const v = (document.querySelector('.graph-view') as HTMLElement).getBoundingClientRect();
    const right = Math.max(...[...document.querySelectorAll('svg.graph .nodes > .node')].map((n) => n.getBoundingClientRect().right));
    return {right, edge: v.right};
  });
  expect(last.right).toBeLessThanOrEqual(last.edge + 1);
});

test('fit scales the graph to its card and actual size restores it, by Tab and Enter', async ({page}) => {
  await fixture(page, 1280, 800);
  await tabTo(page, 'Fit to width');
  await page.keyboard.press('Enter');
  let g = await graph(page);
  expect(g.drawn).toBeLessThanOrEqual(g.view);
  expect(g.drawn).toBeGreaterThan(g.view * 0.9);
  await tabTo(page, 'Zoom in', 'Shift+Tab');
  await page.keyboard.press('Enter');
  expect((await graph(page)).drawn).toBeGreaterThan(g.drawn);
  await tabTo(page, 'Actual size');
  await page.keyboard.press('Enter');
  g = await graph(page);
  expect(Math.abs(g.drawn - g.natural)).toBeLessThan(1);
});

test('at 1366x768 the parts table is on the first screen and a part opens from it', async ({page}) => {
  await fixture(page, 1366, 768);
  const table = page.locator('.parts-card');
  const box = await table.boundingBox();
  expect(box && box.y < 768 - 120, 'the parts table starts on the first screen').toBe(true);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
  await page.getByRole('button', {name: 'gateway', exact: true}).click();
  await expect(page.locator('.detail[data-detail="gateway"]')).toBeVisible();
  await page.locator('select[data-link-filter]').selectOption('gateway');
  await expect(page.locator('.links-table tbody tr:visible')).toHaveCount(13);
  const axe = await new AxeBuilder({page}).withTags(adapter.axe_tags).analyze();
  expect(axe.violations.map((v) => `${v.id}: ${v.nodes.length}`)).toEqual([]);
});

test('a small architecture is drawn at its natural size with no zoom tools', async ({page}) => {
  await page.setViewportSize({width: 1280, height: 800});
  await open(page, '/docs/architectures/solo', 'light');
  const frame = await architecture(page);
  const g = await graph(frame);
  expect(Math.abs(g.drawn - g.natural)).toBeLessThan(1);
  await expect(frame.locator('.graph-tools')).toBeHidden();
});

test('the workstation page at 390 px overflows nothing and shows its whole header', async ({page}) => {
  await page.setViewportSize({width: 390, height: 844});
  await open(page, '/docs/architectures/workstation', 'light');
  const frame = await architecture(page);
  const r = await frame.evaluate(() => {
    const doc = document.documentElement;
    const inside = (sel: string) => {
      const el = document.querySelector(sel);
      if (!el) return `${sel} missing`;
      const b = el.getBoundingClientRect();
      return b.left >= 0 && b.right <= doc.clientWidth + 0.5 ? 'ok' : `${sel} runs ${Math.round(b.left)}..${Math.round(b.right)} past ${doc.clientWidth}`;
    };
    return {scroll: doc.scrollWidth, client: doc.clientWidth, parts: ['header.top h1', 'header.top .summary', 'header.top .facts'].map(inside)};
  });
  expect(r.scroll).toBeLessThanOrEqual(r.client);
  expect(r.parts).toEqual(['ok', 'ok', 'ok']);
});
