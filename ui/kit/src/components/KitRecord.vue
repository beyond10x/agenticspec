<script setup lang="ts">
// One row as label and value pairs. Fields without a value are left out; a long text is clamped
// to two lines. A danger tone marks a refusal.
import { computed, useSlots } from 'vue';
import KitValue from './KitValue.vue';
import { isEmpty, isLongText, key } from '../format';
import type { FieldDef, KitRow, Tone } from '../types';

const props = withDefaults(defineProps<{ fields: FieldDef[]; row?: KitRow | null; tone?: Tone }>(), {
  row: null,
  tone: 'neutral',
});
defineSlots<{ actions?: (p: { row: KitRow }) => unknown }>();
const slots = useSlots();

const shown = computed(() => (props.row ? props.fields.filter((f) => !isEmpty(props.row?.[f.field])) : []));
</script>

<template>
  <div v-if="row" class="kit-record" :class="`tone-${tone}`" :data-tone="tone">
    <dl class="pairs">
      <div v-for="f in shown" :key="f.field" class="pair" :class="{ wide: f.as === 'list' || isLongText(row[f.field]) }">
        <dt>{{ f.label }}</dt>
        <dd :data-field="f.field" :data-value="key(row[f.field])">
          <KitValue :def="f" :value="row[f.field]" clamp />
        </dd>
      </div>
    </dl>
    <div v-if="slots.actions" class="actions"><slot name="actions" :row="row" /></div>
  </div>
</template>

<style scoped>
.kit-record {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
  min-width: 0;
}
.kit-record.tone-danger,
.kit-record.tone-success,
.kit-record.tone-info,
.kit-record.tone-warning {
  padding: var(--kit-space-2) var(--kit-space-3);
  border-radius: var(--kit-radius-md);
  border: 1px solid var(--kit-hairline);
  border-left-width: 4px;
}
.kit-record.tone-danger {
  border-left-color: var(--kit-danger) !important;
  background: var(--kit-danger-soft);
}
.kit-record.tone-success {
  border-left-color: var(--kit-success) !important;
}
.kit-record.tone-info {
  border-left-color: var(--kit-info) !important;
}
.kit-record.tone-warning {
  border-left-color: var(--kit-warning) !important;
}
.pairs {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(7.5rem, 1fr));
  gap: var(--kit-space-2) var(--kit-space-3);
  margin: 0;
}
.pair {
  min-width: 0;
}
.pair.wide {
  grid-column: 1 / -1;
}
dt {
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
dd {
  margin: 0.1rem 0 0;
  color: var(--kit-text);
  font-size: var(--kit-text-sm);
  overflow-wrap: anywhere;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--kit-space-2);
}
</style>
