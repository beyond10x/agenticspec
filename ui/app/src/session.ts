// One session's state for its pages: the session row, its agents, its checks and its timeline
// slots, read once, then kept current by the `session_events` channel. The events themselves are
// read a page at a time by the event log (EventLog.vue); the server holds them.

import { computed, onUnmounted, ref, shallowRef, watch, type Ref } from 'vue';
import { query, subscribe, upsert, type LiveState, type SourceFactory, type Fetch } from './client.ts';
import type {
  EventAppended,
  SessionAgentsRow,
  SessionByIdRow,
  SessionChecksRow,
  SessionTimelineRow,
} from './generated/types.ts';
import { ms } from './stats.ts';

export interface AgentLane {
  agent: SessionAgentsRow;
  /** `Main agent`, `Sub-agent 1` … in order of start, `Side call 1` … */
  name: string;
  slots: SessionTimelineRow[];
  first: number;
  last: number;
}

export interface SessionState {
  session: Ref<SessionByIdRow | null>;
  agents: Ref<SessionAgentsRow[]>;
  slots: Ref<SessionTimelineRow[]>;
  checks: Ref<SessionChecksRow[]>;
  loading: Ref<boolean>;
  error: Ref<string | null>;
  live: Ref<LiveState>;
  lanes: Ref<AgentLane[]>;
  /** The last event the channel brought, for the event log to put on top. */
  appended: Ref<EventAppended | null>;
}

const STEPS = [1, 2, 5, 10, 15, 30, 60, 120, 300, 600, 900, 1800, 3600, 7200];

/** The slot length for a session of `wallSeconds`: about 1,200 slots over the whole of it. */
export function slotSeconds(wallSeconds: number): number {
  const want = wallSeconds / 1200;
  return STEPS.find((s) => s >= want) ?? STEPS[STEPS.length - 1];
}

export const CALL_FIELDS = [
  ['Read', 'read_calls'],
  ['Search', 'search_calls'],
  ['Edit', 'edit_calls'],
  ['Execute', 'execute_calls'],
  ['Network', 'network_calls'],
  ['Spawn', 'spawn_calls'],
  ['Message', 'message_calls'],
  ['Plan', 'plan_calls'],
  ['Other', 'other_calls'],
] as const;

export type CallField = (typeof CALL_FIELDS)[number][1];

export function callsOf(s: SessionTimelineRow): number {
  return CALL_FIELDS.reduce((a, [, f]) => a + s[f], 0);
}

/** The category with the most calls in a slot, or null without a call. */
export function topCategory(s: SessionTimelineRow): string | null {
  let best: string | null = null;
  let n = 0;
  for (const [cat, f] of CALL_FIELDS) {
    if (s[f] > n) {
      best = cat;
      n = s[f];
    }
  }
  return best;
}

function emptySlot(e: EventAppended, width: number): SessionTimelineRow {
  const start = Math.floor(ms(e.at) / (width * 1000)) * width * 1000;
  return {
    slot_id: `${e.session_id}.${e.agent}.${start}`,
    session_id: e.session_id,
    agent: e.agent,
    slot_seconds: width,
    start: e.at,
    end: e.at,
    events: 0,
    read_calls: 0,
    search_calls: 0,
    edit_calls: 0,
    execute_calls: 0,
    network_calls: 0,
    spawn_calls: 0,
    message_calls: 0,
    plan_calls: 0,
    other_calls: 0,
    human_prompts: 0,
    relayed_prompts: 0,
    refusals: 0,
    busts: 0,
    compactions: 0,
    agent_cost_usd: 0,
    session_cost_usd: 0,
  };
}

/** The slots with one more event counted in, as the server would count it. */
export function addEvent(slots: SessionTimelineRow[], e: EventAppended, width: number): SessionTimelineRow[] {
  if (e.kind === 'agent_started') return slots;
  const fresh = emptySlot(e, width);
  const i = slots.findIndex((s) => s.slot_id === fresh.slot_id);
  const s = { ...(i >= 0 ? slots[i] : fresh) };
  s.events += 1;
  if (ms(e.at) < ms(s.start)) s.start = e.at;
  if (ms(e.at) > ms(s.end)) s.end = e.at;
  if (e.kind === 'tool_call') {
    const f = CALL_FIELDS.find(([c]) => c === e.detail)?.[1] ?? 'other_calls';
    s[f] += 1;
  } else if (e.kind === 'prompt' && e.label === 'Human') s.human_prompts += 1;
  else if (e.kind === 'prompt' && (e.label === 'Delegate' || e.label === 'PeerSession')) s.relayed_prompts += 1;
  else if (e.kind === 'decision' && e.ok === false) s.refusals += 1;
  else if (e.kind === 'cache_bust') s.busts += 1;
  else if (e.kind === 'compaction') s.compactions += 1;
  if (e.kind === 'model_turn' && e.context_tokens !== undefined) s.context_max = Math.max(s.context_max ?? 0, e.context_tokens);
  s.agent_cost_usd = Math.max(s.agent_cost_usd, e.agent_cost_usd);
  s.session_cost_usd = Math.max(s.session_cost_usd, e.session_cost_usd);
  if (i >= 0) {
    const next = slots.slice();
    next[i] = s;
    return next;
  }
  return [...slots, s];
}

export function lanesOf(agents: SessionAgentsRow[], slots: SessionTimelineRow[]): AgentLane[] {
  const byAgent = new Map<string, SessionTimelineRow[]>();
  for (const s of slots) {
    const list = byAgent.get(s.agent);
    if (list) list.push(s);
    else byAgent.set(s.agent, [s]);
  }
  const known = new Set(agents.map((a) => a.agent_id));
  const all: SessionAgentsRow[] = [...agents];
  for (const [id, list] of byAgent) {
    if (!known.has(id)) {
      all.push({
        agent_id: id,
        session_id: list[0].session_id,
        role: 'SubAgent',
        started_at: list[0].start,
        turns: 0,
        context_tokens: 0,
        context_max: 0,
        output_tokens: 0,
        cost_usd: 0,
        tool_calls: 0,
        busts: 0,
      });
    }
  }
  const lanes = all.map((agent) => {
    const own = (byAgent.get(agent.agent_id) ?? []).slice().sort((a, b) => ms(a.start) - ms(b.start));
    const first = own.length ? ms(own[0].start) : ms(agent.started_at);
    const last = own.length ? Math.max(...own.map((s) => ms(s.end))) : first;
    return { agent, name: '', slots: own, first, last };
  });
  lanes.sort((a, b) => {
    const ra = a.agent.role === 'Main' ? 0 : a.agent.role === 'SubAgent' ? 1 : 2;
    const rb = b.agent.role === 'Main' ? 0 : b.agent.role === 'SubAgent' ? 1 : 2;
    return ra - rb || a.first - b.first;
  });
  let sub = 0;
  let side = 0;
  for (const l of lanes) {
    if (l.agent.role === 'Main') l.name = 'Main agent';
    else if (l.agent.role === 'SubAgent') l.name = `Sub-agent ${++sub}`;
    else l.name = `Side call ${++side}`;
  }
  return lanes;
}

export function useSession(
  id: Ref<string>,
  deps: { fetch?: Fetch; open?: SourceFactory } = {},
): SessionState {
  const session = ref<SessionByIdRow | null>(null);
  const agents = shallowRef<SessionAgentsRow[]>([]);
  const slots = shallowRef<SessionTimelineRow[]>([]);
  const checks = shallowRef<SessionChecksRow[]>([]);
  const appended = shallowRef<EventAppended | null>(null);
  const loading = ref(true);
  const error = ref<string | null>(null);
  const live = ref<LiveState>('connecting');
  let close: (() => void) | null = null;
  let controller: AbortController | null = null;
  let width = 1;

  async function load(sessionId: string) {
    close?.();
    controller?.abort();
    controller = new AbortController();
    const opts = { fetch: deps.fetch, signal: controller.signal };
    loading.value = true;
    error.value = null;
    try {
      const [s, a, c] = await Promise.all([
        query('session', { session_id: sessionId }, opts),
        query('session-agents', { session_id: sessionId }, opts),
        query('session-checks', { session_id: sessionId }, opts),
      ]);
      session.value = s[0] ?? null;
      agents.value = a;
      checks.value = c;
      if (!s[0]) {
        error.value = 'No session with this id is in the store.';
      } else {
        width = slotSeconds((ms(s[0].last_at) - ms(s[0].first_at)) / 1000);
        slots.value = await query('session-timeline', { session_id: sessionId, slot_seconds: width }, opts);
      }
    } catch (err) {
      if ((err as Error).name === 'AbortError') return;
      error.value = (err as Error).message;
    } finally {
      loading.value = false;
    }
    close = subscribe(
      'session_events',
      { session_id: sessionId },
      {
        'sessionspec.view.EventAppended': (p) => {
          slots.value = addEvent(slots.value, p, width);
          appended.value = p;
        },
        'sessionspec.view.AgentUpdated': (p) => {
          agents.value = upsert(agents.value, p, 'agent_id');
        },
        'sessionspec.view.SessionUpdated': (p) => {
          if (session.value) session.value = { ...session.value, ...p };
        },
      },
      (st) => (live.value = st),
      deps.open,
    );
  }

  watch(id, (v) => void load(v), { immediate: true });
  onUnmounted(() => {
    close?.();
    controller?.abort();
  });

  const lanes = computed(() => lanesOf(agents.value, slots.value));
  return { session, agents, slots, checks, loading, error, live, lanes, appended };
}
