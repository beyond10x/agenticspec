// Unit tests of the package of ui/views: a panel mounted in a Vue app with a fixture source shows
// the fixture's rows, the panels are found by view and by block, a part page drawn from a layout
// sends `agentic-command` for its action, and installing keeps the host's title.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createApp, nextTick } from 'vue';
import { AgenticPanel, AgenticPartPage, elementName, installViews, panelName, panelsOfBlock, PANELS, type Layout } from '../src/index';
import layoutJson from '../examples/workstation.layout.json';

interface Fixture {
  view: string;
  rows: Record<string, unknown>[];
}
const fixtures = Object.values(import.meta.glob<Fixture>('../fixtures/*.json', { eager: true, import: 'default' }));
const rowsOf = (view: string) => fixtures.find((f) => f.view === view)?.rows ?? [];

/** A fetch that answers `<source>/api/views/<view>` from the panels' fixtures. */
function fixtureFetch(source: string) {
  return vi.fn(async (input: RequestInfo | URL) => {
    const url = String(input);
    const m = url.match(/\/api\/views\/([\w.]+)/);
    if (!url.startsWith(source) || !m) return new Response('{}', { status: 404 });
    return new Response(JSON.stringify({ rows: rowsOf(m[1]) }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  });
}

async function settle(): Promise<void> {
  for (let i = 0; i < 5; i++) {
    await new Promise((r) => setTimeout(r, 0));
    await nextTick();
  }
}

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.innerHTML = '';
});

describe('the package of ui/views', () => {
  it('mounts a block panel with a fixture source and shows its rows', async () => {
    vi.stubGlobal('fetch', fixtureFetch('http://fixtures.test'));
    vi.stubGlobal('EventSource', class { addEventListener() {} close() {} });
    const el = document.createElement('div');
    document.body.append(el);
    const app = createApp(AgenticPanel, { view: 'agentic.agent.Agents', source: 'http://fixtures.test', args: { channel: 'level' } });
    installViews(app);
    app.mount(el);
    await settle();
    const keys = [...el.querySelectorAll('tr[data-row]')].map((r) => r.getAttribute('data-row'));
    const want = rowsOf('agentic.agent.Agents').map((r) => String(r.agent_id));
    expect(want.length).toBeGreaterThan(0);
    expect(keys).toEqual(expect.arrayContaining(want));
    app.unmount();
  });

  it('finds a panel by its view and the panels of a block', () => {
    expect(panelName('agentic.harness.Turns')).toBe('AgenticHarnessTurns');
    expect(PANELS.AgenticHarnessTurns).toBeTruthy();
    expect(elementName('AgenticHarnessTurns')).toBe('agentic-harness-turns');
    expect(panelsOfBlock('agent')).toContain('AgenticAgentAgents');
    expect(panelsOfBlock('pool')).toContain('AgenticResourcePools');
    expect(panelsOfBlock('workspace')).not.toContain('AgenticWorkRuns');
  });

  it('draws a part page from a layout and sends agentic-command for its action', async () => {
    vi.stubGlobal('fetch', fixtureFetch('http://fixtures.test'));
    vi.stubGlobal('EventSource', class { addEventListener() {} close() {} });
    const el = document.createElement('div');
    document.body.append(el);
    const seen: unknown[] = [];
    el.addEventListener('agentic-command', (e) => seen.push((e as CustomEvent).detail));
    const app = createApp(AgenticPartPage, {
      source: 'http://fixtures.test',
      architecture: 'workstation',
      part: 'operator',
      layout: layoutJson as unknown as Layout,
    });
    installViews(app);
    app.mount(el);
    await settle();
    expect(el.querySelector('h2.part-title')?.textContent).toBe('operator');
    const button = [...el.querySelectorAll('button')].find((b) => b.textContent?.includes('Send a message'));
    button?.click();
    expect(seen).toEqual([expect.objectContaining({ command: 'agentic.channel.SendMessage', part: 'operator', architecture: 'workstation' })]);
    app.unmount();
  });

  it('keeps the host title and sets a theme only when the host has none', () => {
    document.title = 'Host';
    document.documentElement.dataset.theme = 'dark';
    installViews(createApp({ render: () => null }));
    expect(document.title).toBe('Host');
    expect(document.documentElement.dataset.theme).toBe('dark');
  });
});
