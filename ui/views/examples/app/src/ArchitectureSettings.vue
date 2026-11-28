<script setup lang="ts">
// Host slot `ArchitectureSettings`: the policies and the marked places of one level, read-only.
import { computed } from 'vue';
import { KitEmpty, KitError, KitLoading, KitTable, type FieldDef } from '@kit';

const props = defineProps<{
  read: { state: string; rows: Record<string, unknown>[]; error: string; reload(): void };
}>();

const columns: FieldDef[] = [
  { field: 'place', label: 'Place', as: 'text' },
  { field: 'value', label: 'Value', as: 'text' },
  { field: 'search', label: 'Searched over', as: 'text' },
];
const rows = computed(() => props.read.rows);
</script>

<template>
  <div data-slot="architecture-settings" :data-state="read.state">
    <KitLoading v-if="read.state === 'loading' && rows.length === 0" label="Reading the settings" :lines="3" />
    <KitError v-else-if="read.state === 'failed'" message="The settings could not be read." :detail="read.error" @retry="read.reload()" />
    <KitEmpty v-else-if="rows.length === 0" message="No policy and no marked place." />
    <KitTable v-else label="Policies and marked places" :columns="columns" :rows="rows" row-key="place" />
  </div>
</template>
