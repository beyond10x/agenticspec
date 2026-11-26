<script setup lang="ts">
import { computed, ref, toRef, watch } from 'vue';
import { useSession } from '../session.ts';
import type { LiveState } from '../client.ts';
import { compact, dayClock, duration, int, money, modelName, percent, projectName, clock } from '../format.ts';
import { go } from '../router.ts';
import { AUTHOR } from '../legend.ts';
import { ms, wallSeconds } from '../stats.ts';
import StatTile from '../components/StatTile.vue';
import Timeline from '../components/Timeline.vue';
import AgentTable from '../components/AgentTable.vue';
import CheckList from '../components/CheckList.vue';
import EventLog from '../components/EventLog.vue';
import SegmentBar from '../components/SegmentBar.vue';

// One session. `part` picks what an embed page shows: the whole page, the timeline, or the
// event log; an embed page has no app chrome and no way back to the overview.
const props = withDefaults(defineProps<{ sessionId: string; embed?: boolean; part?: 'all' | 'timeline' | 'events' }>(), {
  embed: false,
  part: 'all',
});
const emit = defineEmits<{ live: [LiveState] }>();

const s = useSession(toRef(props, 'sessionId'));
watch(s.live, (v) => emit('live', v), { immediate: true });
const selected = ref<string | null>(null);

const row = computed(() => s.session.value);
const kpis = computed(() => row.value?.kpis);

const models = computed(() => {
  const k = kpis.value;
  if (!k) return [];
  const byModel = new Map<string, number>();
  for (const m of k.models) byModel.set(m.model, (byModel.get(m.model) ?? 0) + m.cost_microdollars);
  const colors = ['var(--s1)', 'var(--s2)', 'var(--s3)'];
  const sorted = [...byModel.entries()].sort((a, b) => b[1] - a[1]);
  const top = sorted.slice(0, 3);
  const rest = sorted.slice(3).reduce((a, [, v]) => a + v, 0);
  const parts = top.map(([m, v], i) => ({ label: modelName(m), value: v, color: colors[i], text: money(v / 1e6) }));
  if (rest > 0) parts.push({ label: 'Other models', value: rest, color: 'var(--other)', text: money(rest / 1e6) });
  return parts;
});

const prompts = computed(() => {
  const k = kpis.value;
  if (!k) return [];
  const by = new Map<string, number>();
  for (const p of k.autonomy.prompts) by.set(p.author, (by.get(p.author) ?? 0) + p.count);
  const colors: Record<string, string> = {
    Human: 'var(--s1)',
    Delegate: 'var(--s7)',
    PeerSession: 'var(--s5)',
    ParentAgent: 'var(--s3)',
    Harness: 'var(--other)',
    Hook: 'var(--s4)',
  };
  return [...by.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(([a, n]) => ({ label: AUTHOR[a] ?? a, value: n, color: colors[a] ?? 'var(--other)', text: int(n) }));
});

const wall = computed(() => (row.value ? wallSeconds(row.value) : 0));
const sameDay = computed(() => {
  if (!row.value) return false;
  return new Date(ms(row.value.first_at)).toDateString() === new Date(ms(row.value.last_at)).toDateString();
});
const gaps = computed(() => s.checks.value.filter((c) => c.verdict === 'Gap').length);
</script>

<template>
  <section :class="{ embed }">
    <div v-if="!embed" class="crumbs">
      <a href="/" @click.prevent="go({ page: 'overview' })">← Sessions</a>
    </div>

    <div v-if="s.error.value && !row" class="card notice">{{ s.error.value }}</div>
    <div v-else-if="s.loading.value && !row" class="card notice muted">Reading the session…</div>

    <template v-if="row">
      <header class="head">
        <div class="titles">
          <h1 :class="{ small: embed }">{{ row.title || 'Untitled session' }}</h1>
          <div class="meta secondary">
            <span :title="row.project">{{ projectName(row.project) }}</span>
            <span class="sep">·</span>
            <span class="num">
              {{ dayClock(ms(row.first_at)) }} – {{ sameDay ? clock(ms(row.last_at)) : dayClock(ms(row.last_at)) }}
            </span>
            <span class="sep">·</span>
            <span>{{ duration(wall) }} wall</span>
            <span class="sep hide-sm">·</span>
            <span class="hide-sm">{{ int(row.agents) }} {{ row.agents === 1 ? 'agent' : 'agents' }}</span>
          </div>
        </div>
        <div class="hero">
          <div class="hero-value">{{ money(row.cost_usd) }}</div>
          <div class="muted hero-note">at API list price</div>
        </div>
      </header>

      <div v-if="part !== 'timeline'" class="tiles" data-ux-region="tiles">
        <StatTile label="Agent working time" :value="duration(row.working_seconds)" :note="`${percent(wall ? row.working_seconds / wall : 0)} of wall time`" />
        <StatTile label="Waiting on a human" :value="percent(row.waiting_share)" :note="duration(row.waiting_on_human_seconds)" />
        <StatTile label="Peak context" :value="compact(row.context_max)" note="tokens, any agent" />
        <StatTile label="Cache hit share" :value="percent(row.cache_hit_share)" :note="`${int(row.busts)} ${row.busts === 1 ? 'bust' : 'busts'}`" />
        <StatTile
          label="Tool calls"
          :value="int(row.tool_calls)"
          :note="row.refused_tool_calls ? `${int(row.refused_tool_calls)} refused` : 'none refused'"
        />
        <StatTile
          label="Human prompts"
          :value="int(row.human_prompts)"
          :note="row.exposures ? `${int(row.exposures)} secret exposures` : 'no secret exposed'"
        />
      </div>

      <div v-if="part !== 'events'" class="card section-gap" data-ux-region="timeline">
        <div class="card-head">
          <h2>Timeline</h2>
          <span class="hint">{{ selected ? 'Click the agent again to show all' : 'Click an agent to focus it' }}</span>
        </div>
        <div class="card-body">
          <Timeline :lanes="s.lanes.value" :selected="selected" @select="selected = $event" />
        </div>
      </div>

      <div v-if="part === 'all'" class="grid-2 section-gap">
        <div class="card" data-ux-region="agents">
          <div class="card-head"><h2>Agents</h2><span class="hint">{{ int(s.lanes.value.length) }}</span></div>
          <AgentTable class="flush" :lanes="s.lanes.value" :selected="selected" @select="selected = $event" />
        </div>
        <div class="card" data-ux-region="checks">
          <div class="card-head">
            <h2>Checks</h2>
            <span class="hint">{{ gaps ? `${gaps} found` : 'nothing found' }}</span>
          </div>
          <div class="card-body"><CheckList :checks="s.checks.value" /></div>
        </div>
      </div>

      <div v-if="part === 'all' && (models.length || prompts.length)" class="grid-2 section-gap">
        <div v-if="models.length" class="card">
          <div class="card-head"><h2>Cost by model</h2></div>
          <div class="card-body"><SegmentBar :parts="models" /></div>
        </div>
        <div v-if="prompts.length" class="card">
          <div class="card-head"><h2>Prompts by author</h2></div>
          <div class="card-body"><SegmentBar :parts="prompts" /></div>
        </div>
      </div>

      <div v-if="part !== 'timeline'" class="card section-gap" data-ux-region="events">
        <div class="card-head"><h2>Events</h2></div>
        <div class="log">
          <EventLog
            :session-id="sessionId"
            :lanes="s.lanes.value"
            :selected="selected"
            :appended="s.appended.value"
            @select="selected = $event"
          />
        </div>
      </div>
    </template>
  </section>
</template>

<style scoped>
.crumbs {
  margin-bottom: 14px;
  font-size: 13px;
}
.crumbs a {
  color: var(--ink-2);
}
.crumbs a:hover {
  color: var(--ink);
}
.notice {
  padding: 18px 20px;
}
.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 24px;
  margin-bottom: 20px;
}
.titles {
  min-width: 0;
}
h1 {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
h1.small {
  font-size: 18px;
}
.meta {
  margin-top: 6px;
  font-size: 13.5px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px 6px;
}
.sep {
  color: var(--muted);
}
.hero {
  text-align: right;
  flex: none;
}
.hero-value {
  font-size: 40px;
  font-weight: 600;
  letter-spacing: -0.03em;
  line-height: 1;
}
.hero-note {
  font-size: 12px;
  margin-top: 4px;
}
.tiles {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 12px;
}
.grid-2 {
  display: grid;
  grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr);
  gap: 20px;
  align-items: start;
}
.flush {
  margin-top: 6px;
}
.log {
  margin-top: 12px;
}
.embed .head {
  margin-bottom: 14px;
}
@media (max-width: 900px) {
  .grid-2 {
    grid-template-columns: minmax(0, 1fr);
  }
}
@media (max-width: 720px) {
  .head {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  h1 {
    white-space: normal;
  }
  .hero {
    text-align: left;
  }
  .hero-value {
    font-size: 32px;
  }
  .tiles {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }
}
</style>
