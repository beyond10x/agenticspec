import path from 'node:path';
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

// The gallery holds every component of the kit in the light and the dark theme.
const screens = path.resolve(import.meta.dirname, '../../../target/screens');
const sizes = [
  { width: 1366, height: 768 },
  { width: 390, height: 844 },
];

async function open(page: Page): Promise<void> {
  await page.goto(process.env.GALLERY_URL ?? '');
  await expect(page.locator('body[data-ready="true"]')).toBeAttached();
  await expect(page.locator('[data-sheet="dark"]')).toBeVisible();
}

for (const size of sizes) {
  test(`no serious or critical axe finding and no horizontal overflow at ${size.width}x${size.height}`, async ({ page }) => {
    await page.setViewportSize(size);
    await open(page);
    const result = await new AxeBuilder({ page }).analyze();
    const bad = result.violations.filter((v) => v.impact === 'serious' || v.impact === 'critical');
    expect(bad.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`)).toEqual([]);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
  });
}

test('the select opens a listbox, moves with the keys and chooses with Enter', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await open(page);
  const demo = page.locator('[data-demo="select-light"]');
  const combo = demo.getByRole('combobox', { name: 'Empty' });
  await combo.focus();
  await page.keyboard.press('ArrowDown');
  await expect(combo).toHaveAttribute('aria-expanded', 'true');
  const list = demo.getByRole('listbox', { name: 'Empty' });
  await expect(list.getByRole('option')).toHaveCount(5);
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');
  await expect(combo).toHaveAttribute('aria-expanded', 'false');
  await expect(combo).toHaveText(/solo/);
  await expect(combo).toBeFocused();
});

test('controls are drawn by the kit, not by the browser', async ({ page }) => {
  await page.setViewportSize({ width: 1366, height: 768 });
  await open(page);
  expect(await page.locator('select').count()).toBe(0);
  const looks = await page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>('button, input, textarea')].map((el) => {
      const s = getComputedStyle(el);
      return { tag: el.tagName + '.' + el.className + ' ' + (el.textContent ?? '').trim().slice(0, 20), font: s.fontFamily, radius: s.borderTopLeftRadius, appearance: s.appearance };
    }),
  );
  for (const l of looks) {
    expect(l.font, l.tag).toMatch(l.tag.startsWith('TEXTAREA') ? /Fira Code/ : /Inter/);
  }
  expect(looks.filter((l) => l.tag.startsWith('INPUT')).every((l) => l.radius !== '0px')).toBe(true);
});

test('screenshots of the gallery and the open select in both themes', async ({ page }) => {
  test.skip(!process.env.SCREENS, 'SCREENS=1 writes the screenshots');
  await page.setViewportSize({ width: 1366, height: 768 });
  await open(page);
  for (const theme of ['light', 'dark']) {
    const sheet = page.locator(`[data-sheet="${theme}"]`);
    await sheet.screenshot({ path: path.join(screens, `kit-gallery-${theme}.png`), animations: 'disabled' });
    const demo = page.locator(`[data-demo="select-${theme}"]`);
    await demo.getByRole('combobox', { name: 'Architecture' }).click();
    await expect(demo.getByRole('listbox', { name: 'Architecture' })).toBeVisible();
    await demo.hover();
    await page.locator(`[data-sheet="${theme}"] [data-panel="select"]`).screenshot({
      path: path.join(screens, `kit-select-open-${theme}.png`),
    });
    await page.keyboard.press('Escape');
  }
});
