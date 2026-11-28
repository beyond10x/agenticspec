<script setup lang="ts">
// Panels one under the other, with the page's padding around them.
withDefaults(defineProps<{ padded?: boolean }>(), { padded: true });
</script>

<template>
  <div class="kit-stack" :class="{ padded }">
    <slot />
  </div>
</template>

<style scoped>
.kit-stack {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-3);
  min-width: 0;
}
.kit-stack.padded {
  padding: var(--kit-space-4);
}
@media (max-width: 640px) {
  .kit-stack.padded {
    padding: var(--kit-space-3);
  }
}
</style>
