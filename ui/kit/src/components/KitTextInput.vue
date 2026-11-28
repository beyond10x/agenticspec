<script setup lang="ts">
// A one-line text or number input, named by the KitField around it.
withDefaults(
  defineProps<{ id: string; modelValue?: string; type?: 'text' | 'number'; placeholder?: string; disabled?: boolean }>(),
  { modelValue: '', type: 'text', placeholder: '', disabled: false },
);
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();
</script>

<template>
  <input
    :id="id"
    class="kit-text-input"
    :type="type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>

<style scoped>
.kit-text-input {
  width: 100%;
  height: var(--kit-control-height);
  padding: 0 var(--kit-space-3);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-md);
  background: var(--kit-surface);
  color: var(--kit-heading);
  font: inherit;
  font-size: var(--kit-text-sm);
  transition:
    border-color var(--kit-motion),
    box-shadow var(--kit-motion);
}
.kit-text-input::placeholder {
  color: var(--kit-muted);
}
.kit-text-input:hover:not(:disabled) {
  border-color: var(--kit-line);
}
.kit-text-input:focus {
  outline: none;
  border-color: var(--kit-accent);
  box-shadow: 0 0 0 3px var(--kit-accent-soft);
}
</style>
