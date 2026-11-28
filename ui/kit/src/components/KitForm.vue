<script setup lang="ts">
// A form: its fields, its one submit button, and the answer below it, announced politely.
import KitButton from './KitButton.vue';

withDefaults(defineProps<{ submitLabel?: string; busy?: boolean; disabled?: boolean }>(), {
  submitLabel: 'Send',
  busy: false,
  disabled: false,
});
const emit = defineEmits<{ submit: [] }>();
</script>

<template>
  <form class="kit-form" novalidate @submit.prevent="emit('submit')">
    <div class="fields"><slot /></div>
    <div class="footer">
      <slot name="footer" />
      <KitButton type="submit" tone="primary" :busy="busy" :disabled="disabled">{{ submitLabel }}</KitButton>
    </div>
    <div class="result" aria-live="polite"><slot name="result" /></div>
  </form>
</template>

<style scoped>
.kit-form {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
  min-width: 0;
}
.fields {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
}
.footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--kit-space-2);
}
.result:empty {
  display: none;
}
</style>
