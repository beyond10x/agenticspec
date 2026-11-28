<script setup lang="ts">
// One section of a page across parts: the rows of its view at its scope (the architecture, or the
// part it names), ordered newest first when it names a time field, with its row actions. An action
// shows while the action set (R5) of the scope allows it, all of them when the adapter has none,
// and sends `agentic-command` with what its row binds. `refresh` changes when the level's channel
// brings an event, and the rows are read again.
import { computed, ref, watch } from 'vue';
import { KitButton, KitError, KitLoading, KitTable, type FieldDef, type KitRow } from '@kit';
import type { Client, LayoutAction, LayoutControlSection } from '../../views/src/index';
import { emitCommand } from '../../views/src/events';

const props = defineProps<{ client: Client; architecture: string; section: LayoutControlSection; refresh: number }>();

const rows = ref<KitRow[] | null>(null);
const error = ref('');
const allowed = ref<string[] | null>(null);
const root = ref<HTMLElement | null>(null);

const scope = computed(() => `/scopes/${[props.architecture, props.section.part].filter((s): s is string => !!s).map(encodeURIComponent).join('/')}`);
const columns = computed<FieldDef[]>(() => props.section.columns.map((c) => ({ field: c.field, label: c.label, as: c.as })));
const params = computed(() => {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(props.section.args)) if (v !== null && v !== undefined && k !== 'channel') out[k] = String(v);
  return out;
});

function ordered(list: KitRow[]): KitRow[] {
  const field = props.section.order;
  if (!field) return list;
  return [...list].sort((a, b) => String(b[field] ?? '').localeCompare(String(a[field] ?? '')));
}

async function read(): Promise<void> {
  error.value = '';
  try {
    const got = await props.client.scope(scope.value).read(props.section.widget as never, params.value as never);
    rows.value = ordered(got as KitRow[]);
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

watch(() => [scope.value, props.section.widget, props.refresh], read, { immediate: true });
watch(
  scope,
  async (s) => {
    allowed.value = null;
    if (!props.section.row_actions.length) return;
    try {
      const set = await props.client.scope(s).actions();
      if (set) allowed.value = set.filter((a) => a.allowed).map((a) => a.command);
    } catch {
      allowed.value = null;
    }
  },
  { immediate: true },
);

const actions = computed(() => props.section.row_actions.filter((a) => !allowed.value || allowed.value.includes(a.command)));

function act(a: LayoutAction, row: KitRow): void {
  const input: Record<string, unknown> = {};
  for (const [k, f] of Object.entries(a.bind)) input[k] = row[f];
  emitCommand(root.value, { architecture: props.architecture, part: props.section.part ?? '', command: a.command, action: a.name, input });
}
</script>

<template>
  <div ref="root" class="control-table" :data-view="section.widget" :data-part="section.part ?? ''">
    <KitError v-if="error" message="The rows could not be read." :detail="error" />
    <KitLoading v-else-if="!rows" label="Reading the rows" :lines="2" />
    <KitTable v-else-if="actions.length" :label="section.title" :columns="columns" :rows="rows" :row-key="section.key">
      <template #actions="{ row }">
        <KitButton v-for="a in actions" :key="a.name" size="sm" tone="secondary" @click="act(a, row)">{{ a.label }}</KitButton>
      </template>
    </KitTable>
    <KitTable v-else :label="section.title" :columns="columns" :rows="rows" :row-key="section.key" />
  </div>
</template>
