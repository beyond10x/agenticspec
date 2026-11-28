<script setup lang="ts">
// Copies a value to the clipboard and says so for two seconds.
import { ref } from 'vue';
import KitIcon from './KitIcon.vue';

const props = withDefaults(defineProps<{ value: string; label?: string }>(), { label: 'Copy' });
const copied = ref(false);
let timer: ReturnType<typeof setTimeout> | undefined;

async function copy(): Promise<void> {
  try {
    await navigator.clipboard.writeText(props.value);
  } catch {
    // Without clipboard access the value stays selectable in its title.
  }
  copied.value = true;
  clearTimeout(timer);
  timer = setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
  <button type="button" class="kit-copy" :data-value="value" :title="value" :aria-label="label" @click="copy">
    <KitIcon :name="copied ? 'check' : 'copy'" />
    <span aria-live="polite">{{ copied ? 'Copied' : label }}</span>
  </button>
</template>

<style scoped>
.kit-copy {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  height: var(--kit-control-height-sm);
  padding: 0 var(--kit-space-2);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-sm);
  background: var(--kit-surface);
  color: var(--kit-text);
  font: inherit;
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  cursor: pointer;
  transition:
    border-color var(--kit-motion),
    background var(--kit-motion);
}
.kit-copy:hover {
  border-color: var(--kit-line);
  background: var(--kit-surface-muted);
}
</style>
