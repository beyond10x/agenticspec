// The typed client over the `session-view` surface that `agentspec serve` answers. The row and
// event types are generated from spec/domains/view.yaml (src/generated/types.ts, never edited);
// the paths are the views' wire names and the two channels of ui/agentspec.ui.yaml.

import type {
  AgentUpdated,
  EventAppended,
  SessionAgentsRow,
  SessionByIdRow,
  OverviewFiguresRow,
  SessionChecksRow,
  SessionEventCountsRow,
  SessionEventsRow,
  SessionListRow,
  SessionTimelineRow,
  SessionTurnsRow,
  SessionUpdated,
  EventKind,
} from './generated/types.ts';

export type Fetch = (url: string, init?: { signal?: AbortSignal }) => Promise<{ ok: boolean; status: number; json(): Promise<unknown> }>;

/** The views by wire name, with their parameters and rows. */
export interface Views {
  'session-list': { params: { since?: string; search?: string; limit?: number; offset?: number }; row: SessionListRow };
  session: { params: { session_id: string }; row: SessionByIdRow };
  'session-agents': { params: { session_id: string }; row: SessionAgentsRow };
  'session-events': {
    params: { session_id: string; agent?: string; kind?: EventKind; notable?: boolean; before?: number };
    row: SessionEventsRow;
  };
  'session-event-counts': { params: { session_id: string; agent?: string }; row: SessionEventCountsRow };
  'session-timeline': { params: { session_id: string; slot_seconds: number }; row: SessionTimelineRow };
  figures: { params: { since: string; bucket_seconds: number }; row: OverviewFiguresRow };
  'session-turns': { params: { session_id: string }; row: SessionTurnsRow };
  'session-checks': { params: { session_id: string }; row: SessionChecksRow };
}

export class ViewError extends Error {
  constructor(
    readonly view: string,
    readonly status: number,
  ) {
    super(`the view ${view} answered ${status}`);
  }
}

export function viewUrl<V extends keyof Views>(view: V, params: Views[V]['params'], base = ''): string {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null && v !== '') q.set(k, String(v));
  }
  const s = q.toString();
  return `${base}/view/views/${view}${s ? `?${s}` : ''}`;
}

export async function query<V extends keyof Views>(
  view: V,
  params: Views[V]['params'],
  opts: { fetch?: Fetch; signal?: AbortSignal; base?: string } = {},
): Promise<Views[V]['row'][]> {
  const f: Fetch = opts.fetch ?? ((u, i) => fetch(u, i));
  const res = await f(viewUrl(view, params, opts.base), { signal: opts.signal });
  if (!res.ok) throw new ViewError(view, res.status);
  const body = (await res.json()) as { rows?: Views[V]['row'][] };
  return body.rows ?? [];
}

/** The events a channel carries, by qualified name. */
export interface ChannelEvents {
  'agentspec.view.SessionUpdated': SessionUpdated;
  'agentspec.view.AgentUpdated': AgentUpdated;
  'agentspec.view.EventAppended': EventAppended;
}

export type Handlers = { [K in keyof ChannelEvents]?: (payload: ChannelEvents[K]) => void };

export type LiveState = 'connecting' | 'live' | 'offline';

/** The part of `EventSource` a channel uses, so a test can pass its own. */
export interface Source {
  addEventListener(type: string, listener: (e: { data: string }) => void): void;
  onopen: (() => void) | null;
  onerror: (() => void) | null;
  readyState: number;
  close(): void;
}

export type SourceFactory = (url: string) => Source;

const browserSource: SourceFactory = (url) => new EventSource(url) as unknown as Source;

/**
 * Subscribes to a channel: `sessions`, or `session_events` of one session. `EventSource`
 * reconnects by itself and sends `Last-Event-ID`, so the server resends what followed. Returns
 * the function that closes it.
 */
export function subscribe(
  channel: 'sessions' | 'session_events',
  params: { session_id?: string },
  handlers: Handlers,
  onState: (s: LiveState) => void = () => {},
  open: SourceFactory = browserSource,
): () => void {
  const q = params.session_id ? `?session_id=${encodeURIComponent(params.session_id)}` : '';
  const src = open(`/view/channels/${channel}${q}`);
  onState('connecting');
  src.onopen = () => onState('live');
  src.onerror = () => onState(src.readyState === 2 ? 'offline' : 'connecting');
  for (const name of Object.keys(handlers) as (keyof ChannelEvents)[]) {
    const h = handlers[name] as ((p: unknown) => void) | undefined;
    if (!h) continue;
    src.addEventListener(name, (e) => {
      try {
        h(JSON.parse(e.data));
      } catch {
        // A message that is no JSON is dropped; the next view read repairs the state.
      }
    });
  }
  return () => src.close();
}

/** Replaces the row with the same key, or adds it. */
export function upsert<T, K extends keyof T>(rows: T[], row: T, key: K): T[] {
  const i = rows.findIndex((r) => r[key] === row[key]);
  if (i < 0) return [...rows, row];
  const next = rows.slice();
  next[i] = { ...rows[i], ...row };
  return next;
}
