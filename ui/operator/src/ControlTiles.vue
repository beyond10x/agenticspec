<script setup lang="ts">
// The tiles of the overview of a control-plane UI: the open rows of each Inbox view, the rows per
// state of each Operations view, and each computed KPI with its prediction. Each tile links to its
// page; the counts are read again when `refresh` changes.
import { reactive, watch } from 'vue';
import { KitMetric, KitPanel } from '@kit';
import type { Client, LayoutControl, LayoutTile } from '../../views/src/index';
import { byValue, predicted } from './control';

const props = defineProps<{ client: Client; architecture: string; control: LayoutControl; refresh: number; href: (page: string) => string }>();

const counted = reactive<Record<number, { value: string; hint: string }>>({});

async function count(i: number, t: LayoutTile): Promise<void> {
  const scope = `/scopes/${[props.architecture, t.section.part].filter((s): s is string => !!s).map(encodeURIComponent).join('/')}`;
  const params: Record<string, string> = {};
  for (const [k, v] of Object.entries(t.section.args)) if (v !== null && v !== undefined && k !== 'channel') params[k] = String(v);
  try {
    const rows = (await props.client.scope(scope).read(t.section.widget as never, params as never)) as Record<string, unknown>[];
    counted[i] = { value: String(rows.length), hint: t.count === 'states' ? byValue(rows, t.field) : 'open' };
  } catch {
    counted[i] = { value: '?', hint: 'could not be read' };
  }
}

watch(
  () => [props.control, props.refresh],
  () => props.control.tiles.forEach((t, i) => void count(i, t)),
  { immediate: true },
);
</script>

<template>
  <KitPanel name="tiles" title="At a glance">
    <div class="tiles">
      <a v-for="(t, i) in control.tiles" :key="`t${i}`" class="tile" :href="href(t.page)" :data-tile="t.section.name">
        <KitMetric :label="t.label" :value="counted[i]?.value ?? '…'" :hint="counted[i]?.hint ?? ''" />
      </a>
      <a v-for="m in control.metrics" :key="m.kpi" class="tile" :href="href('control.metrics')" :data-tile="m.kpi">
        <KitMetric :label="m.label" :value="m.prediction === null ? '–' : predicted(m.prediction)" :hint="m.prediction === null ? 'no prediction' : 'predicted'" />
      </a>
    </div>
  </KitPanel>
</template>

<style scoped>
.tiles {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(13rem, 1fr));
  gap: var(--kit-space-2);
}
.tile {
  color: inherit;
  text-decoration: none;
}
.tile:focus-visible {
  outline: 2px solid var(--kit-accent);
  border-radius: var(--kit-radius-card);
}
</style>
