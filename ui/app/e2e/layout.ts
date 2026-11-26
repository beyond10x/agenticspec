// The layout contract (L2) and the accessibility gate, checked at every state of a scenario:
// no horizontal page scroll and no element running past the viewport or scrolling sideways in
// its box; no text cut off without its full value in a title; the always-visible elements of the
// page 100 % in view and not covered; 0 serious or critical axe-core violations.

import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';
import { where } from './bot.ts';
import { adapter, pageKind } from './scenarios.ts';

export async function contract(page: Page): Promise<string[]> {
  const found = await page.evaluate(() => {
    const out: string[] = [];
    const vw = document.documentElement.clientWidth;
    const name = (el: Element) => {
      const id = el.id ? `#${el.id}` : '';
      const cls = typeof el.className === 'string' && el.className ? `.${el.className.trim().split(/\s+/).join('.')}` : '';
      return `${el.tagName.toLowerCase()}${id}${cls}`;
    };
    if (document.documentElement.scrollWidth > vw + 1) out.push(`page scrolls sideways: ${document.documentElement.scrollWidth} > ${vw}`);
    const clipped = (el: Element) => {
      for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
        const s = getComputedStyle(p);
        if (s.overflowX !== 'visible') return true;
      }
      return false;
    };
    for (const el of Array.from(document.body.querySelectorAll('*'))) {
      if (el.closest('svg') && el.tagName.toLowerCase() !== 'svg') continue;
      const s = getComputedStyle(el);
      if (s.display === 'none' || s.visibility === 'hidden') continue;
      const r = el.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue;
      if ((r.right > vw + 1 || r.left < -1) && !clipped(el)) out.push(`runs past the viewport: ${name(el)} (${Math.round(r.left)}..${Math.round(r.right)} of ${vw})`);
      const he = el as HTMLElement;
      if ((s.overflowX === 'auto' || s.overflowX === 'scroll') && he.scrollWidth > he.clientWidth + 1) {
        out.push(`scrolls sideways in its box: ${name(el)} (${he.scrollWidth} > ${he.clientWidth})`);
      }
      if (s.textOverflow === 'ellipsis' && he.scrollWidth > he.clientWidth + 1) {
        const full = he.textContent?.trim() ?? '';
        const title = he.getAttribute('title') ?? he.closest('[title]')?.getAttribute('title') ?? '';
        if (!title || !title.includes(full.replace(/…$/, ''))) out.push(`text cut off without its full value: ${name(el)} "${full.slice(0, 40)}"`);
      }
    }
    return out;
  });
  const kind = pageKind(new URL(page.url()).pathname);
  for (const a of adapter.always_visible) {
    if (!a.pages.includes(kind)) continue;
    const l = a.name ? page.getByRole(a.role, { name: a.name }) : page.getByRole(a.role);
    const g = await where(l);
    if (g !== 1) found.push(`${a.id} is not fully in view and uncovered (glance ${g})`);
  }
  return [...new Set(found)];
}

export async function axe(page: Page): Promise<string[]> {
  const r = await new AxeBuilder({ page }).withTags(adapter.axe_tags).analyze();
  return r.violations
    .filter((v) => v.impact === 'serious' || v.impact === 'critical')
    .map((v) => `axe ${v.impact} ${v.id}: ${v.nodes.length} node(s), ${v.nodes[0]?.target.join(' ')}`);
}
