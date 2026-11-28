<script setup lang="ts">
// A page of reading width: its heading, a lead, its actions and its content.
defineProps<{ title: string; lead?: string }>();
</script>

<template>
  <section class="kit-page">
    <header class="head">
      <div>
        <h2 class="title">{{ title }}</h2>
        <p v-if="lead" class="lead">{{ lead }}</p>
      </div>
      <div class="actions"><slot name="actions" /></div>
    </header>
    <slot />
  </section>
</template>

<style scoped>
.kit-page {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-4);
  max-width: 72rem;
  margin: 0 auto;
  padding: var(--kit-space-5) var(--kit-space-4);
}
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--kit-space-3);
}
.title {
  font-size: var(--kit-text-2xl);
  font-weight: var(--kit-weight-title);
}
.lead {
  margin-top: var(--kit-space-1);
  max-width: 44rem;
  color: var(--kit-muted);
}
.actions {
  display: flex;
  gap: var(--kit-space-2);
}
</style>
