<script setup lang="ts">
// A read that failed: what failed, the server's words, and a retry.
import KitButton from './KitButton.vue';
import KitIcon from './KitIcon.vue';

withDefaults(defineProps<{ message?: string; detail?: string; retry?: boolean }>(), {
  message: 'This could not be read.',
  detail: '',
  retry: true,
});
const emit = defineEmits<{ retry: [] }>();
</script>

<template>
  <div class="kit-error" role="alert" data-state="failed">
    <KitIcon name="alert" class="icon" />
    <div class="body">
      <p class="message">{{ message }}</p>
      <p v-if="detail" class="detail">{{ detail }}</p>
    </div>
    <KitButton v-if="retry" size="sm" tone="secondary" @click="emit('retry')">Retry</KitButton>
  </div>
</template>

<style scoped>
.kit-error {
  display: flex;
  align-items: flex-start;
  gap: var(--kit-space-2);
  padding: var(--kit-space-2) var(--kit-space-3);
  background: var(--kit-danger-soft);
  border: 1px solid var(--kit-danger);
  border-radius: var(--kit-radius-md);
  color: var(--kit-text);
  font-size: var(--kit-text-sm);
}
.icon {
  margin-top: 0.2rem;
  color: var(--kit-danger);
}
.body {
  flex: 1;
  min-width: 0;
}
.message {
  font-weight: var(--kit-weight-strong);
  color: var(--kit-heading);
}
.detail {
  color: var(--kit-text);
  overflow-wrap: anywhere;
}
</style>
