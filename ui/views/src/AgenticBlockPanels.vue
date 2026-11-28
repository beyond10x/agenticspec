<script setup lang="ts">
// Every panel of an architecture block (`agent`, `pool`), scoped like a single panel.
import { computed } from 'vue';
import { KitEmpty, KitPanel, KitStack } from '@kit';
import { PANELS, panelsOfBlock } from './panels';

const props = defineProps<{
  block: string;
  source?: string;
  architecture?: string;
  part?: string;
  instance?: number;
  args?: Record<string, unknown>;
}>();

const names = computed(() => panelsOfBlock(props.block));
const bound = computed(() => {
  const scope: Record<string, unknown> = { channel: '', ...(props.args ?? {}) };
  for (const k of ['source', 'architecture', 'part', 'instance'] as const) {
    if (props[k] !== undefined && props[k] !== '') scope[k] = props[k];
  }
  return scope;
});
const title = (name: string) => name.replace(/^Agentic[A-Z][a-z0-9]*/, '').replace(/([a-z0-9])([A-Z])/g, '$1 $2');
</script>

<template>
  <KitStack v-if="names.length" :padded="false" :data-block="block">
    <KitPanel v-for="n in names" :key="n" :name="n" :title="title(n)" collapsible>
      <component :is="PANELS[n]" v-bind="bound" />
    </KitPanel>
  </KitStack>
  <KitEmpty v-else :message="`No panel shows the block ${block}.`" />
</template>
