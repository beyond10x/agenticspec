<script setup lang="ts">
// A table of rows by their fields, compact, scrolling sideways inside itself when narrow. The
// actions slot adds a last column for each row.
import { useSlots } from 'vue';
import KitValue from './KitValue.vue';
import type { FieldDef, KitRow } from '../types';

withDefaults(
  defineProps<{ label: string; columns: FieldDef[]; rows: KitRow[]; rowKey: string; emptyMessage?: string }>(),
  { emptyMessage: 'No rows.' },
);
defineSlots<{ actions?: (p: { row: KitRow }) => unknown }>();
const slots = useSlots();
</script>

<template>
  <div class="kit-table">
    <table v-if="rows.length">
      <caption class="kit-sr-only">{{ label }}</caption>
      <thead>
        <tr>
          <th v-for="c in columns" :key="c.field" scope="col" :class="{ numeric: c.as === 'number' || c.as === 'percent' }">
            {{ c.label }}
          </th>
          <th v-if="slots.actions" scope="col"><span class="kit-sr-only">Actions</span></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, i) in rows" :key="String(row[rowKey] ?? i)" :data-row="String(row[rowKey] ?? i)">
          <td
            v-for="c in columns"
            :key="c.field"
            :data-field="c.field"
            :class="{ numeric: c.as === 'number' || c.as === 'percent' }"
          >
            <KitValue :def="c" :value="row[c.field]" />
          </td>
          <td v-if="slots.actions" class="actions"><slot name="actions" :row="row" /></td>
        </tr>
      </tbody>
    </table>
    <p v-else class="empty">{{ emptyMessage }}</p>
  </div>
</template>

<style scoped>
.kit-table {
  min-width: 0;
  overflow-x: auto;
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--kit-text-sm);
}
th {
  padding: 0.4rem var(--kit-space-2);
  border-bottom: 1px solid var(--kit-hairline);
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  text-align: left;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  white-space: nowrap;
}
td {
  padding: 0.4rem var(--kit-space-2);
  border-bottom: 1px solid var(--kit-hairline);
  vertical-align: top;
}
tbody tr:last-child td {
  border-bottom: 0;
}
tbody tr:hover td {
  background: var(--kit-surface-muted);
}
.numeric {
  text-align: right;
}
.actions {
  text-align: right;
  white-space: nowrap;
}
.empty {
  color: var(--kit-muted);
  font-size: var(--kit-text-sm);
}
</style>
