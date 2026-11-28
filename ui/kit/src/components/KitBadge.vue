<script setup lang="ts">
// A short status word with a tone.
import type { Tone } from '../types';

withDefaults(defineProps<{ tone?: Tone }>(), { tone: 'neutral' });
</script>

<template>
  <span class="kit-badge" :class="`tone-${tone}`" :data-tone="tone"><slot /></span>
</template>

<style scoped>
.kit-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.1rem 0.55rem;
  border: 1px solid transparent;
  border-radius: var(--kit-radius-pill);
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  line-height: 1.5;
  white-space: nowrap;
}
.kit-badge::before {
  content: '';
  width: 0.45rem;
  height: 0.45rem;
  border-radius: 50%;
  background: currentColor;
}
.tone-neutral {
  background: var(--kit-neutral-soft);
  color: var(--kit-neutral);
  border-color: var(--kit-hairline);
}
.tone-info {
  background: var(--kit-info-soft);
  color: var(--kit-info);
}
.tone-success {
  background: var(--kit-success-soft);
  color: var(--kit-success);
}
.tone-warning {
  background: var(--kit-warning-soft);
  color: var(--kit-warning);
}
.tone-danger {
  background: var(--kit-danger-soft);
  color: var(--kit-danger);
}
</style>
