<script setup lang="ts">
// One block panel by its view, scoped by `source`, `architecture`, `part` and `instance`; `args`
// carries the view's own parameters (a `pool_id`, a `channel`).
import { computed } from 'vue';
import { KitEmpty } from '@kit';
import { panelFor } from './panels';

const props = defineProps<{
  view: string;
  source?: string;
  architecture?: string;
  part?: string;
  instance?: number;
  args?: Record<string, unknown>;
}>();

const panel = computed(() => panelFor(props.view));
const bound = computed(() => {
  const scope: Record<string, unknown> = { channel: '', ...(props.args ?? {}) };
  for (const k of ['source', 'architecture', 'part', 'instance'] as const) {
    if (props[k] !== undefined && props[k] !== '') scope[k] = props[k];
  }
  return scope;
});
</script>

<template>
  <component :is="panel" v-if="panel" v-bind="bound" :data-view="view" />
  <KitEmpty v-else :message="`No panel shows ${view}.`" />
</template>
