<script setup lang="ts">
// The shell of every page: a header bar with the app's title, the page's title, the page's bar
// (its filters and actions) and the theme switch; the page below it fills the rest of the
// viewport; the toasts float above. The theme is set on <html> (`global`) or on the shell alone.
import { watch } from 'vue';
import KitThemeToggle from './KitThemeToggle.vue';
import KitToaster from './KitToaster.vue';
import type { KitToast } from '../types';

const props = withDefaults(
  defineProps<{
    title: string;
    pageTitle?: string;
    theme?: string;
    themes?: string[];
    toasts?: KitToast[];
    /** Set the theme on the document; false keeps it on the shell (the gallery shows two). */
    global?: boolean;
  }>(),
  { pageTitle: '', theme: 'light', themes: () => [], toasts: () => [], global: true },
);
const emit = defineEmits<{ 'update:theme': [theme: string]; dismiss: [id: number] }>();

watch(
  () => props.theme,
  (t) => {
    if (props.global && typeof document !== 'undefined') document.documentElement.dataset.theme = t;
  },
  { immediate: true },
);
</script>

<template>
  <div class="kit-shell" :data-theme="global ? undefined : theme">
    <header class="header">
      <div class="titles">
        <span class="brand">{{ title }}</span>
        <span v-if="pageTitle" class="divider" aria-hidden="true">/</span>
        <h1 v-if="pageTitle" class="page-title">{{ pageTitle }}</h1>
      </div>
      <div class="bar">
        <slot name="bar" />
      </div>
      <KitThemeToggle
        v-if="themes.length > 1"
        class="theme"
        :model-value="theme"
        :themes="themes"
        @update:model-value="emit('update:theme', $event)"
      />
    </header>
    <main class="main">
      <slot />
    </main>
    <KitToaster :toasts="toasts" @dismiss="emit('dismiss', $event)" />
  </div>
</template>

<style scoped>
.kit-shell {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
  min-height: 0;
  background: var(--kit-canvas);
  color: var(--kit-text);
  font-family: var(--kit-font);
}
.header {
  position: relative;
  z-index: 20;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--kit-space-2) var(--kit-space-4);
  min-height: var(--kit-header-height);
  padding: var(--kit-space-1) var(--kit-space-4);
  background: var(--kit-surface);
  border-bottom: 1px solid var(--kit-hairline);
}
.titles {
  display: flex;
  align-items: baseline;
  gap: var(--kit-space-2);
  min-width: 0;
}
.brand {
  color: var(--kit-heading);
  font-size: var(--kit-text-lg);
  font-weight: var(--kit-weight-title);
  letter-spacing: var(--kit-tracking-title);
  white-space: nowrap;
}
.divider {
  color: var(--kit-line);
}
.page-title {
  overflow: hidden;
  font-size: var(--kit-text-md);
  font-weight: var(--kit-weight-strong);
  color: var(--kit-muted);
  letter-spacing: normal;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.bar {
  display: flex;
  flex: 1 1 auto;
  align-items: center;
  min-width: 0;
}
.theme {
  margin-left: auto;
}
.main {
  position: relative;
  min-height: 0;
  overflow: auto;
}
@media (max-width: 640px) {
  .header {
    padding: var(--kit-space-1) var(--kit-space-3);
  }
  .bar {
    order: 3;
    flex-basis: 100%;
  }
}
</style>
