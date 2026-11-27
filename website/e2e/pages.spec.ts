// The page sweep: every page of the sitemap under the adapter's pages_checked (the building
// blocks, the architectures and the KPI catalogue), at every gating viewport: no horizontal page
// scroll, no console error, no broken internal link or anchor, and every embedded architecture
// frame lands on the architecture it names and draws it (frames.ts).

import {expect, test, type APIRequestContext, type Page} from '@playwright/test';
import {settle} from './bot';
import {expectedFail} from './debt';
import {frameProblems, watchFrames} from './frames';
import {adapter, ORIGIN, open, sitemapPaths} from './site';

const fetched = new Map<string, Promise<{status: number; body: string}>>();

/** One request per URL path and worker. */
function get(request: APIRequestContext, path: string): Promise<{status: number; body: string}> {
  let p = fetched.get(path);
  if (!p) {
    p = request.get(`${ORIGIN}${path}`, {maxRedirects: 5}).then(async (r) => ({status: r.status(), body: r.ok() ? await r.text() : ''}));
    fetched.set(path, p);
  }
  return p;
}

/** The same-origin links of the page and of its frames. */
async function links(page: Page): Promise<string[]> {
  const out = new Set<string>();
  for (const f of page.frames()) {
    if (!f.url().startsWith(ORIGIN)) continue;
    const hrefs = await f.evaluate(() => Array.from(document.querySelectorAll('a[href]')).map((a) => (a as HTMLAnchorElement).href));
    for (const h of hrefs) if (h.startsWith(ORIGIN)) out.add(h);
  }
  return [...out].sort();
}

async function broken(page: Page, request: APIRequestContext): Promise<string[]> {
  const out: string[] = [];
  for (const href of await links(page)) {
    const u = new URL(href);
    const r = await get(request, u.pathname);
    if (r.status !== 200) {
      out.push(`${u.pathname}: HTTP ${r.status}`);
      continue;
    }
    const id = decodeURIComponent(u.hash.slice(1));
    if (!id) continue;
    const esc = id.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    if (!new RegExp(`\\sid=(?:"${esc}"|'${esc}'|${esc}(?=[\\s>]))`).test(r.body)) out.push(`${u.pathname}${u.hash}: no such anchor`);
  }
  return out;
}

const [first, ...rest] = adapter.gating_viewports;

for (const path of sitemapPaths()) {
  const title = `page ${path}`;
  test(title, async ({page, request}) => {
    const finding = expectedFail(title);
    test.fail(!!finding, finding);
    const errors: string[] = [];
    // A change of the host's network (a container starting) cancels requests with ERR_NETWORK_CHANGED;
    // that is the machine, not the page.
    page.on('console', (m) => {
      if (m.type() === 'error' && !m.text().includes('ERR_NETWORK_CHANGED')) errors.push(`console: ${m.text().slice(0, 160)}`);
    });
    page.on('pageerror', (e) => errors.push(`page error: ${e.message.slice(0, 160)}`));
    const frames = watchFrames(page);
    await page.setViewportSize(first);
    await open(page, path, 'light');
    await settle(page);
    const problems = await frameProblems(page, frames);

    for (const vp of [first, ...rest]) {
      await page.setViewportSize(vp);
      await settle(page);
      const wide = await page.evaluate(() => [document.documentElement.scrollWidth, document.documentElement.clientWidth]);
      if (wide[0] > wide[1] + 1) problems.push(`${vp.width}x${vp.height}: page scrolls sideways (${wide[0]} > ${wide[1]})`);
    }
    problems.push(...(await broken(page, request)).map((b) => `broken link ${b}`));
    problems.push(...errors);
    expect(problems).toEqual([]);
  });
}
