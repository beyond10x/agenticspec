<script setup lang="ts">
import { computed } from 'vue';

// One whole split into parts: a thin bar and a legend that names each part with its value.
const props = defineProps<{ parts: { label: string; value: number; color: string; text?: string }[] }>();

const total = computed(() => props.parts.reduce((a, p) => a + p.value, 0));
const shown = computed(() => props.parts.filter((p) => p.value > 0));
</script>

<template>
  <div class="segbar">
    <div class="bar" role="img" :aria-label="shown.map((p) => `${p.label} ${p.text ?? p.value}`).join(', ')">
      <span
        v-for="p in shown"
        :key="p.label"
        :style="{ flexGrow: p.value, background: p.color }"
        :title="`${p.label}: ${p.text ?? p.value}`"
      />
    </div>
    <ul class="legend">
      <li v-for="p in shown" :key="p.label">
        <i class="dot" :style="{ background: p.color }" />
        <span class="label">{{ p.label }}</span>
        <span class="value num">{{ p.text ?? p.value }}</span>
        <span class="share muted num">{{ total ? Math.round((p.value / total) * 100) : 0 }}%</span>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.bar {
  display: flex;
  gap: 2px;
  height: 8px;
}
.bar span {
  flex-basis: 0;
  min-width: 3px;
  border-radius: 4px;
}
.legend {
  list-style: none;
  margin: 12px 0 0;
  padding: 0;
  display: grid;
  gap: 6px;
  font-size: 13px;
}
.legend li {
  display: grid;
  grid-template-columns: 10px 1fr auto 40px;
  gap: 8px;
  align-items: center;
}
.label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ink-2);
}
.value {
  font-weight: 500;
}
.share {
  text-align: right;
  font-size: 12px;
}
</style>
