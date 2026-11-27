// The embedded architecture frames of a page: each must end on the page its src names (after any
// redirect), answered with 200, drawing something, and that page must be the architecture: its
// title starts with the name in the src.

import type {Page} from '@playwright/test';

/** Starts recording the final answer to every frame document of `page`; call before opening it. */
export function watchFrames(page: Page): Record<string, number> {
  const statuses: Record<string, number> = {};
  page.on('response', (r) => {
    const s = r.status();
    if (r.request().resourceType() === 'document' && r.frame() !== page.mainFrame() && (s < 300 || s >= 400)) statuses[new URL(r.url()).pathname] = s;
  });
  return statuses;
}

export async function frameProblems(page: Page, statuses: Record<string, number>): Promise<string[]> {
  const problems: string[] = [];
  const iframes = page.locator('main iframe');
  for (let i = 0; i < (await iframes.count()); i++) {
    const el = iframes.nth(i);
    const src = new URL((await el.getAttribute('src')) ?? '', page.url()).pathname;
    const name = src.replace(/^.*\//, '').replace(/\.html$/, '');
    const frame = await (await el.elementHandle())?.contentFrame();
    await frame?.waitForLoadState('load').catch(() => {});
    const landed = frame ? new URL(frame.url()).pathname : '';
    const title = frame ? await frame.title().catch(() => '') : '';
    const drawn = frame ? await frame.evaluate(() => document.body?.getBoundingClientRect().height ?? 0).catch(() => 0) : 0;
    const box = await el.boundingBox();
    if (landed.replace(/\.html$/, '') !== src.replace(/\.html$/, '')) problems.push(`frame ${src}: lands on ${landed || 'nothing'}`);
    if (statuses[landed] !== undefined && statuses[landed] !== 200) problems.push(`frame ${src}: HTTP ${statuses[landed]}`);
    if (!title.startsWith(name)) problems.push(`frame ${src}: shows "${title.slice(0, 60)}", not the ${name} architecture`);
    if (drawn < 50) problems.push(`frame ${src}: draws nothing (${Math.round(drawn)} px)`);
    if (!box || box.height < 50) problems.push(`frame ${src}: no height on the page`);
  }
  return problems;
}
