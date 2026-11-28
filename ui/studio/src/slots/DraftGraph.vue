<script setup lang="ts">
// Host slot `DraftGraph`: the graph of the draft's current revision, drawn inline on a canvas that
// pans and zooms. Pointing at a part lights its links; clicking it (or Enter on it) selects it and
// shows its properties and links in a card beside the graph; Escape or Close clears it.
import { computed, nextTick, ref, watch } from 'vue';
import { KitBadge, KitButton, KitEmpty, KitError, KitLoading, KitZoomCanvas } from '@kit';
import type { DraftGraphProps } from '../generated/hosts';
import { asGraph, facts, linksOf } from './graph';
import './graph.css';

const props = defineProps<DraftGraphProps>();

const graph = computed(() => asGraph(props.read.row));
const host = ref<HTMLElement | null>(null);
const selected = ref('');
const pointed = ref('');
const part = computed(() => graph.value?.parts.find((p) => p.id === selected.value) ?? null);

function light(id: string): void {
  const svg = host.value?.querySelector('svg.graph');
  if (!svg) return;
  svg.classList.toggle('focus', id !== '');
  for (const e of svg.querySelectorAll('.edge')) {
    const hot = id !== '' && [e.getAttribute('data-from'), e.getAttribute('data-to'), e.getAttribute('data-over')].includes(id);
    e.classList.toggle('hot', hot);
  }
}

function mark(): void {
  for (const n of host.value?.querySelectorAll('.node[data-part]') ?? []) {
    n.querySelector(':scope > [role=button]')?.setAttribute('aria-pressed', String(n.getAttribute('data-part') === selected.value));
  }
  light(pointed.value || selected.value);
}

function partAt(e: Event): string {
  const n = (e.target as Element | null)?.closest?.('.node[data-part]');
  return n?.getAttribute('data-part') ?? '';
}

function select(id: string): void {
  selected.value = selected.value === id ? '' : id;
  mark();
}

function onClick(e: MouseEvent): void {
  const id = partAt(e);
  if (id) select(id);
}
function onKey(e: KeyboardEvent): void {
  const id = partAt(e);
  if (id && (e.key === 'Enter' || e.key === ' ')) {
    e.preventDefault();
    e.stopPropagation();
    select(id);
  } else if (e.key === 'Escape' && selected.value) {
    selected.value = '';
    mark();
  }
}
function onOver(e: Event): void {
  const id = partAt(e);
  if (id !== pointed.value) {
    pointed.value = id;
    light(id || selected.value);
  }
}

watch(
  () => graph.value?.svg,
  async () => {
    if (selected.value && !graph.value?.parts.some((p) => p.id === selected.value)) selected.value = '';
    await nextTick();
    mark();
  },
);

const count = computed(() => {
  const g = graph.value;
  return g ? `${g.parts.length} parts · ${g.links.length} links · revision ${g.revision}` : '';
});
</script>

<template>
  <div class="draft-graph" data-slot="draft-graph" :data-state="read.state">
    <div v-if="read.state === 'loading' && !graph" class="status"><KitLoading label="Drawing the graph" :lines="4" /></div>
    <div v-else-if="read.state === 'failed'" class="status">
      <KitError message="The graph could not be drawn." :detail="read.error" @retry="read.reload()" />
    </div>
    <div v-else-if="!graph" class="status"><KitEmpty title="No graph" message="Open a draft to see its graph." /></div>
    <KitZoomCanvas
      v-else
      :label="`Graph of ${graph.name}`"
      :content-width="graph.width"
      :content-height="graph.height"
    >
      <!-- The SVG is agenticspec-render's, drawn by the studio's own server from the draft. -->
      <div
        ref="host"
        class="svg-host"
        @click="onClick"
        @keydown="onKey"
        @pointerover="onOver"
        @pointerleave="onOver"
        @focusin="onOver"
        @focusout="pointed = ''; light(selected)"
        v-html="graph.svg"
      />
      <template #overlay>
        <aside v-if="part" class="card" aria-label="Selected part" data-selected-part :data-part="part.id">
          <header class="card-head">
            <span class="swatch" :class="`f-${part.family}`" aria-hidden="true" />
            <h3 class="card-title">{{ part.id }}</h3>
            <KitButton tone="ghost" size="sm" @click="select(part.id)">Close</KitButton>
          </header>
          <p v-if="part.subtitle" class="subtitle">{{ part.subtitle }}</p>
          <dl class="kv">
            <template v-for="[k, v] in facts(part)" :key="k">
              <dt>{{ k }}</dt>
              <dd>{{ v }}</dd>
            </template>
          </dl>
          <template v-if="graph && linksOf(graph, part.id).length">
            <h4 class="links-head">Links</h4>
            <ul class="links">
              <li v-for="(l, i) in linksOf(graph, part.id)" :key="i">
                <span>{{ l.from }}</span>
                <span class="arrow" aria-label="to">→</span>
                <span>{{ l.to }}</span>
                <KitBadge tone="info">{{ l.carries }}</KitBadge>
                <span v-if="l.over" class="over">over {{ l.over }}</span>
              </li>
            </ul>
          </template>
        </aside>
      </template>
    </KitZoomCanvas>
    <p v-if="graph" class="facts" data-graph-facts>{{ count }}</p>
  </div>
</template>

<style scoped>
.draft-graph {
  position: relative;
  display: flex;
  flex: 1 1 auto;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}
.status {
  padding: var(--kit-space-4);
}
.svg-host {
  width: 100%;
  height: 100%;
}
.facts {
  position: absolute;
  left: var(--kit-space-3);
  bottom: var(--kit-space-3);
  z-index: 5;
  padding: 0.2rem 0.6rem;
  background: var(--kit-surface);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-pill);
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
  pointer-events: none;
}
@media (max-width: 640px) {
  .facts {
    display: none;
  }
}
.card {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
  width: 19rem;
  max-width: calc(100vw - 3rem);
  max-height: 100%;
  overflow-y: auto;
  padding: var(--kit-space-3);
  background: var(--kit-surface-raised);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-card);
  box-shadow: var(--kit-shadow-lg);
  font-size: var(--kit-text-sm);
}
.card-head {
  display: flex;
  align-items: center;
  gap: var(--kit-space-2);
}
.card-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  font-size: var(--kit-text-md);
  text-overflow: ellipsis;
  white-space: nowrap;
}
.swatch {
  width: 0.7rem;
  height: 0.7rem;
  flex: none;
  border-radius: 3px;
  background: var(--c, var(--kit-line));
}
.f-principal { --c: var(--kit-series-4); }
.f-agent { --c: var(--kit-series-1); }
.f-collab { --c: var(--kit-series-5); }
.f-work { --c: var(--kit-series-6); }
.f-intel { --c: var(--kit-series-7); }
.f-capability { --c: var(--kit-series-3); }
.f-place { --c: var(--kit-series-2); }
.f-resource { --c: var(--kit-series-8); }
.subtitle {
  color: var(--kit-muted);
}
.kv {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 0.2rem var(--kit-space-2);
  margin: 0;
}
dt {
  color: var(--kit-muted);
}
dd {
  margin: 0;
  color: var(--kit-heading);
  overflow-wrap: anywhere;
}
.links-head {
  font-size: var(--kit-text-xs);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--kit-muted);
}
.links {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
.links li {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.3rem;
}
.arrow,
.over {
  color: var(--kit-muted);
}
</style>
