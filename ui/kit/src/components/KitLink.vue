<script setup lang="ts">
// A link in the accent colour, underlined on hover.
defineProps<{ href: string }>();
</script>

<template>
  <a class="kit-link" :href="href"><slot /></a>
</template>

<style scoped>
.kit-link {
  color: var(--kit-accent);
  font-weight: var(--kit-weight-strong);
  text-decoration: none;
  text-underline-offset: 0.2em;
}
.kit-link:hover {
  color: var(--kit-accent-strong);
  text-decoration: underline;
}
</style>
