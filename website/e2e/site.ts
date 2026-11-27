// The adapter of the docs-site loop (ux/site/adapter.json), its scenario cards and the way a page
// of the built site is opened: in one theme, with reduced motion, on the server that
// playwright.config.ts started.

import {existsSync, readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import type {Page} from '@playwright/test';

export const ROOT = `${resolve(__dirname, '../..')}/`;
export const UX = `${ROOT}ux/site/`;
export const OUT = `${ROOT}target/site-ux/`;

export type Theme = 'light' | 'dark';

export interface Viewport {
  width: number;
  height: number;
}

export interface AlwaysVisible {
  id: string;
  role: 'navigation' | 'button' | 'searchbox';
  name: string;
  pages: string[];
  min_width: number;
}

export interface Adapter {
  base_path: string;
  scenarios: string[];
  anchor_scenarios: string[];
  gating_viewports: Viewport[];
  measured_viewports: Viewport[];
  themes: Theme[];
  always_visible: AlwaysVisible[];
  pages_checked: string[];
  metric_viewport_theme: Theme;
  axe_tags: string[];
}

export const adapter: Adapter = JSON.parse(readFileSync(`${UX}adapter.json`, 'utf8'));

export type Step =
  | {click: 'link' | 'button'; name: string; in: 'navbar' | 'sidebar' | 'toc' | 'main'}
  | {select: string; option: string}
  | {type: string; text: string};

export interface Card {
  name: string;
  anchor: boolean;
  card: {persona: string; task: string};
  start: string;
  steps: Step[];
  target: Record<string, string>;
  truth: Record<string, unknown>;
}

export function card(name: string): Card {
  return JSON.parse(readFileSync(`${UX}scenarios/${name}/scenario.json`, 'utf8'));
}

export const ORIGIN = `http://127.0.0.1:${process.env.SITE_UX_PORT}`;

/** The URL of a site path such as `/docs/blocks` (without the base path). */
export function url(path: string): string {
  return `${ORIGIN}${adapter.base_path}${path}`;
}

/** The site path of the page now open, without the base path and a trailing slash. */
export function sitePath(page: Page): string {
  const p = new URL(page.url()).pathname.replace(adapter.base_path, '') || '/';
  return p.length > 1 ? p.replace(/\/$/, '') : p;
}

/** The kind of page for the always-visible elements: the landing page or a docs page. */
export function pageKind(page: Page): string {
  return sitePath(page).startsWith('/docs') ? 'doc' : 'home';
}

/** Opens a site path in `theme`, with reduced motion; the theme is the one Docusaurus keeps. */
export async function open(page: Page, path: string, theme: Theme): Promise<void> {
  await page.emulateMedia({reducedMotion: 'reduce', colorScheme: theme});
  // The site's own page only: a frame may have no storage, and its error would count as the page's.
  await page.addInitScript((t) => {
    if (window.top !== window) return;
    try {
      localStorage.setItem('theme', t);
    } catch {
      // no storage: the colour scheme above still sets the theme, as Docusaurus follows the system
    }
  }, theme);
  await page.goto(url(path), {waitUntil: 'load'});
}

/** The site paths of the sitemap under the adapter's checked prefixes. */
export function sitemapPaths(): string[] {
  const f = `${ROOT}website/build/sitemap.xml`;
  if (!existsSync(f)) throw new Error('no website/build/sitemap.xml: run task site');
  const locs = [...readFileSync(f, 'utf8').matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => new URL(m[1]).pathname.replace(adapter.base_path, ''));
  return locs.filter((p) => adapter.pages_checked.some((c) => p === `/${c}` || p.startsWith(`/${c}/`))).sort();
}
