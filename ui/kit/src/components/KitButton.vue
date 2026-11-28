<script setup lang="ts">
// A button: primary for the one action of a place, secondary beside it, ghost for tools, danger
// for what removes. Busy keeps its width and says so to assistive technology.
withDefaults(
  defineProps<{
    tone?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md';
    type?: 'button' | 'submit';
    busy?: boolean;
    disabled?: boolean;
    pressed?: boolean;
  }>(),
  { tone: 'secondary', size: 'md', type: 'button', busy: false, disabled: false, pressed: undefined },
);
</script>

<template>
  <button
    class="kit-button"
    :class="[`tone-${tone}`, `size-${size}`, { busy }]"
    :type="type"
    :disabled="disabled || busy"
    :aria-busy="busy || undefined"
    :aria-pressed="pressed"
  >
    <span v-if="busy" class="spinner" aria-hidden="true" />
    <span class="content"><slot /></span>
  </button>
</template>

<style scoped>
.kit-button {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--kit-space-1);
  height: var(--kit-control-height);
  padding: 0 var(--kit-space-3);
  border: 1px solid transparent;
  border-radius: var(--kit-radius-md);
  font: inherit;
  font-size: var(--kit-text-sm);
  font-weight: var(--kit-weight-strong);
  line-height: 1;
  white-space: nowrap;
  cursor: pointer;
  transition:
    background var(--kit-motion),
    border-color var(--kit-motion),
    color var(--kit-motion),
    box-shadow var(--kit-motion);
}
.kit-button.size-sm {
  height: var(--kit-control-height-sm);
  padding: 0 var(--kit-space-2);
  font-size: var(--kit-text-xs);
  border-radius: var(--kit-radius-sm);
}
.content {
  display: inline-flex;
  align-items: center;
  gap: var(--kit-space-1);
}
.tone-primary {
  background: var(--kit-accent);
  color: var(--kit-on-accent);
  box-shadow: var(--kit-shadow-sm);
}
.tone-primary:hover:not(:disabled) {
  background: var(--kit-accent-strong);
}
.tone-secondary {
  background: var(--kit-surface);
  border-color: var(--kit-hairline);
  color: var(--kit-heading);
  box-shadow: var(--kit-shadow-sm);
}
.tone-secondary:hover:not(:disabled) {
  border-color: var(--kit-line);
  background: var(--kit-surface-muted);
}
.tone-ghost {
  background: transparent;
  color: var(--kit-text);
}
.tone-ghost:hover:not(:disabled),
.tone-ghost[aria-pressed='true'] {
  background: var(--kit-surface-muted);
  color: var(--kit-heading);
}
.tone-danger {
  background: var(--kit-danger-soft);
  border-color: var(--kit-danger);
  color: var(--kit-danger);
}
.kit-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
  box-shadow: none;
}
.kit-button.busy {
  opacity: 1;
  cursor: progress;
}
.busy .content {
  opacity: 0;
}
.spinner {
  position: absolute;
  width: 1em;
  height: 1em;
  border: 2px solid currentColor;
  border-right-color: transparent;
  border-radius: 50%;
  animation: kit-spin 0.8s linear infinite;
}
@keyframes kit-spin {
  to {
    transform: rotate(360deg);
  }
}
@media (prefers-reduced-motion: reduce) {
  .spinner {
    animation-duration: 2.4s;
  }
}
</style>
