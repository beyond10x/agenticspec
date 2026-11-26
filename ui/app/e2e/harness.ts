// Opens the built app (target/ui/www) on a page with nothing behind it: every view is answered
// from the synthetic fixtures of ui/fixtures by src/testing/fixtures.ts, the live channels are a
// stub that stays open, and the clock stands just after the newest fixture session. No server,
// no network, no key.

import { existsSync, readFileSync, statSync } from 'node:fs';
import { extname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';
import { answer, FIXTURE_NOW } from '../src/testing/fixtures.ts';

export const ORIGIN = 'http://ux.test';
const WWW = fileURLToPath(new URL('../../../target/ui/www/', import.meta.url));

export type Theme = 'light' | 'dark';

const TYPES: Record<string, string> = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.woff2': 'font/woff2',
  '.json': 'application/json',
};

function file(path: string): { body: Buffer; type: string } {
  const p = join(WWW, path);
  const hit = path !== '/' && existsSync(p) && statSync(p).isFile();
  const f = hit ? p : join(WWW, 'index.html');
  if (!existsSync(f)) throw new Error(`no app build in ${WWW}: run task ui:build`);
  return { body: readFileSync(f), type: TYPES[extname(f)] ?? 'application/octet-stream' };
}

/** A channel that opens and stays open, and never sends unless a test pushes a message. */
function streamStub() {
  type Listener = (e: MessageEvent | Event) => void;
  class StubSource {
    static readonly CONNECTING = 0;
    static readonly OPEN = 1;
    static readonly CLOSED = 2;
    readyState = 0;
    url: string;
    onopen: Listener | null = null;
    onmessage: Listener | null = null;
    onerror: Listener | null = null;
    private listeners = new Map<string, Listener[]>();
    constructor(url: string) {
      this.url = url;
      setTimeout(() => {
        this.readyState = 1;
        this.fire('open', new Event('open'));
      }, 0);
    }
    addEventListener(type: string, l: Listener) {
      this.listeners.set(type, [...(this.listeners.get(type) ?? []), l]);
    }
    removeEventListener(type: string, l: Listener) {
      this.listeners.set(type, (this.listeners.get(type) ?? []).filter((x) => x !== l));
    }
    close() {
      this.readyState = 2;
    }
    private fire(type: string, e: Event) {
      const own = (this as unknown as Record<string, Listener | null>)[`on${type}`];
      if (own) own(e);
      for (const l of this.listeners.get(type) ?? []) l(e);
    }
  }
  (window as unknown as { EventSource: unknown }).EventSource = StubSource;
}

/** Opens `path` in `theme`, answered from the fixtures. */
export async function open(page: Page, path: string, theme: Theme): Promise<void> {
  await page.clock.setFixedTime(FIXTURE_NOW);
  await page.addInitScript(streamStub);
  await page.addInitScript((t) => localStorage.setItem('sessionspec.theme', t), theme);
  await page.route('**/*', async (route) => {
    const url = new URL(route.request().url());
    if (url.origin !== ORIGIN) return route.abort();
    if (url.pathname.startsWith('/view/views/')) {
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(answer(url.pathname + url.search)) });
    }
    const f = file(url.pathname);
    return route.fulfill({ status: 200, contentType: f.type, body: f.body });
  });
  await page.goto(ORIGIN + path);
}
