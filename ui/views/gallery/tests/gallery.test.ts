// Unit tests of the generated block panels: a widget re-reads once per coalesce window however many
// events came in it, patches or inserts a row on insert_or_patch, drops an event outside its
// parameters, reads through the client of its scope, and every view of the fixtures answers.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createApp, h, nextTick, type Component } from 'vue';
import { install } from '../src/generated/runtime';
import { createClient, type Client, type Row, type Value } from '../src/generated/client';
import { fixtureClient, offset, scripts, views } from '../src/fixtures';
import Principals from '../src/generated/widgets/AgenticPrincipalPrincipals.vue';
import Turns from '../src/generated/widgets/AgenticHarnessTurns.vue';
import Directions from '../src/generated/widgets/AgenticKpiDirections.vue';

type Emit = (event: string, data: Value) => void;

/** A client that counts its reads and hands the test the channel's emitter. */
function fake(rows: Row[]) {
  const state = { reads: 0, emit: null as Emit | null, scopes: [] as string[], subscriptions: 0 };
  const client: Client = {
    async read() {
      state.reads++;
      return rows.map((r) => ({ ...r }));
    },
    async command() {
      return { ok: true, row: null };
    },
    subscribe(_channel, _params, onEvent) {
      state.subscriptions++;
      state.emit = onEvent;
      return () => undefined;
    },
    async actions() {
      return null;
    },
    scope(path) {
      state.scopes.push(path);
      return client;
    },
  };
  return { client, state };
}

const mounted: { unmount(): void }[] = [];

function mount(component: Component, props: Record<string, unknown>, client: Client): HTMLElement {
  const app = createApp({ render: () => h(component, props) });
  install(app, { app: 't', title: 't', home: 'p', routes: [], client, hosts: {}, hooks: {}, shell: {}, theme: null });
  const el = document.createElement('div');
  document.body.append(el);
  app.mount(el);
  mounted.push(app);
  return el;
}

async function settle(): Promise<void> {
  for (let i = 0; i < 5; i++) {
    await Promise.resolve();
    await nextTick();
  }
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  while (mounted.length) mounted.pop()?.unmount();
  vi.useRealTimers();
});

describe('live re-read', () => {
  it('re-reads once for three matching events within the coalesce window', async () => {
    const { client, state } = fake([{ principal_id: 'p1', name: 'one', state: 'Active' }]);
    const el = mount(Principals, { channel: 'principal' }, client);
    await settle();
    expect(state.reads).toBe(1);
    expect(el.querySelectorAll('tr[data-row]')).toHaveLength(1);
    state.emit?.('agentic.principal.PrincipalRegistered', { principal_id: 'p2' });
    vi.advanceTimersByTime(300);
    state.emit?.('agentic.principal.PrincipalRetired', { principal_id: 'p1' });
    vi.advanceTimersByTime(300);
    state.emit?.('agentic.principal.PrincipalRegistered', { principal_id: 'p3' });
    await settle();
    expect(state.reads).toBe(1);
    vi.advanceTimersByTime(400);
    await settle();
    expect(state.reads).toBe(2);
    vi.advanceTimersByTime(5000);
    await settle();
    expect(state.reads).toBe(2);
  });

  it('ignores an event the panel does not take', async () => {
    const { client, state } = fake([{ principal_id: 'p1' }]);
    mount(Principals, { channel: 'principal' }, client);
    await settle();
    state.emit?.('agentic.delegation.DelegationGranted', { delegation_id: 'd1' });
    vi.advanceTimersByTime(2000);
    await settle();
    expect(state.reads).toBe(1);
  });

  it('two panels on one channel share one subscription', async () => {
    const { client, state } = fake([{ principal_id: 'p1' }]);
    mount(Principals, { channel: 'principal' }, client);
    mount(Principals, { channel: 'principal' }, client);
    await settle();
    expect(state.subscriptions).toBe(1);
  });
});

describe('live patch', () => {
  it('patches the row the event matches and puts a new one first', async () => {
    const { client, state } = fake([
      { turn_id: 't1', context_tokens: 1 },
      { turn_id: 't2', context_tokens: 2 },
    ]);
    const el = mount(Turns, { channel: 'harness' }, client);
    await settle();
    state.emit?.('agentic.harness.TurnRecorded', { turn_id: 't2', context_tokens: 20 });
    await settle();
    expect(el.querySelector('tr[data-row="t2"] [data-field="context_tokens"]')?.textContent?.trim()).toBe('20');
    state.emit?.('agentic.harness.TurnRecorded', { turn_id: 't3', context_tokens: 3 });
    await settle();
    const keys = [...el.querySelectorAll('tr[data-row]')].map((r) => r.getAttribute('data-row'));
    expect(keys).toEqual(['t3', 't1', 't2']);
    expect(state.reads).toBe(1);
  });

  it('drops an event whose parameter field differs (only_if: matches(params))', async () => {
    const { client, state } = fake([{ direction_id: 'd1', work_mode_id: 'w1', direction: 'Higher' }]);
    const el = mount(Directions, { channel: 'kpi', work_mode_id: 'w1' }, client);
    await settle();
    state.emit?.('agentic.kpi.DirectionSet', { direction_id: 'd2', work_mode_id: 'w2', direction: 'Lower' });
    await settle();
    expect(el.querySelectorAll('tr[data-row]')).toHaveLength(1);
    state.emit?.('agentic.kpi.DirectionSet', { direction_id: 'd3', work_mode_id: 'w1', direction: 'Lower' });
    await settle();
    expect(el.querySelectorAll('tr[data-row]')).toHaveLength(2);
  });
});

describe('scope', () => {
  it('reads under /scopes/<architecture>/<part>/<instance> of the app client', async () => {
    const { client, state } = fake([{ principal_id: 'p1' }]);
    mount(Principals, { channel: 'principal', architecture: 'a1', part: 'team.lead', instance: 2 }, client);
    await settle();
    expect(state.scopes).toEqual(['/scopes/a1/team.lead/2']);
  });

  it('reads from the adapter at source, which answers the scope', async () => {
    const fetcher = vi.fn(async (_url: string, _init?: RequestInit) => new Response(JSON.stringify({ rows: [{ principal_id: 'p9' }] }), { status: 200, headers: { 'Content-Type': 'application/json' } }));
    vi.stubGlobal('fetch', fetcher);
    vi.stubGlobal(
      'EventSource',
      class {
        addEventListener() {}
        close() {}
      },
    );
    const { client, state } = fake([]);
    const el = mount(Principals, { channel: 'principal', source: 'https://runtime.example/scopes/a1/' }, client);
    await settle();
    expect(fetcher).toHaveBeenCalledWith('https://runtime.example/scopes/a1/api/views/agentic.principal.Principals', { credentials: 'include' });
    expect(state.reads).toBe(0);
    expect(el.querySelector('tr[data-row="p9"]')).not.toBeNull();
    vi.unstubAllGlobals();
  });

  it('the generated client keeps createClient(base) and scopes below its base', () => {
    const c = createClient('https://x.example');
    expect(typeof c.scope('/scopes/a').read).toBe('function');
  });
});

describe('fixtures', () => {
  it('answer every view with rows and play a script per channel at its offsets', async () => {
    const client = fixtureClient(views, scripts);
    expect(views.length).toBeGreaterThanOrEqual(60);
    for (const v of views) {
      const rows = await client.read(v.view as never, {} as never);
      expect(rows.length, v.view).toBeGreaterThan(0);
    }
    const got: string[] = [];
    const stop = client.subscribe('harness', {}, (e) => got.push(e));
    vi.advanceTimersByTime(1000);
    expect(got).toHaveLength(1);
    vi.advanceTimersByTime(60_000);
    expect(got.length).toBe(scripts.find((s) => s.channel === 'harness')?.events.length);
    stop();
    expect(offset('2s')).toBe(2000);
    expect(offset('250ms')).toBe(250);
  });
});
