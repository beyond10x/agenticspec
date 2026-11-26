// The views of `sessionspec serve` answered from the synthetic fixtures of ui/fixtures, for the
// unit tests and the layout and user-bot tests: the same paging, filters and sums the server's
// queries apply (crates/sessionspec-records/src/store.rs), over a handful of rows.

import sessions from '../../../fixtures/sessions.json' with { type: 'json' };
import agents from '../../../fixtures/agents.json' with { type: 'json' };
import events from '../../../fixtures/events.json' with { type: 'json' };
import checks from '../../../fixtures/checks.json' with { type: 'json' };
import type {
  OverviewFiguresRow,
  SessionAgentsRow,
  SessionChecksRow,
  SessionEventCountsRow,
  SessionEventsRow,
  SessionListRow,
  SessionTimelineRow,
  EventKind,
} from '../generated/types.ts';

export const fixtures = {
  sessions: sessions.rows as SessionListRow[],
  agents: agents.rows as SessionAgentsRow[],
  events: events.rows as SessionEventsRow[],
  checks: checks.rows as SessionChecksRow[],
};

/** Just after the newest fixture session: the 72 h slice holds all of them. */
export const FIXTURE_NOW = Math.max(...fixtures.sessions.map((r) => Date.parse(r.last_at))) + 60_000;

const ms = (t: string) => Date.parse(t);
const CATS = ['Read', 'Search', 'Edit', 'Execute', 'Network', 'Spawn', 'Message', 'Plan'];

function ranked(): SessionListRow[] {
  return fixtures.sessions
    .slice()
    .sort((a, b) => ms(b.last_at) - ms(a.last_at) || a.session_id.localeCompare(b.session_id))
    .map((r, i) => ({ ...r, rank: i + 1 }));
}

function timeline(id: string, width: number): SessionTimelineRow[] {
  const w = Math.max(1, width) * 1000;
  const slots = new Map<string, SessionTimelineRow>();
  const evs = fixtures.events.filter((e) => e.session_id === id && e.kind !== 'agent_started');
  for (const e of evs) {
    const start = Math.floor(ms(e.at) / w) * w;
    const key = `${e.agent}.${start}`;
    let s = slots.get(key);
    if (!s) {
      s = {
        slot_id: `${id}.${key}`,
        session_id: id,
        agent: e.agent,
        slot_seconds: Math.max(1, width),
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
      slots.set(key, s);
    }
    s.events += 1;
    if (ms(e.at) < ms(s.start)) s.start = e.at;
    if (ms(e.at) > ms(s.end)) s.end = e.at;
    if (e.kind === 'tool_call') {
      const c = CATS.includes(e.detail ?? '') ? (e.detail as string).toLowerCase() : 'other';
      (s as unknown as Record<string, number>)[`${c}_calls`] += 1;
    }
    if (e.kind === 'prompt' && e.label === 'Human') s.human_prompts += 1;
    if (e.kind === 'prompt' && (e.label === 'Delegate' || e.label === 'PeerSession')) s.relayed_prompts += 1;
    if (e.kind === 'decision' && e.ok === false) s.refusals += 1;
    if (e.kind === 'cache_bust') s.busts += 1;
    if (e.kind === 'compaction') s.compactions += 1;
    if (e.kind === 'model_turn' && e.context_tokens !== undefined) s.context_max = Math.max(s.context_max ?? 0, e.context_tokens);
    s.agent_cost_usd = Math.max(s.agent_cost_usd, e.agent_cost_usd);
    s.session_cost_usd = Math.max(s.session_cost_usd, e.session_cost_usd);
  }
  return [...slots.values()].sort((a, b) => ms(a.start) - ms(b.start) || a.agent.localeCompare(b.agent));
}

function figures(since: number, bucketSeconds: number): OverviewFiguresRow[] {
  const w = bucketSeconds * 1000;
  const hour = 3_600_000;
  const by = new Map<number, OverviewFiguresRow>();
  for (const r of fixtures.sessions) {
    const h = Math.floor(ms(r.last_at) / hour) * hour;
    if (h < since) continue;
    const b = Math.floor((h - since) / w);
    const start = since + b * w;
    const x = by.get(b) ?? {
      bucket_id: String(start),
      start: new Date(start).toISOString(),
      bucket_seconds: bucketSeconds,
      sessions: 0,
      cost_usd: 0,
      working_seconds: 0,
      waiting_share: 0,
      cache_hit_share: 0,
      tool_calls: 0,
      refused_tool_calls: 0,
    };
    x.sessions += 1;
    x.cost_usd += r.cost_usd;
    x.working_seconds += r.working_seconds;
    x.waiting_share += r.waiting_share;
    x.cache_hit_share += r.cache_hit_share;
    x.tool_calls += r.tool_calls;
    x.refused_tool_calls += r.refused_tool_calls;
    by.set(b, x);
  }
  return [...by.entries()].sort((a, b) => a[0] - b[0]).map(([, v]) => v);
}

/** The JSON body the server answers for a view URL. */
export function answer(url: string): { rows: unknown[] } {
  const u = new URL(url, 'http://fixtures');
  const view = u.pathname.replace(/^\/view\/views\//, '');
  const p = (k: string) => u.searchParams.get(k) ?? undefined;
  const id = p('session_id');
  switch (view) {
    case 'session-list': {
      const since = p('since') ? ms(p('since')!) : -Infinity;
      const q = p('search')?.toLowerCase();
      const offset = Number(p('offset') ?? 0);
      const limit = p('limit') ? Number(p('limit')) : Infinity;
      return {
        rows: ranked().filter((r) => ms(r.last_at) >= since && (!q || r.names.includes(q)) && r.rank > offset && r.rank <= limit),
      };
    }
    case 'session':
      return { rows: ranked().filter((r) => r.session_id === id).map((r) => ({ ...r, kpis: kpisOf(r) })) };
    case 'session-agents':
      return { rows: fixtures.agents.filter((a) => a.session_id === id) };
    case 'session-checks':
      return { rows: fixtures.checks.filter((c) => c.session_id === id) };
    case 'session-events': {
      const agent = p('agent');
      const kind = p('kind') as EventKind | undefined;
      const notable = p('notable');
      const before = p('before') ? Number(p('before')) : Infinity;
      return {
        rows: fixtures.events
          .filter(
            (e) =>
              e.session_id === id &&
              (!agent || e.agent === agent) &&
              (!kind || e.kind === kind) &&
              (notable === undefined || String(e.notable) === notable) &&
              e.ord < before,
          )
          .sort((a, b) => b.ord - a.ord)
          .slice(0, 500),
      };
    }
    case 'session-event-counts': {
      const agent = p('agent');
      const c = new Map<EventKind, number>();
      for (const e of fixtures.events) {
        if (e.session_id !== id || (agent && e.agent !== agent)) continue;
        c.set(e.kind, (c.get(e.kind) ?? 0) + 1);
      }
      const rows: SessionEventCountsRow[] = [...c.entries()].map(([kind, count]) => ({
        count_id: `${id}.${agent ?? '*'}.${kind}`,
        session_id: id!,
        agent,
        kind,
        count,
      }));
      return { rows };
    }
    case 'session-timeline':
      return { rows: timeline(id!, Number(p('slot_seconds') ?? 1)) };
    case 'figures':
      return { rows: figures(ms(p('since')!), Number(p('bucket_seconds'))) };
    default:
      return { rows: [] };
  }
}

/** A KPI report for a fixture session, enough for the session page's cost and prompt bars. */
function kpisOf(r: SessionListRow) {
  const evs = fixtures.events.filter((e) => e.session_id === r.session_id);
  const byModel = new Map<string, number>();
  for (const e of evs) if (e.kind === 'model_turn') byModel.set(e.label, (byModel.get(e.label) ?? 0) + (e.cost_microdollars ?? 0));
  const byAuthor = new Map<string, number>();
  for (const e of evs) if (e.kind === 'prompt') byAuthor.set(e.label, (byAuthor.get(e.label) ?? 0) + 1);
  return {
    sessions: 1,
    models: [...byModel.entries()].map(([model, cost_microdollars]) => ({ model, cost_microdollars, output_tokens: 0 })),
    autonomy: { prompts: [...byAuthor.entries()].map(([author, count]) => ({ author, count })) },
  };
}
