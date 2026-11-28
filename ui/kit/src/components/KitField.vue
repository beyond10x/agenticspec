<script setup lang="ts">
// A labelled field: the label names the control whose id `for` gives; a hint may follow.
defineProps<{ label: string; for: string; hint?: string }>();
</script>

<template>
  <div class="kit-field">
    <label class="label" :for="$props.for">{{ label }}</label>
    <slot />
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>

<style scoped>
.kit-field {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  min-width: 0;
}
.label {
  color: var(--kit-heading);
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
}
.hint {
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
}
</style>
