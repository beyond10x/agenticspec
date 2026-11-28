<script setup lang="ts">
// Nothing to show yet: what is missing and what to do about it.
import KitIcon from './KitIcon.vue';

defineProps<{ title?: string; message?: string }>();
</script>

<template>
  <div class="kit-empty" data-state="empty">
    <KitIcon name="inbox" class="icon" />
    <p v-if="title" class="title">{{ title }}</p>
    <p v-if="message" class="message">{{ message }}</p>
    <slot />
  </div>
</template>

<style scoped>
.kit-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  padding: var(--kit-space-4) var(--kit-space-3);
  color: var(--kit-muted);
  text-align: center;
  font-size: var(--kit-text-sm);
}
.icon {
  font-size: 1.4rem;
  color: var(--kit-line);
}
.title {
  color: var(--kit-heading);
  font-weight: var(--kit-weight-strong);
}
</style>
