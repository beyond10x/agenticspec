<script setup lang="ts">
// The shell of every page: a header bar with the app's title, the page's title, the page's bar
// (its filters and actions) and the theme switch; the page below it fills the rest of the
// viewport; the toasts float above. The theme is set on <html> (`global`) or on the shell alone.
// With a navigation, it stands at the left as the docs site's sidebar; on a narrow screen a
// Pages button in the header opens it above the page.
import { computed, ref, useId, watch } from 'vue';
import KitIcon from './KitIcon.vue';
import KitNav from './KitNav.vue';
import KitThemeToggle from './KitThemeToggle.vue';
import KitToaster from './KitToaster.vue';
import type { KitNavSection, KitToast } from '../types';

const props = withDefaults(
  defineProps<{
    title: string;
    pageTitle?: string;
    theme?: string;
    themes?: string[];
    toasts?: KitToast[];
    nav?: KitNavSection[];
    /** Set the theme on the document; false keeps it on the shell (the gallery shows two). */
    global?: boolean;
  }>(),
  { pageTitle: '', theme: 'light', themes: () => [], toasts: () => [], nav: () => [], global: true },
);
const emit = defineEmits<{ 'update:theme': [theme: string]; dismiss: [id: number] }>();

const id = useId();
const hasNav = computed(() => props.nav.length > 0);
const menuOpen = ref(false);
const current = computed(() => props.nav.flatMap((s) => s.items).find((i) => i.current)?.href ?? '');
watch(current, () => (menuOpen.value = false));

watch(
  () => props.theme,
  (t) => {
    if (props.global && typeof document !== 'undefined') document.documentElement.dataset.theme = t;
  },
  { immediate: true },
);
</script>

<template>
  <div class="kit-shell" :class="{ 'with-nav': hasNav }" :data-theme="global ? undefined : theme">
    <header class="header">
      <button
        v-if="hasNav"
        type="button"
        class="menu"
        :aria-expanded="menuOpen"
        :aria-controls="`${id}-nav`"
        @click="menuOpen = !menuOpen"
      >
        <KitIcon name="menu" />
        <span>Pages</span>
      </button>
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
    <div v-if="hasNav" class="frame">
      <aside :id="`${id}-nav`" class="side" :class="{ open: menuOpen }">
        <KitNav :sections="nav" />
      </aside>
      <main class="main">
        <slot />
      </main>
    </div>
    <main v-else class="main">
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
.menu {
  display: none;
  align-items: center;
  gap: 0.35rem;
  height: var(--kit-control-height-sm);
  padding: 0 0.6rem;
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-sm);
  background: var(--kit-surface);
  color: var(--kit-text);
  font: inherit;
  font-size: var(--kit-text-sm);
  font-weight: var(--kit-weight-strong);
  cursor: pointer;
}
.menu:hover {
  background: var(--kit-surface-muted);
}
.frame {
  display: grid;
  grid-template-columns: var(--kit-nav-width) minmax(0, 1fr);
  min-height: 0;
}
.side {
  min-height: 0;
  overflow-y: auto;
  background: var(--kit-surface);
  border-right: 1px solid var(--kit-hairline);
}
@media (max-width: 900px) {
  .menu {
    display: inline-flex;
  }
  .frame {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
  }
  .frame > .main {
    grid-row: 2;
  }
  .side {
    display: none;
    max-height: 55vh;
    border-right: 0;
    border-bottom: 1px solid var(--kit-hairline);
  }
  .side.open {
    display: block;
  }
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
