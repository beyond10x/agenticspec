<script setup lang="ts">
// A panel: a card with a title that may collapse it, and the life of the read behind it
// (loading, empty, failed, ready). `fill` takes the rest of the height; `bare` drops the card.
import { ref, useId, watch } from 'vue';
import KitEmpty from './KitEmpty.vue';
import KitError from './KitError.vue';
import KitIcon from './KitIcon.vue';
import KitLoading from './KitLoading.vue';

const props = withDefaults(
  defineProps<{
    name: string;
    title?: string;
    collapsible?: boolean;
    collapsed?: boolean;
    state?: 'idle' | 'loading' | 'ready' | 'empty' | 'failed';
    error?: string;
    emptyTitle?: string;
    emptyMessage?: string;
    failedMessage?: string;
    fill?: boolean;
    bare?: boolean;
  }>(),
  {
    title: '',
    collapsible: false,
    collapsed: false,
    state: undefined,
    error: '',
    emptyTitle: '',
    emptyMessage: 'Nothing here yet.',
    failedMessage: 'This could not be read.',
    fill: false,
    bare: false,
  },
);
const emit = defineEmits<{ retry: [] }>();

const id = useId();
const open = ref(!props.collapsed);
watch(
  () => props.collapsed,
  (c) => (open.value = !c),
);
</script>

<template>
  <section
    class="kit-panel"
    :class="{ fill, bare, closed: !open }"
    :data-panel="name"
    :data-state="state"
    :aria-labelledby="title ? `${id}-title` : undefined"
    :aria-label="title ? undefined : name"
  >
    <header v-if="title" class="head">
      <h2 :id="`${id}-title`" class="title">
        <button
          v-if="collapsible"
          type="button"
          class="toggle"
          :aria-expanded="open"
          :aria-controls="`${id}-body`"
          @click="open = !open"
        >
          <KitIcon name="chevron-right" class="chevron" />
          <span>{{ title }}</span>
        </button>
        <span v-else>{{ title }}</span>
      </h2>
      <div class="head-extra"><slot name="head" /></div>
    </header>
    <div v-show="open" :id="`${id}-body`" class="body">
      <KitLoading v-if="state === 'loading'" />
      <KitError v-else-if="state === 'failed'" :message="failedMessage" :detail="error" @retry="emit('retry')" />
      <KitEmpty v-else-if="state === 'empty'" :title="emptyTitle" :message="emptyMessage" />
      <KitEmpty v-else-if="state === 'idle'" message="Waiting for a choice." />
      <slot v-else />
    </div>
  </section>
</template>

<style scoped>
.kit-panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--kit-surface);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-card);
  box-shadow: var(--kit-shadow-sm);
}
.kit-panel.bare {
  background: transparent;
  border: 0;
  border-radius: 0;
  box-shadow: none;
}
.kit-panel.fill {
  flex: 1 1 auto;
  min-height: 0;
}
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--kit-space-2);
  padding: var(--kit-space-2) var(--kit-space-3);
  border-bottom: 1px solid var(--kit-hairline);
}
.closed .head {
  border-bottom-color: transparent;
}
.title {
  font-size: var(--kit-text-sm);
  font-weight: var(--kit-weight-strong);
  letter-spacing: 0.01em;
}
.toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  margin: -0.2rem -0.35rem;
  padding: 0.2rem 0.35rem;
  border: 0;
  border-radius: var(--kit-radius-sm);
  background: transparent;
  color: inherit;
  font: inherit;
  cursor: pointer;
}
.toggle:hover {
  background: var(--kit-surface-muted);
}
.chevron {
  color: var(--kit-muted);
  transform: rotate(90deg);
  transition: transform var(--kit-motion);
}
.closed .chevron {
  transform: rotate(0deg);
}
.body {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
  min-width: 0;
  padding: var(--kit-space-3);
}
.bare > .body {
  padding: 0;
}
.fill > .body {
  flex: 1 1 auto;
  min-height: 0;
}
</style>
