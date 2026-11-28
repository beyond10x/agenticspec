<script setup lang="ts">
// Host slot `DraftPanels`: the block panels of one part of the draft, imported from the package of
// ui/views (no panel is written here) and answered by the panels' fixtures until the studio runs
// the draft. The panels draw with the runtime they were generated with, so they mount as an island
// of their own beside the studio's pages.
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { KitEmpty, KitSelect, type KitScalar } from '@kit';
import { AgenticBlockPanels, mountViews } from '../../../views/src/index';
import { fixtureClient, scripts, views } from '../../../views/gallery/src/fixtures';
import type { DraftPanelsProps } from '../generated/hosts';
import { asGraph } from './graph';

const props = defineProps<DraftPanelsProps>();

const graph = computed(() => asGraph(props.read.row));
const parts = computed(() => (graph.value?.parts ?? []).filter((p) => p.block !== 'architecture'));
const options = computed(() => parts.value.map((p) => ({ value: p.id, label: `${p.id} (${p.block})` })));
const chosen = ref('');
watch(
  parts,
  (ps) => {
    if (!ps.some((p) => p.id === chosen.value)) chosen.value = ps[0]?.id ?? '';
  },
  { immediate: true },
);
const part = computed(() => parts.value.find((p) => p.id === chosen.value) ?? null);

const island = ref<HTMLElement | null>(null);
const client = fixtureClient(views, scripts, { play: false });
let unmount: (() => void) | null = null;

watch(
  [part, island],
  ([p, el]) => {
    unmount?.();
    unmount = null;
    if (!p || !el) return;
    unmount = mountViews(el, AgenticBlockPanels, { block: p.block, architecture: graph.value?.name ?? '', part: p.id }, { client });
  },
  { flush: 'post' },
);
onBeforeUnmount(() => unmount?.());

function choose(v: KitScalar): void {
  chosen.value = String(v ?? '');
}
</script>

<template>
  <div class="draft-panels" data-slot="draft-panels" :data-state="read.state" :data-part="chosen">
    <KitEmpty v-if="parts.length === 0" message="The draft has no part with panels." />
    <template v-else>
      <KitSelect label="Part" :options="options" :model-value="chosen" @update:model-value="choose" />
      <div ref="island" class="island" :data-block="part?.block" />
    </template>
  </div>
</template>

<style scoped>
.draft-panels {
  display: grid;
  gap: var(--kit-space-2);
  min-width: 0;
}
.island {
  min-width: 0;
}
</style>
