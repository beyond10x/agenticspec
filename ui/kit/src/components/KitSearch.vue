<script setup lang="ts">
// A search field with its icon; the label names it and shows as its placeholder.
import { useId } from 'vue';
import KitIcon from './KitIcon.vue';

withDefaults(defineProps<{ label: string; modelValue?: string; placeholder?: string }>(), {
  modelValue: '',
  placeholder: '',
});
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();
const id = useId();
</script>

<template>
  <div class="kit-search">
    <label class="kit-sr-only" :for="id">{{ label }}</label>
    <KitIcon name="search" class="icon" />
    <input
      :id="id"
      type="search"
      class="input"
      :value="modelValue"
      :placeholder="placeholder || label"
      autocomplete="off"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
  </div>
</template>

<style scoped>
.kit-search {
  position: relative;
  display: flex;
  align-items: center;
  min-width: 0;
}
.icon {
  position: absolute;
  left: 0.65rem;
  color: var(--kit-muted);
  pointer-events: none;
}
.input {
  width: 100%;
  height: var(--kit-control-height-sm);
  padding: 0 var(--kit-space-2) 0 2rem;
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-pill);
  background: var(--kit-surface);
  color: var(--kit-heading);
  font: inherit;
  font-size: var(--kit-text-sm);
  appearance: none;
  transition:
    border-color var(--kit-motion),
    box-shadow var(--kit-motion);
}
.input::placeholder {
  color: var(--kit-muted);
}
.input:focus {
  outline: none;
  border-color: var(--kit-accent);
  box-shadow: 0 0 0 3px var(--kit-accent-soft);
}
.input::-webkit-search-cancel-button {
  appearance: none;
}
</style>
