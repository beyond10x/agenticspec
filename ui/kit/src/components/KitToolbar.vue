<script setup lang="ts">
// A row of controls that belong together (a page's filters and actions), wrapping when narrow.
defineProps<{ label: string }>();
</script>

<template>
  <div class="kit-toolbar" role="group" :aria-label="label">
    <slot />
  </div>
</template>

<style scoped>
.kit-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--kit-space-2);
  min-width: 0;
}
</style>
