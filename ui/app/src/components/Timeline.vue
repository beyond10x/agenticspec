<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import type { SessionTimelineRow } from '../generated/types.ts';
import { CALL_FIELDS, callsOf, topCategory, type AgentLane } from '../session.ts';
import { scale, breakLabel } from '../timeline.ts';
import { clock, clockSeconds, compact, money, modelName, int, plural } from '../format.ts';
import { TOOL_CATEGORIES, toolColor, toolLabel } from '../legend.ts';
import { ms } from '../stats.ts';

// Agents as lanes over one time axis whose idle gaps are folded into breaks. The server counts
// each agent's events per time slot; a lane shows the agent's context as a grey wash, a tick per
// slot with tool calls coloured by the category it called most, and the slot's prompts,
// refusals, cache busts and compactions. Above the lanes, the session's running cost.
const props = withDefaults(
  defineProps<{ lanes: AgentLane[]; selected?: string | null; showCost?: boolean; maxLanes?: number }>(),
  { selected: null, showCost: true, maxLanes: 12 },
);
const emit = defineEmits<{ select: [string | null] }>();

const root = ref<HTMLElement | null>(null);
const width = ref(900);
let observer: ResizeObserver | null = null;
onMounted(() => {
  if (!root.value) return;
  width.value = root.value.clientWidth || 900;
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(([e]) => (width.value = Math.max(320, Math.floor(e.contentRect.width))));
    observer.observe(root.value);
  }
});
onUnmounted(() => observer?.disconnect());

const narrow = computed(() => width.value < 640);
const labelW = computed(() => (narrow.value ? 104 : 176));
const plotW = computed(() => Math.max(120, width.value - labelW.value - 8));
const laneH = 38;
const costH = computed(() => (props.showCost ? 64 : 0));
const axisH = 26;

const showAll = ref(false);
// A session with many sub-agents shows the busiest ones; the rest are one click away.
const visible = computed(() => {
  if (showAll.value || props.lanes.length <= props.maxLanes) return props.lanes;
  // The main agent, the selected one, then the agents that did the most work, in time order.
  const keep = new Set(props.lanes.filter((l) => l.agent.role === 'Main' || l.agent.agent_id === props.selected));
  const rest = props.lanes
    .filter((l) => !keep.has(l))
    .sort((a, b) => b.agent.tool_calls - a.agent.tool_calls || b.agent.cost_usd - a.agent.cost_usd);
  for (const l of rest) {
    if (keep.size >= props.maxLanes) break;
    keep.add(l);
  }
  return props.lanes.filter((l) => keep.has(l));
});
const hidden = computed(() => props.lanes.length - visible.value.length);

const all = computed(() => props.lanes.flatMap((l) => l.slots));
const sc = computed(() => scale(all.value.flatMap((s) => [ms(s.start), ms(s.end)]), plotW.value));
const contextMax = computed(() => Math.max(1, ...all.value.map((s) => s.context_max ?? 0)));

const cost = computed(() => {
  if (!props.showCost) return null;
  const slots = all.value.slice().sort((a, b) => ms(a.end) - ms(b.end));
  if (slots.length === 0) return null;
  let run = 0;
  const pts = slots.map((s) => {
    run = Math.max(run, s.session_cost_usd);
    return { x: sc.value.x(ms(s.end)), v: run };
  });
  const max = Math.max(run, 0.0001);
  const top = 14;
  const h = costH.value - top - 6;
  const y = (v: number) => top + h - (v / max) * h;
  let line = '';
  pts.forEach((p, i) => (line += `${i ? 'L' : 'M'}${p.x.toFixed(1)},${y(p.v).toFixed(1)}`));
  const last = pts[pts.length - 1];
  const area = `${line}L${last.x.toFixed(1)},${top + h}L${pts[0].x.toFixed(1)},${top + h}Z`;
  return { line, area, last: { x: last.x, y: y(last.v) }, total: run, base: top + h };
});

interface Mark {
  x: number;
  lane: number;
  s: SessionTimelineRow;
}

const lanesGeo = computed(() =>
  visible.value.map((l, i) => {
    const y0 = costH.value + axisH + i * laneH;
    const at = (s: SessionTimelineRow) => sc.value.x((ms(s.start) + ms(s.end)) / 2);
    const turns = l.slots.filter((s) => s.context_max !== undefined && s.context_max !== null);
    const h = laneH - 12;
    const cy = (v: number) => y0 + laneH - 5 - (v / contextMax.value) * h;
    let ctx = '';
    if (turns.length) {
      const pts = turns.map((t) => ({ x: at(t), y: cy(t.context_max ?? 0) }));
      let d = `M${pts[0].x.toFixed(1)},${y0 + laneH - 5}`;
      for (const p of pts) d += `L${p.x.toFixed(1)},${p.y.toFixed(1)}`;
      d += `L${pts[pts.length - 1].x.toFixed(1)},${y0 + laneH - 5}Z`;
      ctx = d;
    }
    return {
      lane: l,
      y0,
      x1: sc.value.x(l.first),
      x2: Math.max(sc.value.x(l.last), sc.value.x(l.first) + 2),
      ctx,
      tools: l.slots.filter((s) => callsOf(s) > 0).map((s) => ({ x: at(s), s, color: toolColor(topCategory(s) ?? 'Other') })),
      prompts: spaced(l.slots.filter((s) => s.human_prompts + s.relayed_prompts > 0).map((s) => ({ x: at(s), s, human: s.human_prompts > 0 }))),
      refusals: spaced(l.slots.filter((s) => s.refusals > 0).map((s) => ({ x: at(s), s }))),
      busts: spaced(l.slots.filter((s) => s.busts > 0).map((s) => ({ x: at(s), s }))),
      compactions: spaced(l.slots.filter((s) => s.compactions > 0).map((s) => ({ x: at(s), s }))),
      all: l.slots.map((s) => ({ x: at(s), s })),
    };
  }),
);

// Marks closer than 7 px to the one before are not drawn: on a long session they would stack
// into a band. The tooltip still finds every slot.
function spaced<T extends { x: number }>(xs: T[]): T[] {
  const out: T[] = [];
  for (const m of xs) if (!out.length || m.x - out[out.length - 1].x >= 7) out.push(m);
  return out;
}

const marks = computed<Mark[]>(() => lanesGeo.value.flatMap((g, lane) => g.all.map((m) => ({ x: m.x, lane, s: m.s }))));

const lanesBottom = computed(() => costH.value + axisH + visible.value.length * laneH);
const footH = computed(() => (sc.value.breaks.length ? 20 : 0));
const height = computed(() => lanesBottom.value + footH.value + 4);

const sums = computed(() => {
  const by = new Map<string, number>();
  let human = 0;
  let relayed = 0;
  let refusals = 0;
  let busts = 0;
  let compactions = 0;
  for (const s of all.value) {
    for (const [cat, f] of CALL_FIELDS) by.set(cat, (by.get(cat) ?? 0) + s[f]);
    human += s.human_prompts;
    relayed += s.relayed_prompts;
    refusals += s.refusals;
    busts += s.busts;
    compactions += s.compactions;
  }
  return { by, human, relayed, refusals, busts, compactions };
});
const usedCategories = computed(() =>
  TOOL_CATEGORIES.filter((c) => (sums.value.by.get(c.id) ?? 0) > 0 || (c.id === 'Other' && (sums.value.by.get('Plan') ?? 0) > 0)),
);
const has = computed(() => ({
  human: sums.value.human > 0,
  otherPrompt: sums.value.relayed > 0,
  refusal: sums.value.refusals > 0,
  bust: sums.value.busts > 0,
  compaction: sums.value.compactions > 0,
}));

const hover = ref<{ m: Mark; left: number; top: number } | null>(null);

function onMove(ev: MouseEvent) {
  const svg = ev.currentTarget as SVGSVGElement;
  const r = svg.getBoundingClientRect();
  const mx = ev.clientX - r.left - labelW.value;
  const my = ev.clientY - r.top - costH.value - axisH;
  const lane = Math.floor(my / laneH);
  if (mx < 0 || lane < 0 || lane >= visible.value.length) {
    hover.value = null;
    return;
  }
  let best: Mark | null = null;
  let bestD = 9;
  for (const m of marks.value) {
    if (m.lane !== lane) continue;
    const d = Math.abs(m.x - mx);
    if (d < bestD) {
      best = m;
      bestD = d;
    }
  }
  if (!best) {
    hover.value = null;
    return;
  }
  const left = Math.min(labelW.value + best.x + 12, width.value - 240);
  hover.value = { m: best, left: Math.max(0, left), top: costH.value + axisH + lane * laneH + laneH };
}

function describe(s: SessionTimelineRow): { title: string; lines: string[] } {
  const a = ms(s.start);
  const b = ms(s.end);
  const title = a === b ? clockSeconds(a) : `${clockSeconds(a)} to ${s.slot_seconds >= 60 ? clock(b) : clockSeconds(b)}`;
  const lines: string[] = [];
  const calls = CALL_FIELDS.filter(([, f]) => s[f] > 0).map(([c, f]) => `${int(s[f])} ${toolLabel(c)}`);
  if (calls.length) lines.push(`${plural(callsOf(s), 'tool call')}: ${calls.join(', ')}`);
  if (s.human_prompts) lines.push(plural(s.human_prompts, 'human prompt'));
  if (s.relayed_prompts) lines.push(plural(s.relayed_prompts, 'operator or peer prompt'));
  if (s.refusals) lines.push(plural(s.refusals, 'refused call'));
  if (s.busts) lines.push(plural(s.busts, 'cache bust'));
  if (s.compactions) lines.push(plural(s.compactions, 'compaction'));
  if (s.context_max) lines.push(`context up to ${compact(s.context_max)} tokens`);
  if (!lines.length) lines.push(plural(s.events, 'event'));
  return { title, lines };
}

function laneMeta(l: AgentLane): string {
  return `${modelName(l.agent.model)} · ${money(l.agent.cost_usd)}`;
}
</script>

<template>
  <div ref="root" class="timeline">
    <div class="legend">
      <span v-for="c in usedCategories" :key="c.id" class="key"><i class="swatch" :style="{ background: c.color }" />{{ c.label }}</span>
      <span v-if="has.human" class="key"><svg width="10" height="8" aria-hidden="true"><path d="M0,0L10,0L5,7Z" class="prompt-human" /></svg>Human prompt</span>
      <span v-if="has.otherPrompt" class="key"><svg width="10" height="8" aria-hidden="true"><path d="M0.5,0.5L9.5,0.5L5,6.5Z" class="prompt-other" /></svg>Operator or peer prompt</span>
      <span v-if="has.refusal" class="key"><svg width="10" height="10" aria-hidden="true" class="refusal"><circle cx="5" cy="5" r="4.5" /><path d="M3,3L7,7M7,3L3,7" /></svg>Refused</span>
      <span v-if="has.bust" class="key"><svg width="10" height="10" aria-hidden="true"><rect x="1.5" y="1.5" width="7" height="7" transform="rotate(45 5 5)" class="bust" /></svg>Cache bust</span>
      <span v-if="has.compaction" class="key"><svg width="8" height="12" aria-hidden="true" class="compaction"><line x1="4" x2="4" y1="2" y2="12" /><rect x="1" y="0" width="6" height="5" rx="1" /></svg>Compaction</span>
      <span class="key"><i class="swatch ctx-swatch" />Context size</span>
      <span v-if="sc.breaks.length" class="key"><i class="swatch break-swatch" />Idle, folded</span>
    </div>
    <div class="plot">
    <svg
      :width="width"
      :height="height"
      :viewBox="`0 0 ${width} ${height}`"
      role="group"
      :aria-label="`Timeline of ${lanes.length} agents`"
      @mousemove="onMove"
      @mouseleave="hover = null"
    >
      <g :transform="`translate(${labelW},0)`">
        <!-- breaks: folded idle time -->
        <g v-for="(b, i) in sc.breaks" :key="`b${i}`">
          <rect :x="b.x + 2" :y="costH" :width="b.w - 4" :height="lanesBottom - costH" class="break" rx="3" />
          <text v-if="b.w >= 30" :x="b.x + b.w / 2" :y="lanesBottom + 15" class="break-label" text-anchor="middle">
            {{ breakLabel(b) }}
          </text>
        </g>

        <!-- running cost -->
        <g v-if="cost">
          <text x="0" y="10" class="axis-title">Running cost</text>
          <line :x1="0" :x2="plotW" :y1="cost.base" :y2="cost.base" class="grid" />
          <path :d="cost.area" class="cost-area" />
          <path :d="cost.line" class="cost-line" />
          <circle :cx="cost.last.x" :cy="cost.last.y" r="4" class="cost-dot" />
          <text
            :x="Math.min(cost.last.x, plotW - 2)"
            :y="Math.max(cost.last.y - 8, 10)"
            text-anchor="end"
            class="cost-label"
          >
            {{ money(cost.total) }}
          </text>
        </g>

        <!-- axis -->
        <g>
          <template v-for="(t, i) in sc.ticks" :key="`t${i}`">
            <line :x1="t.x" :x2="t.x" :y1="costH + axisH - 6" :y2="height - 4" class="grid" />
            <text :x="t.x + 3" :y="costH + axisH - 9" class="tick">{{ t.label }}</text>
          </template>
        </g>

        <!-- lanes -->
        <g v-for="(g, i) in lanesGeo" :key="g.lane.agent.agent_id" :class="{ dim: selected && selected !== g.lane.agent.agent_id }">
          <line :x1="0" :x2="plotW" :y1="g.y0 + laneH" :y2="g.y0 + laneH" class="lane-rule" />
          <path v-if="g.ctx" :d="g.ctx" class="ctx" />
          <line :x1="g.x1" :x2="g.x2" :y1="g.y0 + laneH - 5" :y2="g.y0 + laneH - 5" class="alive" />
          <rect
            v-for="(t, k) in g.tools"
            :key="`c${i}-${k}`"
            :x="t.x - 1"
            :y="g.y0 + 9"
            width="2"
            :height="laneH - 18"
            rx="1"
            :fill="t.color"
          />
          <path
            v-for="(p, k) in g.prompts"
            :key="`p${i}-${k}`"
            :d="`M${p.x - 4.5},${g.y0 + 1}L${p.x + 4.5},${g.y0 + 1}L${p.x},${g.y0 + 7}Z`"
            :class="p.human ? 'prompt-human' : 'prompt-other'"
          />
          <g v-for="(d, k) in g.refusals" :key="`r${i}-${k}`" class="refusal">
            <circle :cx="d.x" :cy="g.y0 + laneH - 9" r="4.5" />
            <path :d="`M${d.x - 2},${g.y0 + laneH - 11}L${d.x + 2},${g.y0 + laneH - 7}M${d.x + 2},${g.y0 + laneH - 11}L${d.x - 2},${g.y0 + laneH - 7}`" />
          </g>
          <rect
            v-for="(b, k) in g.busts"
            :key="`x${i}-${k}`"
            :x="b.x - 4"
            :y="g.y0 + laneH / 2 - 4"
            width="8"
            height="8"
            :transform="`rotate(45 ${b.x} ${g.y0 + laneH / 2})`"
            class="bust"
          />
          <g v-for="(c, k) in g.compactions" :key="`k${i}-${k}`" class="compaction">
            <line :x1="c.x" :x2="c.x" :y1="g.y0 + 3" :y2="g.y0 + laneH - 3" />
            <rect :x="c.x - 3" :y="g.y0 + 2" width="6" height="6" rx="1" />
          </g>
        </g>
      </g>

      <!-- lane labels -->
      <g v-for="g in lanesGeo" :key="`l${g.lane.agent.agent_id}`">
        <rect
          :x="0"
          :y="g.y0"
          :width="labelW - 8"
          :height="laneH"
          class="lane-hit"
          :class="{ on: selected === g.lane.agent.agent_id }"
          rx="6"
          tabindex="0"
          role="button"
          :aria-label="`Show only ${g.lane.name}`"
          :aria-pressed="selected === g.lane.agent.agent_id"
          @click="emit('select', selected === g.lane.agent.agent_id ? null : g.lane.agent.agent_id)"
          @keydown.enter="emit('select', selected === g.lane.agent.agent_id ? null : g.lane.agent.agent_id)"
          @keydown.space.prevent="emit('select', selected === g.lane.agent.agent_id ? null : g.lane.agent.agent_id)"
        />
        <text :x="8" :y="g.y0 + 16" class="lane-name">{{ g.lane.name }}</text>
        <text :x="8" :y="g.y0 + 30" class="lane-meta">{{ narrow ? money(g.lane.agent.cost_usd) : laneMeta(g.lane) }}</text>
      </g>
    </svg>

    <div v-if="hover" class="tip card" :style="{ left: `${hover.left}px`, top: `${hover.top}px` }">
      <div class="tip-title">{{ describe(hover.m.s).title }}</div>
      <div v-for="(l, i) in describe(hover.m.s).lines" :key="i" class="tip-line">{{ l }}</div>
    </div>
    </div>

    <div v-if="hidden > 0 || showAll" class="more">
      <button type="button" class="chip" @click="showAll = !showAll">
        {{ showAll ? `Show the ${maxLanes} busiest agents` : `Show all ${lanes.length} agents (${hidden} with little work hidden)` }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.timeline {
  width: 100%;
}
.plot {
  position: relative;
}
svg {
  display: block;
  overflow: visible;
  font-family: var(--font);
}
.grid {
  stroke: var(--grid);
  stroke-width: 1;
}
.lane-rule {
  stroke: var(--grid);
  stroke-width: 1;
}
.tick {
  font-size: 11px;
  fill: var(--muted);
  font-variant-numeric: tabular-nums;
}
.axis-title {
  font-size: 11px;
  fill: var(--muted);
}
.break {
  fill: var(--surface-2);
}
.break-label {
  font-size: 10.5px;
  fill: var(--muted);
  font-variant-numeric: tabular-nums;
}
.cost-area {
  fill: var(--s1);
  opacity: 0.1;
}
.cost-line {
  fill: none;
  stroke: var(--s1);
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}
.cost-dot {
  fill: var(--s1);
  stroke: var(--surface);
  stroke-width: 2;
}
.cost-label {
  font-size: 12px;
  font-weight: 600;
  fill: var(--ink);
  font-variant-numeric: tabular-nums;
}
.ctx {
  fill: var(--ink);
  opacity: 0.06;
}
.alive {
  stroke: var(--axis);
  stroke-width: 2;
  stroke-linecap: round;
}
.prompt-human {
  fill: var(--ink);
}
.prompt-other {
  fill: none;
  stroke: var(--muted);
  stroke-width: 1.2;
}
.refusal circle {
  fill: var(--critical);
  stroke: var(--surface);
  stroke-width: 1.5;
}
.refusal path {
  stroke: #fff;
  stroke-width: 1.4;
  stroke-linecap: round;
}
.bust {
  fill: var(--serious);
  stroke: var(--surface);
  stroke-width: 1.5;
}
.compaction line {
  stroke: var(--ink-2);
  stroke-width: 1.5;
}
.compaction rect {
  fill: var(--ink-2);
}
.dim {
  opacity: 0.28;
}
.lane-hit {
  fill: transparent;
  cursor: pointer;
}
.lane-hit:hover {
  fill: var(--wash);
}
.lane-hit.on {
  fill: var(--accent-wash);
}
.lane-hit:focus-visible {
  outline: none;
  stroke: var(--accent);
  stroke-width: 2;
}
.lane-name {
  font-size: 12.5px;
  font-weight: 500;
  fill: var(--ink);
  pointer-events: none;
}
.lane-meta {
  font-size: 12px;
  fill: var(--muted);
  pointer-events: none;
  font-variant-numeric: tabular-nums;
}
.tip {
  position: absolute;
  z-index: 5;
  min-width: 160px;
  max-width: 240px;
  padding: 8px 10px;
  pointer-events: none;
  font-size: 12px;
}
.tip-title {
  font-weight: 600;
  margin-bottom: 2px;
}
.tip-line {
  color: var(--ink-2);
  font-variant-numeric: tabular-nums;
}
.more {
  margin-top: 8px;
}
.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 6px 16px;
  margin-bottom: 14px;
  font-size: 12px;
  color: var(--ink-2);
}
.key {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.swatch {
  display: inline-block;
  width: 3px;
  height: 12px;
  border-radius: 1px;
}
.ctx-swatch {
  width: 12px;
  background: var(--ink);
  opacity: 0.12;
}
.break-swatch {
  width: 12px;
  background: var(--surface-2);
  box-shadow: inset 0 0 0 1px var(--ring);
}
</style>
