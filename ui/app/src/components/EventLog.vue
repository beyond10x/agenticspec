<script setup lang="ts">
import { computed, onUnmounted, ref, shallowRef, watch } from 'vue';
import type { EventAppended, EventKind, SessionEventsRow } from '../generated/types.ts';
import type { AgentLane } from '../session.ts';
import { query } from '../client.ts';
import { clockSeconds, compact, day, int, modelName, money } from '../format.ts';
import { AUTHOR, KIND_LABELS, KIND_PLURALS, toolColor, toolLabel } from '../legend.ts';
import { ms } from '../stats.ts';

// The events of a session, newest first, narrowed to the notable ones, all of them or one kind,
// and to the selected agent. The server answers a page of 500 at a time; the log shows them 60
// at a time and asks for the next page when those run out. Live events come in on top.
const props = withDefaults(
  defineProps<{
    sessionId: string;
    lanes: AgentLane[];
    selected?: string | null;
    appended?: EventAppended | null;
    pageSize?: number;
  }>(),
  { selected: null, appended: null, pageSize: 60 },
);
const emit = defineEmits<{ select: [string | null] }>();

const SERVER_PAGE = 500;
type Mode = 'notable' | 'all' | EventKind;
const KINDS: EventKind[] = ['prompt', 'model_turn', 'tool_call', 'decision', 'tool_result', 'cache_bust', 'compaction', 'exposure', 'turn_end'];

const mode = ref<Mode>('notable');
const rows = shallowRef<SessionEventsRow[]>([]);
const counts = ref(new Map<EventKind, number>());
const shown = ref(props.pageSize);
const more = ref(false);
const loading = ref(false);
const error = ref<string | null>(null);
let controller: AbortController | null = null;

const names = computed(() => new Map(props.lanes.map((l) => [l.agent.agent_id, l.name])));

function params(before?: number) {
  const m = mode.value;
  return {
    session_id: props.sessionId,
    agent: props.selected ?? undefined,
    kind: m === 'notable' || m === 'all' ? undefined : m,
    notable: m === 'notable' ? true : undefined,
    before,
  };
}

async function load(next = false) {
  controller?.abort();
  controller = new AbortController();
  const signal = controller.signal;
  loading.value = true;
  error.value = null;
  try {
    const before = next && rows.value.length ? rows.value[rows.value.length - 1].ord : undefined;
    const page = await query('session-events', params(before), { signal });
    rows.value = next ? [...rows.value, ...page] : page;
    more.value = page.length >= SERVER_PAGE;
    if (!next) {
      shown.value = props.pageSize;
      const c = await query('session-event-counts', { session_id: props.sessionId, agent: props.selected ?? undefined }, { signal });
      counts.value = new Map(c.map((x) => [x.kind, x.count]));
    }
  } catch (err) {
    if ((err as Error).name !== 'AbortError') error.value = (err as Error).message;
  } finally {
    loading.value = false;
  }
}

watch(() => [props.sessionId, props.selected, mode.value], () => void load(), { immediate: true });
onUnmounted(() => controller?.abort());

function matches(e: EventAppended): boolean {
  if (props.selected && e.agent !== props.selected) return false;
  if (mode.value === 'all') return true;
  if (mode.value === 'notable') return e.notable;
  return e.kind === mode.value;
}

watch(
  () => props.appended,
  (e) => {
    if (!e || e.session_id !== props.sessionId) return;
    if (!props.selected || e.agent === props.selected) {
      const c = new Map(counts.value);
      c.set(e.kind, (c.get(e.kind) ?? 0) + 1);
      counts.value = c;
    }
    if (!matches(e) || rows.value.some((r) => r.seq === e.seq)) return;
    rows.value = [e, ...rows.value];
    shown.value += 1;
  },
);

const total = computed(() => {
  const m = mode.value;
  const c = counts.value;
  if (m === 'all') return [...c.values()].reduce((a, b) => a + b, 0);
  if (m === 'notable') return null;
  return c.get(m) ?? 0;
});
const options = computed(() => [
  { id: 'notable' as Mode, label: 'Key events', count: null as number | null },
  { id: 'all' as Mode, label: 'All', count: [...counts.value.values()].reduce((a, b) => a + b, 0) },
  ...KINDS.filter((k) => counts.value.get(k)).map((k) => ({ id: k as Mode, label: KIND_PLURALS[k], count: counts.value.get(k) ?? 0 })),
]);
const visible = computed(() => rows.value.slice(0, shown.value));
const hasMore = computed(() => rows.value.length > shown.value || more.value);

async function showMore() {
  if (rows.value.length <= shown.value && more.value) await load(true);
  shown.value += props.pageSize;
}

function what(e: SessionEventsRow): string {
  switch (e.kind) {
    case 'tool_call':
      return e.label;
    case 'model_turn':
      return modelName(e.label);
    case 'prompt':
      return `by ${(AUTHOR[e.label] ?? e.label).toLowerCase()}`;
    case 'decision':
      return `${e.detail ?? ''} · ${e.label}`;
    case 'tool_result':
      return e.ok === false ? 'error' : 'ok';
    case 'cache_bust':
      return e.label.replace(/_/g, ' ');
    default:
      return e.label;
  }
}

function amount(e: SessionEventsRow): string {
  if (e.kind === 'model_turn') {
    const parts = [];
    if (e.context_tokens !== undefined) parts.push(`${compact(e.context_tokens)} ctx`);
    if (e.output_tokens) parts.push(`${compact(e.output_tokens)} out`);
    return parts.join(' · ');
  }
  if (e.kind === 'cache_bust' && e.cache_write_tokens) return `${compact(e.cache_write_tokens)} rewritten`;
  if (e.kind === 'compaction' && e.context_tokens) return `${compact(e.context_tokens)} ctx`;
  if (e.bytes !== undefined) return `${int(e.bytes)} B`;
  return '';
}
</script>

<template>
  <div>
    <div class="filters" role="radiogroup" aria-label="Events to show">
      <button
        v-for="o in options"
        :key="o.id"
        type="button"
        class="chip"
        role="radio"
        :aria-checked="mode === o.id"
        :aria-pressed="mode === o.id"
        @click="mode = o.id"
      >
        {{ o.label }} <span v-if="o.count !== null" class="count">{{ int(o.count) }}</span>
      </button>
      <button v-if="selected" type="button" class="chip clear" aria-pressed="true" @click="emit('select', null)">
        {{ names.get(selected) ?? 'Agent' }} only ✕
      </button>
    </div>
    <div class="wrap">
      <table class="data">
        <thead>
          <tr>
            <th>Time</th>
            <th class="hide-sm">Agent</th>
            <th>Event</th>
            <th>What</th>
            <th class="r hide-sm">Size</th>
            <th class="r">Cost</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="e in visible" :key="e.seq">
            <td class="time num">
              {{ clockSeconds(ms(e.at)) }}
              <span class="muted day">{{ day(ms(e.at)) }}</span>
            </td>
            <td class="hide-sm agent">{{ names.get(e.agent) ?? e.agent }}</td>
            <td class="kind">
              <i v-if="e.kind === 'tool_call'" class="dot" :style="{ background: toolColor(e.detail) }" />
              <span>{{ e.kind === 'tool_call' ? toolLabel(e.detail) : KIND_LABELS[e.kind] }}</span>
            </td>
            <td class="what" :class="{ bad: (e.kind === 'decision' && e.ok === false) || (e.kind === 'tool_result' && e.ok === false) }">{{ what(e) }}</td>
            <td class="r hide-sm muted">{{ amount(e) }}</td>
            <td class="r">
              <span v-if="e.cost_microdollars">{{ money(e.cost_microdollars / 1e6) }}</span>
              <span v-else class="muted">·</span>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-if="error" class="empty">The events could not be read: {{ error }}</div>
      <div v-else-if="!rows.length && !loading" class="empty">No events of this kind.</div>
      <div v-if="hasMore" class="more">
        <button type="button" class="chip" :disabled="loading" @click="showMore">
          Show {{ pageSize }} more{{ total !== null ? ` of ${int(total - shown)}` : '' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 0 20px 12px;
}
.clear {
  margin-left: auto;
}
.wrap {
  border-top: 1px solid var(--grid);
}
.time {
  white-space: nowrap;
}
.day {
  margin-left: 6px;
  font-size: 12px;
}
.agent {
  white-space: nowrap;
  color: var(--ink-2);
}
.kind {
  white-space: nowrap;
}
.kind .dot {
  margin-right: 7px;
}
.what {
  max-width: 340px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.what.bad {
  color: var(--bad-text);
}
.more {
  display: flex;
  justify-content: center;
  padding: 12px;
  border-top: 1px solid var(--grid);
}
@media (max-width: 720px) {
  .filters {
    padding: 0 14px 12px;
  }
  .day {
    display: none;
  }
  .what {
    max-width: 120px;
  }
}
</style>
