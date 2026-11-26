import { describe, expect, it } from 'vitest';
import sessions from '../../fixtures/sessions.json';
import { query, subscribe, upsert, viewUrl, ViewError, type Source } from './client.ts';

class FakeSource implements Source {
  listeners = new Map<string, ((e: { data: string }) => void)[]>();
  onopen: (() => void) | null = null;
  onerror: (() => void) | null = null;
  readyState = 0;
  closed = false;
  constructor(readonly url: string) {}
  addEventListener(type: string, l: (e: { data: string }) => void) {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), l]);
  }
  emit(type: string, data: unknown) {
    for (const l of this.listeners.get(type) ?? []) l({ data: typeof data === 'string' ? data : JSON.stringify(data) });
  }
  close() {
    this.closed = true;
  }
}

describe('client', () => {
  it('names a view by its wire name and leaves out empty parameters', () => {
    expect(viewUrl('session-list', { since: '2026-11-20T00:00:00Z', search: '', limit: 100 })).toBe(
      '/view/views/session-list?since=2026-11-20T00%3A00%3A00Z&limit=100',
    );
    expect(viewUrl('session-events', { session_id: 'ses demo' })).toBe('/view/views/session-events?session_id=ses+demo');
  });

  it('reads the rows of a view', async () => {
    let asked = '';
    const rows = await query('session-list', {}, {
      fetch: async (url) => {
        asked = url;
        return { ok: true, status: 200, json: async () => sessions };
      },
    });
    expect(asked).toBe('/view/views/session-list');
    expect(rows.map((r) => r.session_id)).toEqual(sessions.rows.map((r) => r.session_id));
  });

  it('refuses an answer that is not 2xx', async () => {
    await expect(
      query('session', { session_id: 'x' }, { fetch: async () => ({ ok: false, status: 503, json: async () => ({}) }) }),
    ).rejects.toBeInstanceOf(ViewError);
  });

  it('dispatches channel events by qualified name and reports its state', async () => {
    let src: FakeSource | null = null;
    const got: string[] = [];
    const states: string[] = [];
    const close = subscribe(
      'session_events',
      { session_id: 'ses-demo-0001' },
      {
        'agenticspec.view.EventAppended': (p) => got.push(`${p.kind}:${p.seq}`),
        'agenticspec.view.AgentUpdated': (p) => got.push(`agent:${p.agent_id}`),
      },
      (s) => states.push(s),
      (url) => (src = new FakeSource(url)),
    );
    const s = src as unknown as FakeSource;
    expect(s.url).toBe('/view/channels/session_events?session_id=ses-demo-0001');
    s.onopen?.();
    s.emit('agenticspec.view.EventAppended', { seq: 26, kind: 'prompt' });
    s.emit('agenticspec.view.AgentUpdated', { agent_id: 'a1' });
    s.emit('agenticspec.view.EventAppended', 'not json');
    s.emit('agenticspec.view.SessionUpdated', { session_id: 'ignored, no handler' });
    expect(got).toEqual(['prompt:26', 'agent:a1']);
    expect(states).toEqual(['connecting', 'live']);
    close();
    expect(s.closed).toBe(true);
  });

  it('upserts by key', () => {
    const rows = [
      { id: 'a', n: 1 },
      { id: 'b', n: 2 },
    ];
    expect(upsert(rows, { id: 'b', n: 3 }, 'id')).toEqual([
      { id: 'a', n: 1 },
      { id: 'b', n: 3 },
    ]);
    expect(upsert(rows, { id: 'c', n: 4 }, 'id')).toHaveLength(3);
    expect(rows[1].n).toBe(2);
  });
});
