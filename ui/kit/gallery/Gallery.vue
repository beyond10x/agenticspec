<script setup lang="ts">
// The gallery of ui/kit: every component in the light theme, then in the dark theme.
import Sheet from './Sheet.vue';
</script>

<template>
  <main class="gallery">
    <h1 class="kit-sr-only">ui/kit gallery</h1>
    <Sheet theme="light" />
    <Sheet theme="dark" />
  </main>
</template>

<style scoped>
.gallery {
  min-height: 100%;
}
</style>
