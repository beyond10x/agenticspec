<script setup lang="ts">
import { computed } from 'vue';

// A trend in the de-emphasis hue with the latest point in the accent. A bucket without data
// breaks the line.
const props = withDefaults(defineProps<{ values: (number | null)[]; width?: number; height?: number }>(), {
  width: 120,
  height: 28,
});

const geometry = computed(() => {
  const vs = props.values;
  const present = vs.filter((v): v is number => v !== null);
  if (present.length === 0) return null;
  const min = Math.min(...present, 0);
  const max = Math.max(...present);
  const span = max - min || 1;
  const pad = 3;
  const step = vs.length > 1 ? (props.width - pad * 2) / (vs.length - 1) : 0;
  const y = (v: number) => props.height - pad - ((v - min) / span) * (props.height - pad * 2);
  const segments: string[] = [];
  let current = '';
  vs.forEach((v, i) => {
    if (v === null) {
      if (current) segments.push(current);
      current = '';
      return;
    }
    current += `${current ? 'L' : 'M'}${(pad + i * step).toFixed(1)},${y(v).toFixed(1)}`;
  });
  if (current) segments.push(current);
  let lastIndex = -1;
  for (let i = vs.length - 1; i >= 0; i--) {
    if (vs[i] !== null) {
      lastIndex = i;
      break;
    }
  }
  return {
    paths: segments,
    last: { x: pad + lastIndex * step, y: y(vs[lastIndex] as number) },
    dots: segments.length > 1 || present.length === 1,
  };
});
</script>

<template>
  <svg :width="width" :height="height" :viewBox="`0 0 ${width} ${height}`" aria-hidden="true" class="spark">
    <template v-if="geometry">
      <path v-for="(d, i) in geometry.paths" :key="i" :d="d" />
      <circle :cx="geometry.last.x" :cy="geometry.last.y" r="3" />
    </template>
  </svg>
</template>

<style scoped>
.spark {
  display: block;
  overflow: visible;
}
path {
  fill: none;
  stroke: var(--axis);
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
}
circle {
  fill: var(--accent);
  stroke: var(--surface);
  stroke-width: 2;
}
</style>
