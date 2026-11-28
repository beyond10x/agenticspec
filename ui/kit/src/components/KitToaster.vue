<script setup lang="ts">
// The toasts of the shell, newest last, each dismissable. A danger toast is an alert; the others
// are announced politely.
import KitIcon from './KitIcon.vue';
import type { KitToast } from '../types';

defineProps<{ toasts: KitToast[] }>();
const emit = defineEmits<{ dismiss: [id: number] }>();
</script>

<template>
  <div class="kit-toaster" aria-live="polite">
    <div
      v-for="t in toasts"
      :key="t.id"
      class="toast"
      :class="`tone-${t.tone}`"
      :role="t.tone === 'danger' ? 'alert' : 'status'"
      :data-tone="t.tone"
    >
      <span class="text">{{ t.text }}</span>
      <button type="button" class="close" aria-label="Dismiss" @click="emit('dismiss', t.id)">
        <KitIcon name="close" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.kit-toaster {
  position: fixed;
  right: var(--kit-space-4);
  bottom: var(--kit-space-4);
  z-index: 60;
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
  max-width: min(26rem, calc(100vw - 2 * var(--kit-space-4)));
  pointer-events: none;
}
.toast {
  display: flex;
  align-items: flex-start;
  gap: var(--kit-space-2);
  padding: var(--kit-space-2) var(--kit-space-2) var(--kit-space-2) var(--kit-space-3);
  background: var(--kit-surface-raised);
  border: 1px solid var(--kit-hairline);
  border-left: 4px solid var(--kit-neutral);
  border-radius: var(--kit-radius-md);
  box-shadow: var(--kit-shadow-lg);
  color: var(--kit-text);
  font-size: var(--kit-text-sm);
  pointer-events: auto;
}
.tone-info {
  border-left-color: var(--kit-info);
}
.tone-success {
  border-left-color: var(--kit-success);
}
.tone-warning {
  border-left-color: var(--kit-warning);
}
.tone-danger {
  border-left-color: var(--kit-danger);
}
.text {
  flex: 1;
  overflow-wrap: anywhere;
}
.close {
  display: inline-flex;
  font: inherit;
  padding: 0.2rem;
  border: 0;
  border-radius: var(--kit-radius-sm);
  background: transparent;
  color: var(--kit-muted);
  cursor: pointer;
}
.close:hover {
  background: var(--kit-surface-muted);
  color: var(--kit-heading);
}
</style>
