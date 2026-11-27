// The layout contract (L2) and the accessibility gate of the docs site, checked at every state of a
// scenario: no horizontal page scroll; no element running past the viewport or scrolling sideways
// in its box (a code block may); no text cut off without its full value in a title; the
// always-visible elements of the page 100 % in view and not covered; 0 serious or critical
// axe-core violations.

import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readdirSync, readFileSync, renameSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import AxeBuilder from '@axe-core/playwright';
import type {Page} from '@playwright/test';
import {nameOf, where} from './bot';
import {adapter, OUT, pageKind, ROOT, sitePath} from './site';

/** Layout problems of the page as it is now, without the always-visible elements. */
export async function overflow(page: Page): Promise<string[]> {
  return page.evaluate(() => {
    const out: string[] = [];
    const vw = document.documentElement.clientWidth;
    const name = (el: Element) => {
      const id = el.id ? `#${el.id}` : '';
      const cls =
        typeof el.className === 'string' && el.className
          ? `.${el.className
              .trim()
              .split(/\s+/)
              .filter((c) => !/_[A-Za-z0-9]{4}$/.test(c))
              .slice(0, 3)
              .join('.')}`
          : '';
      return `${el.tagName.toLowerCase()}${id}${cls === '.' ? '' : cls}`;
    };
    if (document.documentElement.scrollWidth > vw + 1) out.push(`page scrolls sideways: ${document.documentElement.scrollWidth} > ${vw}`);
    const clipped = (el: Element) => {
      for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
        if (getComputedStyle(p).overflowX !== 'visible') return true;
      }
      return false;
    };
    for (const el of Array.from(document.body.querySelectorAll('*'))) {
      if (el.closest('svg') && el.tagName.toLowerCase() !== 'svg') continue;
      const s = getComputedStyle(el);
      if (s.display === 'none' || s.visibility === 'hidden') continue;
      const r = el.getBoundingClientRect();
      if (r.width === 0 || r.height === 0) continue;
      if ((r.right > vw + 1 || r.left < -1) && !clipped(el) && s.position !== 'fixed') {
        out.push(`runs past the viewport: ${name(el)} (${Math.round(r.left)}..${Math.round(r.right)} of ${vw})`);
      }
      const he = el as HTMLElement;
      if ((s.overflowX === 'auto' || s.overflowX === 'scroll') && he.scrollWidth > he.clientWidth + 1 && !el.closest('pre')) {
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
}

/** The adapter's always-visible elements of this page and viewport that are not 100 % in view. */
export async function alwaysVisible(page: Page): Promise<string[]> {
  const out: string[] = [];
  const kind = pageKind(page);
  const vw = page.viewportSize()?.width ?? 0;
  for (const a of adapter.always_visible) {
    if (!a.pages.includes(kind) || vw < a.min_width) continue;
    const g = await where(page.getByRole(a.role, {name: nameOf(a.name)}));
    if (g !== 1) out.push(`${a.id} is not fully in view and uncovered (glance ${g})`);
  }
  return out;
}

export async function contract(page: Page): Promise<string[]> {
  return [...new Set([...(await overflow(page)), ...(await alwaysVisible(page))])];
}

export interface AxeHit {
  id: string;
  impact: string;
  nodes: number;
  first: string;
}

/** Serious and critical axe-core violations of the page under the adapter's tags. */
export async function axeHits(page: Page): Promise<AxeHit[]> {
  const r = await new AxeBuilder({page}).withTags(adapter.axe_tags).analyze();
  return r.violations
    .filter((v) => v.impact === 'serious' || v.impact === 'critical')
    .map((v) => ({id: v.id, impact: v.impact ?? '', nodes: v.nodes.length, first: v.nodes[0]?.target.join(' ') ?? ''}));
}

let buildDigest = '';

/**
 * The digest of what axe-core reads: every HTML file of website/build (the CSS and scripts are
 * named by their hash inside it), the axe-core version and the adapter's tags.
 */
function digest(): string {
  if (buildDigest) return buildDigest;
  const h = createHash('sha256');
  const walk = (dir: string) => {
    for (const e of readdirSync(dir, {withFileTypes: true}).sort((a, b) => a.name.localeCompare(b.name))) {
      const p = join(dir, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith('.html')) h.update(p).update(readFileSync(p));
    }
  };
  walk(`${ROOT}website/build`);
  h.update(JSON.parse(readFileSync(require.resolve('axe-core/package.json'), 'utf8')).version);
  h.update(adapter.axe_tags.join(','));
  buildDigest = h.digest('hex').slice(0, 16);
  return buildDigest;
}

/**
 * axe-core on the page, once per build: the result for a page state (its path and a label), a
 * viewport and a theme is kept on disk under target/site-ux/axe/<digest of the build>/ and read by every later
 * test and run. A scan costs up to 5 s, and the same page is opened by many runs.
 */
export async function axeOnce(page: Page, theme: string, state = ''): Promise<AxeHit[]> {
  const key = `${sitePath(page)}${state ? `#${state}` : ''}`.replace(/[^A-Za-z0-9_-]+/g, '_');
  const vp = page.viewportSize();
  const dir = `${OUT}axe/${digest()}/${vp?.width}x${vp?.height}-${theme}/`;
  const f = `${dir}${key}.json`;
  if (existsSync(f)) return JSON.parse(readFileSync(f, 'utf8'));
  const hits = await axeHits(page);
  mkdirSync(dir, {recursive: true});
  const tmp = `${f}.${process.pid}`;
  writeFileSync(tmp, JSON.stringify(hits));
  renameSync(tmp, f);
  return hits;
}

export function axeLines(hits: AxeHit[]): string[] {
  return hits.map((v) => `axe ${v.impact} ${v.id}: ${v.nodes} node(s), ${v.first}`);
}

export async function axe(page: Page): Promise<string[]> {
  return axeLines(await axeHits(page));
}
