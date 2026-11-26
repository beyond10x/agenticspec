<script setup lang="ts">
import { computed, ref } from 'vue';
import { route, go } from './router.ts';
import { toggleTheme, theme } from './theme.ts';
import type { LiveState } from './client.ts';
import Overview from './pages/Overview.vue';
import SessionPage from './pages/SessionPage.vue';

const live = ref<LiveState>('connecting');
const embed = computed(() => route.value.page.startsWith('embed.'));
const liveText = computed(() => ({ connecting: 'Connecting', live: 'Live', offline: 'Offline' })[live.value]);
const themeLabel = computed(() => (theme.value === 'dark' ? 'Use light theme' : 'Use dark theme'));
</script>

<template>
  <div class="app" :class="{ embed }">
    <header v-if="!embed" class="bar">
      <div class="bar-inner">
        <a href="/" class="brand" @click.prevent="go({ page: 'overview' })">
          <svg width="18" height="18" viewBox="0 0 18 18" aria-hidden="true">
            <rect x="1" y="3" width="16" height="3" rx="1.5" class="b1" />
            <rect x="1" y="7.5" width="10" height="3" rx="1.5" class="b2" />
            <rect x="5" y="12" width="9" height="3" rx="1.5" class="b3" />
          </svg>
          <span>agentspec</span>
        </a>
        <div class="right">
          <span class="live" :class="live" :title="`Updates: ${liveText}`">
            <i class="dot" />{{ liveText }}
          </span>
          <button type="button" class="icon-button" :aria-label="themeLabel" :title="themeLabel" @click="toggleTheme">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.5" />
              <path d="M8 2a6 6 0 0 1 0 12z" fill="currentColor" />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <main class="main">
      <Overview v-if="route.page === 'overview'" @live="live = $event" />
      <SessionPage
        v-else
        :key="route.sessionId"
        :session-id="route.sessionId"
        :embed="embed"
        :part="route.page === 'embed.timeline' ? 'timeline' : route.page === 'embed.events' ? 'events' : 'all'"
        @live="live = $event"
      />
    </main>
  </div>
</template>

<style scoped>
.bar {
  position: sticky;
  top: 0;
  z-index: 10;
  background: color-mix(in srgb, var(--plane) 86%, transparent);
  backdrop-filter: saturate(1.4) blur(10px);
  border-bottom: 1px solid var(--grid);
}
.bar-inner,
.main {
  max-width: 1280px;
  margin: 0 auto;
  padding: 0 32px;
}
.bar-inner {
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.brand {
  display: inline-flex;
  align-items: center;
  gap: 9px;
  font-weight: 600;
  font-size: 15px;
  letter-spacing: -0.01em;
}
.b1 {
  fill: var(--s1);
}
.b2 {
  fill: var(--s2);
}
.b3 {
  fill: var(--s3);
}
.right {
  display: flex;
  align-items: center;
  gap: 8px;
}
.live {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12.5px;
  color: var(--ink-2);
  padding: 0 6px;
}
.live .dot {
  background: var(--muted);
}
.live.live .dot {
  background: var(--good);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--good) 22%, transparent);
}
.live.offline .dot {
  background: var(--critical);
}
.main {
  padding-top: 28px;
  padding-bottom: 64px;
}
.embed .main {
  max-width: none;
  padding: 16px;
}
@media (max-width: 720px) {
  .bar-inner,
  .main {
    padding-left: 16px;
    padding-right: 16px;
  }
  .main {
    padding-top: 20px;
  }
}
</style>
