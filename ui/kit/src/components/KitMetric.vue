<script setup lang="ts">
// One number that matters, its label and a hint.
import type { Tone } from '../types';

withDefaults(defineProps<{ label: string; value: string | number; hint?: string; tone?: Tone }>(), {
  hint: '',
  tone: 'neutral',
});
</script>

<template>
  <div class="kit-metric" :class="`tone-${tone}`">
    <span class="label">{{ label }}</span>
    <span class="value">{{ value }}</span>
    <span v-if="hint" class="hint">{{ hint }}</span>
  </div>
</template>

<style scoped>
.kit-metric {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  padding: var(--kit-space-3);
  background: var(--kit-surface);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-card);
}
.label {
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.value {
  color: var(--kit-heading);
  font-size: var(--kit-text-2xl);
  font-weight: var(--kit-weight-title);
  letter-spacing: var(--kit-tracking-title);
  font-variant-numeric: tabular-nums;
}
.hint {
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
}
.tone-success .value {
  color: var(--kit-success);
}
.tone-danger .value {
  color: var(--kit-danger);
}
.tone-warning .value {
  color: var(--kit-warning);
}
.tone-info .value {
  color: var(--kit-info);
}
</style>
