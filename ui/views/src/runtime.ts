// The runtime the panels draw with: the one generated beside them (ui/views/gallery/src/generated),
// installed with no pages, no routes and no navigation of its own, so a host keeps its own.
// Everything the generated runtime and client export is exported again, so what a later
// generation adds (such as the actions of a scope) reaches the package's users unchanged.
import type { App } from 'vue';
import { install, type CommandHooks } from '../gallery/src/generated/runtime';
import { createClient, type Client } from '../gallery/src/generated/client';

export * from '../gallery/src/generated/runtime';
export * from '../gallery/src/generated/client';

export interface ViewsOptions {
  /** The client the panels read through when a panel names no `source`. */
  client?: Client;
  /** The base URL of the server that answers the panels, when no `client` is given. */
  source?: string;
  hooks?: CommandHooks;
}

/** The theme the kit's tokens are written for: the host's, else the system's. */
export function ensureTheme(): void {
  const html = document.documentElement;
  if (html.dataset.theme) return;
  const dark = typeof matchMedia === 'function' && matchMedia('(prefers-color-scheme: dark)').matches;
  html.dataset.theme = dark ? 'dark' : 'light';
}

/** Installs the panels' runtime in `app`; the document's title stays the host's. */
export function installViews(app: App, options: ViewsOptions = {}): void {
  const title = document.title;
  install(app, {
    app: 'agentic-views',
    title,
    home: '',
    routes: [],
    client: options.client ?? createClient((options.source ?? '').replace(/\/+$/, '')),
    hosts: {},
    hooks: options.hooks ?? {},
    shell: {},
    theme: null,
  });
  document.title = title;
  ensureTheme();
}
