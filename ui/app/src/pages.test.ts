// The pages over the synthetic fixtures of ui/fixtures, with the views answered from the files
// and a channel the test drives.
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushPromises, mount } from '@vue/test-utils';
import { answer, fixtures, FIXTURE_NOW } from "./testing/fixtures.ts";
import Overview from './pages/Overview.vue';
import SessionPage from './pages/SessionPage.vue';
import { lanesOf } from './session.ts';
import type { SessionTimelineRow } from "./generated/types.ts";

const sessions = { rows: fixtures.sessions };
const agents = { rows: fixtures.agents };
const checks = { rows: fixtures.checks };

type Listener = (e: { data: string }) => void;
const sources: FakeEventSource[] = [];
class FakeEventSource {
  listeners = new Map<string, Listener[]>();
  onopen: (() => void) | null = null;
  onerror: (() => void) | null = null;
  readyState = 1;
  constructor(readonly url: string) {
    sources.push(this);
  }
  addEventListener(t: string, l: Listener) {
    this.listeners.set(t, [...(this.listeners.get(t) ?? []), l]);
  }
  emit(t: string, payload: unknown) {
    for (const l of this.listeners.get(t) ?? []) l({ data: JSON.stringify(payload) });
  }
  close() {}
}

beforeEach(() => {
  sources.length = 0;
  vi.stubGlobal('EventSource', FakeEventSource);
  vi.stubGlobal(
    'fetch',
    vi.fn(async (url: string) => ({ ok: true, status: 200, json: async () => answer(url) })),
  );
  // Pin "now" just after the newest fixture session, so the 72 h slice holds all of them.
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(FIXTURE_NOW);
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('overview', () => {
  it('lists the sessions of the slice with their figures', async () => {
    const w = mount(Overview);
    await flushPromises();
    const rows = w.findAll('tbody tr');
    expect(rows).toHaveLength(sessions.rows.length);
    expect(w.text()).toContain('Port the store');
    expect(w.findAll('.tile')).toHaveLength(6);
    expect(w.text()).toContain(`${sessions.rows.length} sessions in the last 72 h`);
  });

  it('takes a changed session from the sessions channel', async () => {
    const w = mount(Overview);
    await flushPromises();
    const ch = sources.find((s) => s.url === '/view/channels/sessions')!;
    const changed = { ...sessions.rows[0], title: 'Renamed live', last_at: new Date(Date.now()).toISOString() };
    ch.emit('agentspec.view.SessionUpdated', changed);
    await flushPromises();
    expect(w.text()).toContain('Renamed live');
    expect(w.findAll('tbody tr')).toHaveLength(sessions.rows.length);
  });
});

describe('session page', () => {
  const id = sessions.rows[0].session_id;

  it('shows the header, a lane per agent, the checks and the events', async () => {
    const w = mount(SessionPage, { props: { sessionId: id } });
    await flushPromises();
    expect(w.find('h1').text()).toBe(sessions.rows[0].title);
    const mine = agents.rows.filter((a) => a.session_id === id);
    expect(w.findAll('.lane-name')).toHaveLength(mine.length);
    expect(w.findAll('.checks li').length).toBe(checks.rows.filter((c) => c.session_id === id).length);
    expect(w.findAll('.log tbody tr').length).toBeGreaterThan(0);
  });

  it('appends an event from the session_events channel', async () => {
    const w = mount(SessionPage, { props: { sessionId: id } });
    await flushPromises();
    const before = w.findAll('.log tbody tr').length;
    const ch = sources.find((s) => s.url.startsWith('/view/channels/session_events'))!;
    const main = agents.rows.find((a) => a.session_id === id && a.role === 'Main')!;
    ch.emit('agentspec.view.EventAppended', {
      event_id: `${id}.900`,
      session_id: id,
      seq: 900,
      ord: 900,
      notable: true,
      at: new Date(Date.now()).toISOString(),
      agent: main.agent_id,
      kind: 'tool_call',
      label: 'Bash',
      detail: 'Execute',
      agent_cost_usd: 1,
      session_cost_usd: 1,
    });
    await flushPromises();
    expect(w.findAll('.log tbody tr').length).toBe(before + 1);
  });

  it('an embed page has no way back to the overview', async () => {
    const w = mount(SessionPage, { props: { sessionId: id, embed: true, part: 'events' } });
    await flushPromises();
    expect(w.find('.crumbs').exists()).toBe(false);
    expect(w.find('.timeline').exists()).toBe(false);
    expect(w.find('.log').exists()).toBe(true);
  });
});

describe('lanes', () => {
  it('orders the main agent first and names sub-agents by start', () => {
    const id = sessions.rows[0].session_id;
    const slots = answer(`/view/views/session-timeline?session_id=${id}&slot_seconds=1`).rows as SessionTimelineRow[];
    const lanes = lanesOf(
      agents.rows.filter((a) => a.session_id === id),
      slots,
    );
    expect(lanes[0].agent.role).toBe('Main');
    expect(lanes[0].name).toBe('Main agent');
    expect(lanes.slice(1).every((l, i) => l.name === `Sub-agent ${i + 1}` || l.name.startsWith('Side call'))).toBe(true);
  });
});
