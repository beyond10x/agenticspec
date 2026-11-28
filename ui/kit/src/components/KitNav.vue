<script setup lang="ts">
// The navigation of an app, as the docs site's sidebar: a heading per section and its links, the
// current page marked in the accent colour.
import type { KitNavSection } from '../types';

withDefaults(defineProps<{ sections: KitNavSection[]; label?: string }>(), { label: 'Pages' });
</script>

<template>
  <nav class="kit-nav" :aria-label="label">
    <section v-for="s in sections" :key="s.label" class="section">
      <h2 class="heading">{{ s.label }}</h2>
      <ul class="links">
        <li v-for="i in s.items" :key="i.href">
          <a class="link" :href="i.href" :aria-current="i.current ? 'page' : undefined">{{ i.label }}</a>
        </li>
      </ul>
    </section>
  </nav>
</template>

<style scoped>
.kit-nav {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-3);
  padding: var(--kit-space-3) var(--kit-space-2);
}
.heading {
  margin: 0 0 0.2rem;
  padding: 0 var(--kit-space-2);
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}
.links {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin: 0;
  padding: 0;
  list-style: none;
}
.link {
  display: block;
  padding: 0.3rem var(--kit-space-2);
  border-radius: var(--kit-radius-sm);
  color: var(--kit-text);
  font-size: var(--kit-text-sm);
  text-decoration: none;
  transition:
    background var(--kit-motion),
    color var(--kit-motion);
}
.link:hover {
  background: var(--kit-surface-muted);
  color: var(--kit-heading);
}
.link[aria-current='page'] {
  background: var(--kit-accent-soft);
  color: var(--kit-accent-strong);
  font-weight: var(--kit-weight-strong);
}
</style>
