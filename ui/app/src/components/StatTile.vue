<script setup lang="ts">
import { computed } from 'vue';
import Sparkline from './Sparkline.vue';

// A figure: label, value, an optional change against a named period, an optional trend.
const props = defineProps<{
  label: string;
  value: string;
  note?: string;
  change?: number | null;
  higherIsBetter?: boolean | null;
  period?: string;
  trend?: (number | null)[];
}>();

const delta = computed(() => {
  const c = props.change;
  if (c === null || c === undefined || !Number.isFinite(c)) return null;
  const p = Math.round(c * 100);
  if (p === 0) return { text: 'no change', tone: 'flat', arrow: '' };
  const up = p > 0;
  let tone = 'flat';
  if (props.higherIsBetter === true) tone = up ? 'good' : 'bad';
  if (props.higherIsBetter === false) tone = up ? 'bad' : 'good';
  const size = Math.abs(p) >= 1000 ? `${Math.round(Math.abs(p) / 100) / 10}×` : `${Math.abs(p)}%`;
  return { text: `${up ? '+' : '−'}${size}`, tone, arrow: up ? '↑' : '↓' };
});
const hasTrend = computed(() => (props.trend ?? []).filter((v) => v !== null).length >= 2);
</script>

<template>
  <div class="tile card">
    <div class="top">
      <div class="label">{{ label }}</div>
      <Sparkline v-if="hasTrend" class="spark" :values="trend!" :width="56" :height="18" />
    </div>
    <div class="value">{{ value }}</div>
    <div class="meta">
      <div v-if="delta" class="line">
        <span class="delta" :class="delta.tone">{{ delta.arrow }} {{ delta.text }}</span>
        <span v-if="period" class="muted"> vs {{ period }}</span>
      </div>
      <div v-if="note" class="line muted">{{ note }}</div>
    </div>
  </div>
</template>

<style scoped>
.tile {
  padding: 14px 16px 14px;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.label {
  font-size: 12.5px;
  color: var(--ink-2);
  min-width: 0;
}
.top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  min-height: 18px;
}
.value {
  margin-top: 6px;
  font-size: 26px;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.1;
  white-space: nowrap;
}
.spark {
  flex: none;
}
.meta {
  margin-top: 8px;
  font-size: 12px;
  line-height: 1.45;
  min-height: 17px;
}
.line {
  overflow-wrap: anywhere;
}
.delta {
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}
.delta.good {
  color: var(--good-text);
}
.delta.bad {
  color: var(--bad-text);
}
.delta.flat {
  color: var(--ink-2);
}
@media (max-width: 720px) {
  .tile {
    padding: 12px 12px 12px;
  }
  .value {
    font-size: 21px;
  }
  .spark {
    display: none;
  }
}
</style>
