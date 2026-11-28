<script setup lang="ts">
// An on/off switch with its label beside it (role switch). Space and Enter flip it.
import { useId } from 'vue';

withDefaults(defineProps<{ label: string; modelValue?: boolean; disabled?: boolean }>(), {
  modelValue: false,
  disabled: false,
});
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();
const id = useId();
</script>

<template>
  <div class="kit-switch">
    <button
      :id="id"
      type="button"
      role="switch"
      class="track"
      :aria-checked="modelValue"
      :disabled="disabled"
      @click="emit('update:modelValue', !modelValue)"
    >
      <span class="knob" aria-hidden="true" />
    </button>
    <label class="label" :for="id">{{ label }}</label>
  </div>
</template>

<style scoped>
.kit-switch {
  display: inline-flex;
  align-items: center;
  gap: var(--kit-space-2);
}
.track {
  position: relative;
  font: inherit;
  flex: none;
  width: 2.25rem;
  height: 1.3rem;
  padding: 0;
  border: 1px solid var(--kit-line);
  border-radius: var(--kit-radius-pill);
  background: var(--kit-surface-muted);
  cursor: pointer;
  transition:
    background var(--kit-motion),
    border-color var(--kit-motion);
}
.knob {
  position: absolute;
  top: 50%;
  left: 0.15rem;
  width: 0.9rem;
  height: 0.9rem;
  border-radius: 50%;
  background: var(--kit-muted);
  box-shadow: var(--kit-shadow-sm);
  transform: translateY(-50%);
  transition:
    transform var(--kit-motion),
    background var(--kit-motion);
}
.track[aria-checked='true'] {
  background: var(--kit-accent);
  border-color: var(--kit-accent);
}
.track[aria-checked='true'] .knob {
  background: var(--kit-on-accent);
  transform: translate(0.95rem, -50%);
}
.track:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.label {
  color: var(--kit-text);
  font-size: var(--kit-text-sm);
  cursor: pointer;
  user-select: none;
}
</style>
