<script setup lang="ts">
// A read on its way: a few shimmering lines the size of what will come.
withDefaults(defineProps<{ label?: string; lines?: number }>(), { label: 'Loading', lines: 3 });
</script>

<template>
  <div class="kit-loading" role="status" data-state="loading">
    <span class="kit-sr-only">{{ label }}</span>
    <span v-for="i in lines" :key="i" class="line" :style="{ width: `${100 - ((i * 17) % 40)}%` }" aria-hidden="true" />
  </div>
</template>

<style scoped>
.kit-loading {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  padding: var(--kit-space-2) 0;
}
.line {
  height: 0.7rem;
  border-radius: var(--kit-radius-pill);
  background: linear-gradient(90deg, var(--kit-surface-muted), var(--kit-hairline), var(--kit-surface-muted));
  background-size: 200% 100%;
  animation: kit-shimmer 1.4s ease-in-out infinite;
}
@keyframes kit-shimmer {
  from {
    background-position: 100% 0;
  }
  to {
    background-position: -100% 0;
  }
}
@media (prefers-reduced-motion: reduce) {
  .line {
    animation: none;
  }
}
</style>
