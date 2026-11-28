<script setup lang="ts">
// The overview of one level of a derived UI, drawn from its layout: its parts, each with a link to
// its page. Following a link sends `agentic-navigate`; the host's URL changes only when the host
// acts on it.
import { computed, ref, watch } from 'vue';
import { KitError, KitLoading, KitPanel, KitTable, type FieldDef } from '@kit';
import { emitNavigate } from './events';
import { loadLayout, type Layout, type LayoutPart } from './layout';

const props = defineProps<{
  source?: string;
  architecture: string;
  /** The path of a nested level; the top level when absent. */
  level?: string;
  instance?: number;
  layout?: Layout;
}>();

const read = ref<Layout | null>(props.layout ?? null);
const error = ref('');
const root = ref<HTMLElement | null>(null);

watch(
  () => [props.source, props.architecture, props.layout] as const,
  async ([source, architecture, given]) => {
    error.value = '';
    if (given) {
      read.value = given;
      return;
    }
    read.value = null;
    try {
      read.value = await loadLayout(source ?? '', architecture);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  },
  { immediate: true },
);

const level = computed(() => read.value?.levels.find((l) => l.path === (props.level ?? '')) ?? read.value?.levels[0] ?? null);
const columns: FieldDef[] = [
  { field: 'label', label: 'Part', as: 'text' },
  { field: 'block', label: 'Block', as: 'badge' },
  { field: 'count', label: 'Copies', as: 'number' },
];
const rows = computed(() => (level.value?.parts ?? []) as unknown as Record<string, unknown>[]);

function hrefOf(p: LayoutPart): string {
  return `#/${p.page.split('.').join('/')}`;
}

function open(p: LayoutPart): void {
  emitNavigate(root.value, { architecture: props.architecture, part: p.path, page: p.page, instance: props.instance });
}
</script>

<template>
  <section ref="root" class="agentic-overview" :data-architecture="architecture">
    <KitError v-if="error" message="The layout could not be read." :detail="error" />
    <KitLoading v-else-if="!level" label="Reading the layout" :lines="3" />
    <KitPanel v-else name="parts" :title="level.name">
      <KitTable label="Parts" :columns="columns" :rows="rows" row-key="path">
        <template #actions="{ row }">
          <a class="open" :href="hrefOf(row as unknown as LayoutPart)" @click.prevent="open(row as unknown as LayoutPart)">Open {{ row.label }}</a>
        </template>
      </KitTable>
    </KitPanel>
  </section>
</template>

<style scoped>
.open {
  color: var(--kit-accent);
}
</style>
