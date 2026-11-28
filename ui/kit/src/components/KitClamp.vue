<script setup lang="ts">
// A long text shown in a few lines, with "more" to read all of it and "less" to fold it again.
// The button shows only when the text is cut.
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

const props = withDefaults(defineProps<{ text: string; lines?: number }>(), { lines: 2 });

const open = ref(false);
const cut = ref(false);
const box = ref<HTMLElement | null>(null);
let observer: ResizeObserver | null = null;

function measure(): void {
  const el = box.value;
  if (!el || open.value) return;
  cut.value = el.scrollHeight > el.clientHeight + 1;
}

onMounted(() => {
  measure();
  if (typeof ResizeObserver !== 'undefined' && box.value) {
    observer = new ResizeObserver(measure);
    observer.observe(box.value);
  }
});
onBeforeUnmount(() => observer?.disconnect());
watch(
  () => props.text,
  () => {
    open.value = false;
    void nextTick(measure);
  },
);
</script>

<template>
  <div class="kit-clamp">
    <p ref="box" class="text" :class="{ open }" :style="{ '-webkit-line-clamp': open ? 'unset' : lines }">{{ text }}</p>
    <button v-if="cut || open" type="button" class="more" :aria-expanded="open" @click="open = !open">
      {{ open ? 'less' : 'more' }}
    </button>
  </div>
</template>

<style scoped>
.kit-clamp {
  min-width: 0;
}
.text {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  overflow: hidden;
  overflow-wrap: anywhere;
}
.text.open {
  display: block;
}
.more {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--kit-accent);
  font: inherit;
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  cursor: pointer;
}
.more:hover {
  text-decoration: underline;
}
</style>
