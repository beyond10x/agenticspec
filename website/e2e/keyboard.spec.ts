// Gate 5, second half: every docs task can be done with the keyboard alone (Tab, Enter, typing,
// Page Down; the skip link before a control of the content) at the first gating viewport, and ends
// with its target in view and showing its truth. Each test prints its key presses.

import {expect, test, type Locator, type Page} from '@playwright/test';
import {nameOf, region, settle, where} from './bot';
import {EXPECTED_FAIL} from './debt';
import {target, truth} from './scenarios';
import {adapter, card, open, type Step} from './site';

async function tabTo(page: Page, l: Locator, what: string): Promise<number> {
  for (let i = 1; i <= 250; i++) {
    await page.keyboard.press('Tab');
    if (await l.evaluateAll((els) => (els as Element[]).includes(document.activeElement as Element))) return i;
  }
  throw new Error(`Tab never reaches ${what}`);
}

/**
 * A control in the page's content is reached as a keyboard user does: the first Tab of a fresh page
 * lands on "Skip to main content", Enter moves past the navbar and the sidebar. Returns the presses.
 */
async function skip(page: Page): Promise<number> {
  const fresh = await page.evaluate(() => !document.activeElement || document.activeElement === document.body);
  if (!fresh) return 0;
  await page.keyboard.press('Tab');
  const onSkip = await page.evaluate(() => (document.activeElement?.textContent ?? '').trim() === 'Skip to main content');
  if (!onSkip) return 1;
  await page.keyboard.press('Enter');
  return 2;
}

async function keys(page: Page, s: Step): Promise<number> {
  const inContent = !('click' in s) || s.in === 'main' || s.in === 'toc';
  const skipped = inContent ? await skip(page) : 0;
  return skipped + (await press(page, s));
}

async function press(page: Page, s: Step): Promise<number> {
  if ('select' in s) {
    const list = page.getByRole('combobox', {name: s.select});
    const n = await tabTo(page, list, `the list ${s.select}`);
    // A closed list chooses the option whose label starts with what is typed.
    await page.keyboard.type(s.option.replace(/[^\x20-\x7e].*$/, '').trim());
    const chosen = await list.evaluate((el) => (el as HTMLSelectElement).selectedOptions[0]?.label ?? '');
    if (chosen !== s.option) throw new Error(`typing chose "${chosen}", not "${s.option}"`);
    return n + 1;
  }
  if ('type' in s) {
    const n = await tabTo(page, page.getByRole('searchbox', {name: s.type}), `the field ${s.type}`);
    await page.keyboard.type(s.text);
    return n + 1;
  }
  const l = region(page, s.in).getByRole(s.click, {name: nameOf(s.name), exact: !s.name.startsWith('^')});
  const n = await tabTo(page, l, `the ${s.click} ${s.name}`);
  const href = await page.evaluate(() => (document.activeElement as HTMLAnchorElement | null)?.getAttribute('href') ?? null);
  await page.keyboard.press('Enter');
  if (href && href.startsWith('/')) {
    const want = new URL(href, page.url());
    await page.waitForURL((u) => u.pathname.replace(/\/$/, '') === want.pathname.replace(/\/$/, '') && u.hash === want.hash, {timeout: 15_000});
  }
  await settle(page);
  return n + 1;
}

const vp = adapter.gating_viewports[0];

for (const name of adapter.scenarios) {
  const title = `${name} by keyboard`;
  test(title, async ({page}) => {
    test.fail(!!EXPECTED_FAIL[title], EXPECTED_FAIL[title]);
    const c = card(name);
    await page.setViewportSize(vp);
    await open(page, c.start, adapter.metric_viewport_theme);
    await settle(page);
    let presses = 0;
    for (const s of c.steps) presses += await keys(page, s);
    const t = target(page, c);
    for (let i = 0; i < 12 && (await where(t)) !== 1; i++) {
      await page.keyboard.press('PageDown');
      presses += 1;
      await settle(page);
    }
    test.info().annotations.push({type: 'key presses', description: String(presses)});
    console.log(`${name}: ${presses} key presses`);
    expect(await where(t), 'the target is in view').toBe(1);
    expect(await truth(page, c), 'the target shows the truth').toBeNull();
  });
}
