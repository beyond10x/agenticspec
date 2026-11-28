<script setup lang="ts">
// A canvas that pans and zooms its content: drag to pan, the wheel or a pinch to zoom about the
// pointer, the buttons to zoom in, out, fit the view or go to 100 %. Focused, + and - zoom, 0 goes
// to 100 %, F fits and the arrow keys pan. It opens at a readable scale (zoomView.initialView); a
// click that ended a drag does not reach the content.
import { onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue';
import KitIcon from './KitIcon.vue';
import { clamp, fitView, initialView, place, wheelFactor, zoomAt, type View } from '../zoom';

const props = withDefaults(
  defineProps<{
    label: string;
    contentWidth: number;
    contentHeight: number;
    /** The smallest scale the canvas opens at; below it the content is not fitted. */
    readable?: number;
    min?: number;
    max?: number;
    pad?: number;
  }>(),
  { readable: 0.85, min: 0.1, max: 4, pad: 24 },
);
const emit = defineEmits<{ 'update:scale': [scale: number] }>();

const viewport = ref<HTMLElement | null>(null);
const view = reactive<View>({ scale: 1, x: 0, y: 0 });
const size = reactive({ width: 0, height: 0 });
const placed = ref(false);
const panning = ref(false);
const limits = () => ({ min: props.min, max: props.max });
const content = () => ({ width: props.contentWidth, height: props.contentHeight });

function set(v: View): void {
  view.scale = v.scale;
  view.x = v.x;
  view.y = v.y;
  emit('update:scale', v.scale);
}

function placeFirst(): void {
  if (placed.value || size.width === 0 || props.contentWidth <= 0) return;
  set(initialView(size, content(), props.readable, props.pad));
  placed.value = true;
}

function zoomBy(factor: number, px = size.width / 2, py = size.height / 2): void {
  set(zoomAt(view, factor, px, py, limits()));
}
function fit(): void {
  set(fitView(size, content(), limits(), props.pad));
}
function actual(): void {
  const centre = { x: size.width / 2, y: size.height / 2 };
  const at = zoomAt(view, 1 / view.scale, centre.x, centre.y, limits());
  set(view.scale === 1 ? place(size, content(), 1, props.pad) : at);
}
defineExpose({ zoomBy, fit, actual, view });

let observer: ResizeObserver | null = null;
onMounted(() => {
  const el = viewport.value;
  if (!el) return;
  const measure = () => {
    size.width = el.clientWidth;
    size.height = el.clientHeight;
    placeFirst();
  };
  measure();
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(measure);
    observer.observe(el);
  }
  el.addEventListener('wheel', onWheel, { passive: false });
});
onBeforeUnmount(() => {
  observer?.disconnect();
  viewport.value?.removeEventListener('wheel', onWheel);
});
watch(() => [props.contentWidth, props.contentHeight], placeFirst);

function local(e: { clientX: number; clientY: number }): { x: number; y: number } {
  const r = viewport.value?.getBoundingClientRect();
  return { x: e.clientX - (r?.left ?? 0), y: e.clientY - (r?.top ?? 0) };
}

function onWheel(e: WheelEvent): void {
  e.preventDefault();
  const p = local(e);
  if (!e.ctrlKey && Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
    set({ ...view, x: view.x - e.deltaX });
    return;
  }
  zoomBy(wheelFactor(e.deltaY, e.deltaMode, e.ctrlKey), p.x, p.y);
}

// Pointers: one drags, two pinch.
const pointers = new Map<number, { x: number; y: number }>();
let start: { x: number; y: number; vx: number; vy: number } | null = null;
let moved = false;
let pinch: { d: number } | null = null;

function onDown(e: PointerEvent): void {
  if (e.button !== 0 && e.pointerType === 'mouse') return;
  if ((e.target as HTMLElement).closest('.controls')) return;
  pointers.set(e.pointerId, local(e));
  if (pointers.size === 1) {
    const p = local(e);
    start = { x: p.x, y: p.y, vx: view.x, vy: view.y };
    moved = false;
  } else if (pointers.size === 2) {
    const [a, b] = [...pointers.values()];
    pinch = { d: Math.hypot(a.x - b.x, a.y - b.y) };
  }
}

function onMove(e: PointerEvent): void {
  if (!pointers.has(e.pointerId)) return;
  const p = local(e);
  pointers.set(e.pointerId, p);
  if (pointers.size === 2 && pinch) {
    const [a, b] = [...pointers.values()];
    const d = Math.hypot(a.x - b.x, a.y - b.y);
    if (pinch.d > 0) zoomBy(d / pinch.d, (a.x + b.x) / 2, (a.y + b.y) / 2);
    pinch.d = d;
    moved = true;
    return;
  }
  if (!start) return;
  const dx = p.x - start.x;
  const dy = p.y - start.y;
  if (!moved && Math.hypot(dx, dy) < 4) return;
  if (!moved) {
    moved = true;
    panning.value = true;
    viewport.value?.setPointerCapture?.(e.pointerId);
  }
  set({ scale: view.scale, x: start.vx + dx, y: start.vy + dy });
}

function onUp(e: PointerEvent): void {
  pointers.delete(e.pointerId);
  if (pointers.size < 2) pinch = null;
  if (pointers.size === 0) {
    start = null;
    panning.value = false;
    if (moved) {
      const swallow = (c: Event) => {
        c.stopPropagation();
        c.preventDefault();
      };
      window.addEventListener('click', swallow, { capture: true, once: true });
      setTimeout(() => window.removeEventListener('click', swallow, { capture: true }), 0);
    }
  }
}

function onKey(e: KeyboardEvent): void {
  if (e.target !== viewport.value) return;
  const step = 60;
  const keys: Record<string, () => void> = {
    '+': () => zoomBy(1.25),
    '=': () => zoomBy(1.25),
    '-': () => zoomBy(0.8),
    '0': actual,
    f: fit,
    F: fit,
    ArrowLeft: () => set({ ...view, x: view.x + step }),
    ArrowRight: () => set({ ...view, x: view.x - step }),
    ArrowUp: () => set({ ...view, y: view.y + step }),
    ArrowDown: () => set({ ...view, y: view.y - step }),
  };
  const k = keys[e.key];
  if (k) {
    e.preventDefault();
    k();
  }
}

const percent = () => `${Math.round(clamp(view.scale, 0, 99) * 100)} %`;
</script>

<template>
  <div class="kit-zoom-canvas">
    <div
      ref="viewport"
      class="viewport"
      :class="{ panning }"
      role="group"
      tabindex="0"
      :aria-label="`${label}. Drag to pan, wheel or pinch to zoom; keys + and - zoom, 0 is 100 %, F fits, arrows pan.`"
      :data-scale="view.scale.toFixed(3)"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
      @pointercancel="onUp"
      @keydown="onKey"
    >
      <div
        class="content"
        :style="{
          width: `${contentWidth}px`,
          height: `${contentHeight}px`,
          transform: `translate(${view.x}px, ${view.y}px) scale(${view.scale})`,
        }"
      >
        <slot />
      </div>
    </div>
    <div class="controls" role="toolbar" aria-label="Zoom">
      <button type="button" class="ctl" aria-label="Zoom out" title="Zoom out" @click="zoomBy(0.8)">
        <KitIcon name="minus" />
      </button>
      <output class="scale" aria-live="off">{{ percent() }}</output>
      <button type="button" class="ctl" aria-label="Zoom in" title="Zoom in" @click="zoomBy(1.25)">
        <KitIcon name="plus" />
      </button>
      <span class="sep" aria-hidden="true" />
      <button type="button" class="ctl text" title="Fit the whole graph" @click="fit">
        <KitIcon name="fit" />
        <span>Fit</span>
      </button>
      <button type="button" class="ctl text" title="Actual size" @click="actual">100 %</button>
    </div>
    <div class="overlay"><slot name="overlay" /></div>
  </div>
</template>

<style scoped>
.kit-zoom-canvas {
  position: relative;
  flex: 1 1 auto;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background-color: var(--kit-canvas);
  background-image: radial-gradient(var(--kit-hairline) 1px, transparent 1px);
  background-size: 20px 20px;
}
.viewport {
  position: absolute;
  inset: 0;
  overflow: hidden;
  cursor: grab;
  touch-action: none;
}
.viewport:focus-visible {
  outline: 2px solid var(--kit-focus);
  outline-offset: -2px;
}
.viewport.panning {
  cursor: grabbing;
}
.content {
  position: absolute;
  top: 0;
  left: 0;
  transform-origin: 0 0;
  will-change: transform;
}
.controls {
  position: absolute;
  right: var(--kit-space-3);
  bottom: var(--kit-space-3);
  z-index: 5;
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 3px;
  background: var(--kit-surface);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-md);
  box-shadow: var(--kit-shadow-md);
}
.ctl {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
  min-width: 1.9rem;
  height: 1.9rem;
  padding: 0 0.45rem;
  border: 0;
  border-radius: var(--kit-radius-sm);
  background: transparent;
  color: var(--kit-text);
  font: inherit;
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  cursor: pointer;
}
.ctl:hover {
  background: var(--kit-surface-muted);
  color: var(--kit-heading);
}
.scale {
  min-width: 3.4rem;
  color: var(--kit-muted);
  font-size: var(--kit-text-xs);
  font-variant-numeric: tabular-nums;
  text-align: center;
}
.sep {
  width: 1px;
  height: 1.2rem;
  margin: 0 2px;
  background: var(--kit-hairline);
}
.overlay {
  position: absolute;
  top: var(--kit-space-3);
  right: var(--kit-space-3);
  z-index: 6;
  max-height: calc(100% - 5rem);
  display: flex;
  pointer-events: none;
}
.overlay > :deep(*) {
  pointer-events: auto;
}
</style>
