<script setup lang="ts">
// One value as its field shows it: text, a number, a badge, a list, a time. A digest is shown
// short with the whole in its title; a long text in a record is clamped to two lines.
import { computed } from 'vue';
import KitBadge from './KitBadge.vue';
import KitClamp from './KitClamp.vue';
import { formatValue, isDigest, isEmpty, isLongText, key, listItems, toneOf } from '../format';
import type { FieldDef } from '../types';

const props = withDefaults(defineProps<{ def: FieldDef; value: unknown; clamp?: boolean }>(), { clamp: false });

const shown = computed(() => formatValue(props.def, props.value));
const digest = computed(() => isDigest(props.value));
</script>

<template>
  <span v-if="isEmpty(value)" class="kit-value empty">—</span>
  <KitBadge v-else-if="def.as === 'badge'" :tone="toneOf(def, value)">{{ shown }}</KitBadge>
  <ul v-else-if="def.as === 'list'" class="kit-value list">
    <li v-for="(item, i) in listItems(value)" :key="i">{{ item }}</li>
  </ul>
  <KitClamp v-else-if="clamp && isLongText(value)" class="kit-value" :text="key(value)" />
  <code v-else-if="digest" class="kit-value digest" :title="key(value)">{{ shown }}</code>
  <span v-else class="kit-value" :class="{ numeric: def.as === 'number' || def.as === 'percent' }">{{ shown }}</span>
</template>

<style scoped>
.empty {
  color: var(--kit-muted);
}
.numeric {
  font-variant-numeric: tabular-nums;
}
.digest {
  padding: 0.05rem 0.35rem;
  background: var(--kit-inline-code);
  border-radius: var(--kit-radius-sm);
  font-size: 0.85em;
}
.list {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  margin: 0;
  padding: 0;
  list-style: none;
  font-family: var(--kit-mono);
  font-size: var(--kit-text-xs);
}
.list li {
  padding: 0.2rem 0.45rem;
  background: var(--kit-code-bg);
  border: 1px solid var(--kit-code-border);
  border-radius: var(--kit-radius-sm);
  overflow-wrap: anywhere;
}
</style>
