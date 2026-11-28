<script setup lang="ts">
// Rows as a list for a narrow place: the first field leads, the others follow in small type.
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
  <ul v-if="rows.length" class="kit-list" :aria-label="label">
    <li v-for="(row, i) in rows" :key="String(row[rowKey] ?? i)" class="item" :data-row="String(row[rowKey] ?? i)">
      <div class="fields">
        <span v-for="(c, j) in columns" :key="c.field" :class="j === 0 ? 'lead' : 'rest'" :data-field="c.field">
          <span v-if="j > 0" class="kit-sr-only">{{ c.label }}: </span>
          <KitValue :def="c" :value="row[c.field]" />
        </span>
      </div>
      <div v-if="slots.actions" class="actions"><slot name="actions" :row="row" /></div>
    </li>
  </ul>
  <p v-else class="kit-list-empty">{{ emptyMessage }}</p>
</template>

<style scoped>
.kit-list {
  display: flex;
  flex-direction: column;
  margin: 0;
  padding: 0;
  list-style: none;
}
.item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--kit-space-2);
  padding: 0.45rem 0;
  border-bottom: 1px solid var(--kit-hairline);
}
.item:last-child {
  border-bottom: 0;
}
.fields {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.15rem var(--kit-space-2);
  min-width: 0;
  font-size: var(--kit-text-sm);
}
.lead {
  color: var(--kit-heading);
  font-weight: var(--kit-weight-strong);
}
.rest {
  color: var(--kit-muted);
}
.rest:last-child {
  flex-basis: 100%;
  color: var(--kit-text);
}
.kit-list-empty {
  color: var(--kit-muted);
  font-size: var(--kit-text-sm);
}
</style>
