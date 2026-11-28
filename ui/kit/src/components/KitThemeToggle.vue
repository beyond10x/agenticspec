<script setup lang="ts">
// The theme switch: one pressed button per theme, in a group named Theme.
import KitIcon from './KitIcon.vue';

defineProps<{ modelValue: string; themes: string[] }>();
const emit = defineEmits<{ 'update:modelValue': [theme: string] }>();

const order = (themes: string[]) => [...themes].sort((a, b) => (a === 'light' ? -1 : b === 'light' ? 1 : a.localeCompare(b)));
const title = (t: string) => t.charAt(0).toUpperCase() + t.slice(1);
</script>

<template>
  <div class="kit-theme-toggle" role="group" aria-label="Theme">
    <button
      v-for="t in order(themes)"
      :key="t"
      type="button"
      class="choice"
      :data-theme-choice="t"
      :aria-pressed="t === modelValue"
      :title="`${title(t)} theme`"
      @click="emit('update:modelValue', t)"
    >
      <KitIcon v-if="t === 'light'" name="sun" />
      <KitIcon v-else-if="t === 'dark'" name="moon" />
      <span class="name">{{ title(t) }}</span>
    </button>
  </div>
</template>

<style scoped>
.kit-theme-toggle {
  display: inline-flex;
  padding: 2px;
  gap: 2px;
  background: var(--kit-surface-muted);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-pill);
}
.choice {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  height: 1.75rem;
  padding: 0 0.65rem;
  border: 0;
  border-radius: var(--kit-radius-pill);
  background: transparent;
  color: var(--kit-muted);
  font: inherit;
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  cursor: pointer;
  transition:
    background var(--kit-motion),
    color var(--kit-motion);
}
.choice:hover {
  color: var(--kit-heading);
}
.choice[aria-pressed='true'] {
  background: var(--kit-surface);
  color: var(--kit-heading);
  box-shadow: var(--kit-shadow-sm);
}
@media (max-width: 640px) {
  .name {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
  }
}
</style>
