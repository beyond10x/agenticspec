// The targets and truths of the scenario cards (ux/site/scenarios/<name>/scenario.json). A card's
// steps are played by the screen-only user (bot.ts); here, by the kind of its target, is the element
// that must end 100 % in view and the check that the screen shows the card's truth.

import type {Locator, Page} from '@playwright/test';
import {sitePath, type Card} from './site';

type Kind = 'section' | 'section-list' | 'map-card' | 'kpi-rows' | 'section-table' | 'lifecycle';

const heading = (page: Page, name: string) =>
  page.locator('main article h2').filter({hasText: new RegExp(`^${name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\W*$`)});

/** The first list or table after a heading, inside its section. */
const after = (page: Page, name: string, tag: 'ul' | 'table') =>
  heading(page, name).locator(`xpath=following-sibling::*[self::h2 or descendant-or-self::${tag}][1][not(self::h2)]`).locator(`xpath=descendant-or-self::${tag}`).first();

/** The text of a section: from its heading to the next heading of its level. */
async function sectionText(page: Page, name: string): Promise<string> {
  const h = heading(page, name);
  if ((await h.count()) === 0) return '';
  return h.first().evaluate((el) => {
    let text = '';
    for (let n = el.nextElementSibling; n && n.tagName !== 'H2'; n = n.nextElementSibling) text += ` ${n.textContent ?? ''}`;
    return text;
  });
}

const mapCard = (page: Page, block: string) => page.locator('.as-map').getByRole('link', {name: new RegExp(`^${block}\\b`)});

export function target(page: Page, c: Card): Locator {
  const t = c.target;
  switch (t.kind as Kind) {
    case 'section':
      return heading(page, t.heading);
    case 'section-list':
      return after(page, t.heading, 'ul');
    case 'map-card':
      return mapCard(page, t.block);
    case 'kpi-rows':
      return page.locator('.as-kpis tbody');
    case 'section-table':
      return after(page, t.heading, 'table');
    case 'lifecycle':
      return page.locator('figure.as-life').filter({has: page.getByRole('img', {name: new RegExp(`^Lifecycle of ${t.entity}:`)})});
  }
}

const same = (a: string[], b: string[]) => JSON.stringify([...a].sort()) === JSON.stringify([...b].sort());

/** Whether the screen shows the card's truth; the reason when it does not. */
export async function truth(page: Page, c: Card): Promise<string | null> {
  const want = c.truth;
  if (sitePath(page) !== want.path) return `on ${sitePath(page)}, not ${want.path}`;
  const t = c.target;
  switch (t.kind as Kind) {
    case 'section-list': {
      const got = (await target(page, c).locator('li > a').allTextContents()).map((s) => s.trim());
      return same(got, want.blocks as string[]) ? null : `the list names ${got.join(', ')}`;
    }
    case 'section': {
      const text = await sectionText(page, t.heading);
      const missing = (want.kpis as string[]).filter((k) => !text.includes(k));
      return missing.length ? `the section lacks ${missing.join(', ')}` : null;
    }
    case 'map-card': {
      const card = mapCard(page, t.block).first();
      const label = await card.evaluate((el) => {
        const r = el.getBoundingClientRect();
        const vh = window.innerHeight;
        const vw = document.documentElement.clientWidth;
        const svg = el.closest('svg');
        if (!svg) return null;
        let best: {n: string; name: string; d: number; shown: boolean} | null = null;
        for (const n of Array.from(svg.querySelectorAll('.as-map__layer-n'))) {
          const b = n.getBoundingClientRect();
          const name = n.nextElementSibling;
          const nb = name?.getBoundingClientRect();
          const d = Math.abs((b.top + b.bottom) / 2 - (r.top + Math.min(r.height, 40) / 2));
          const shown = b.top >= 0 && b.bottom <= vh && b.left >= 0 && b.right <= vw && !!nb && nb.top >= 0 && nb.bottom <= vh && nb.right <= vw;
          if (!best || d < best.d) best = {n: n.textContent ?? '', name: name?.textContent ?? '', d, shown};
        }
        return best;
      });
      if (!label) return 'no layer label beside the card';
      if (!label.shown) return `the layer label (${label.n} ${label.name}) is not in view`;
      return label.n === want.layer && label.name === want.layer_name ? null : `the card sits by layer ${label.n} ${label.name}`;
    }
    case 'kpi-rows': {
      const got = (await page.locator('.as-kpis tbody tr td.as-kpis__what code').allTextContents()).map((s) => s.trim());
      if (!same(got, want.kpis as string[])) return `the table shows ${got.length} KPI(s): ${got.slice(0, 6).join(', ')}`;
      const count = ((await page.locator('.as-kpis__count').textContent()) ?? '').trim();
      return count === want.count ? null : `the count reads "${count}"`;
    }
    case 'section-table': {
      const rows = await target(page, c).locator('tbody tr').evaluateAll((trs) => trs.map((tr) => Array.from(tr.querySelectorAll('td')).map((td) => td.textContent?.trim() ?? '')));
      for (const [place, value] of Object.entries(want.best as Record<string, string>)) {
        const row = rows.find((r) => r[0] === place);
        if (!row) return `no row for ${place}`;
        if (row[2] !== value) return `${place}: best value ${row[2]}, not ${value}`;
      }
      return null;
    }
    case 'lifecycle': {
      const got = (await target(page, c).locator('.as-life__node--terminal text').allTextContents()).map((s) => s.trim());
      return same(got, want.terminal as string[]) ? null : `the end states drawn are ${got.join(', ') || 'none'}`;
    }
  }
}
