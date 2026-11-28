<script setup lang="ts">
// A select drawn by the kit: a combobox button and a listbox of options (the ARIA select-only
// combobox pattern). Keyboard: ArrowDown/ArrowUp/Enter/Space open it; in the list ArrowDown and
// ArrowUp move, Home and End jump, a letter moves to the next option starting with it, Enter or
// Space choose, Escape and Tab close.
import { computed, nextTick, onBeforeUnmount, ref, useId, watch } from 'vue';
import KitIcon from './KitIcon.vue';
import type { KitOption, KitScalar } from '../types';

const props = withDefaults(
  defineProps<{
    label: string;
    options: KitOption[];
    modelValue?: KitScalar;
    placeholder?: string;
    /** Show the label above the control instead of only naming it. */
    showLabel?: boolean;
    disabled?: boolean;
  }>(),
  { modelValue: null, placeholder: 'Choose…', showLabel: false, disabled: false },
);
const emit = defineEmits<{ 'update:modelValue': [value: KitScalar] }>();

const id = useId();
const listId = `${id}-list`;
const labelId = `${id}-label`;
const open = ref(false);
const active = ref(-1);
const root = ref<HTMLElement | null>(null);
const list = ref<HTMLElement | null>(null);
const button = ref<HTMLButtonElement | null>(null);

const selectedIndex = computed(() => props.options.findIndex((o) => o.value === props.modelValue));
const selected = computed(() => (selectedIndex.value >= 0 ? props.options[selectedIndex.value] : null));
const optionId = (i: number) => `${id}-opt-${i}`;

function show(at?: number): void {
  if (props.disabled || props.options.length === 0) return;
  open.value = true;
  active.value = at ?? (selectedIndex.value >= 0 ? selectedIndex.value : 0);
  void nextTick(scrollActive);
}

function hide(focusButton = true): void {
  open.value = false;
  if (focusButton) button.value?.focus();
}

function choose(i: number): void {
  const o = props.options[i];
  if (!o) return;
  emit('update:modelValue', o.value);
  hide();
}

function scrollActive(): void {
  const el = list.value?.children[active.value] as HTMLElement | undefined;
  el?.scrollIntoView?.({ block: 'nearest' });
}

function move(to: number): void {
  const n = props.options.length;
  if (n === 0) return;
  active.value = Math.max(0, Math.min(n - 1, to));
  void nextTick(scrollActive);
}

let typed = '';
let typedAt = 0;
function typeahead(ch: string): void {
  const now = Date.now();
  typed = now - typedAt > 700 ? ch : typed + ch;
  typedAt = now;
  const n = props.options.length;
  const start = typed.length === 1 ? active.value + 1 : active.value;
  for (let k = 0; k < n; k++) {
    const i = (start + k + n) % n;
    if (props.options[i].label.toLowerCase().startsWith(typed.toLowerCase())) {
      if (!open.value) show(i);
      else move(i);
      return;
    }
  }
}

function onKey(e: KeyboardEvent): void {
  if (!open.value) {
    if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(e.key)) {
      e.preventDefault();
      show(e.key === 'ArrowUp' && selectedIndex.value < 0 ? props.options.length - 1 : undefined);
    } else if (e.key.length === 1 && /\S/.test(e.key)) {
      typeahead(e.key);
    }
    return;
  }
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      move(active.value + 1);
      break;
    case 'ArrowUp':
      e.preventDefault();
      move(active.value - 1);
      break;
    case 'Home':
      e.preventDefault();
      move(0);
      break;
    case 'End':
      e.preventDefault();
      move(props.options.length - 1);
      break;
    case 'PageDown':
      e.preventDefault();
      move(active.value + 8);
      break;
    case 'PageUp':
      e.preventDefault();
      move(active.value - 8);
      break;
    case 'Enter':
    case ' ':
      e.preventDefault();
      choose(active.value);
      break;
    case 'Escape':
      e.preventDefault();
      hide();
      break;
    case 'Tab':
      hide(false);
      break;
    default:
      if (e.key.length === 1 && /\S/.test(e.key)) typeahead(e.key);
  }
}

function onOutside(e: PointerEvent): void {
  if (open.value && root.value && !root.value.contains(e.target as Node)) hide(false);
}
watch(open, (o) => {
  if (o) document.addEventListener('pointerdown', onOutside, true);
  else document.removeEventListener('pointerdown', onOutside, true);
});
onBeforeUnmount(() => document.removeEventListener('pointerdown', onOutside, true));
</script>

<template>
  <div ref="root" class="kit-select" :class="{ open, 'with-label': showLabel }">
    <span :id="labelId" class="label" :class="{ 'kit-sr-only': !showLabel }">{{ label }}</span>
    <button
      ref="button"
      type="button"
      class="control"
      role="combobox"
      aria-haspopup="listbox"
      :aria-expanded="open"
      :aria-controls="listId"
      :aria-labelledby="labelId"
      :aria-activedescendant="open && active >= 0 ? optionId(active) : undefined"
      :disabled="disabled"
      :data-value="modelValue ?? ''"
      @click="open ? hide() : show()"
      @keydown="onKey"
    >
      <span class="value" :class="{ placeholder: !selected }">{{ selected ? selected.label : placeholder }}</span>
      <KitIcon name="chevron-down" class="chevron" />
    </button>
    <ul v-show="open" :id="listId" ref="list" class="list" role="listbox" :aria-labelledby="labelId" tabindex="-1">
      <li
        v-for="(o, i) in options"
        :id="optionId(i)"
        :key="String(o.value)"
        role="option"
        class="option"
        :class="{ active: i === active }"
        :aria-selected="o.value === modelValue"
        @pointerenter="active = i"
        @click="choose(i)"
      >
        <span class="option-label">{{ o.label }}</span>
        <KitIcon v-if="o.value === modelValue" name="check" class="tick" />
      </li>
    </ul>
  </div>
</template>

<style scoped>
.kit-select {
  position: relative;
  display: inline-flex;
  flex-direction: column;
  gap: var(--kit-space-1);
  min-width: 14rem;
  max-width: 100%;
}
.label {
  font-size: var(--kit-text-xs);
  font-weight: var(--kit-weight-strong);
  color: var(--kit-muted);
}
.control {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--kit-space-2);
  width: 100%;
  height: var(--kit-control-height);
  padding: 0 var(--kit-space-2) 0 var(--kit-space-3);
  background: var(--kit-surface);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-md);
  box-shadow: var(--kit-shadow-sm);
  color: var(--kit-heading);
  font: inherit;
  font-size: var(--kit-text-sm);
  font-weight: var(--kit-weight-medium);
  text-align: left;
  cursor: pointer;
  transition:
    border-color var(--kit-motion),
    box-shadow var(--kit-motion);
}
.control:hover:not(:disabled) {
  border-color: var(--kit-line);
}
.open .control {
  border-color: var(--kit-accent);
  box-shadow: 0 0 0 3px var(--kit-accent-soft);
}
.control:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}
.value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.placeholder {
  color: var(--kit-muted);
  font-weight: 400;
}
.chevron {
  color: var(--kit-muted);
  transition: transform var(--kit-motion);
}
.open .chevron {
  transform: rotate(180deg);
}
.list {
  position: absolute;
  top: calc(100% + 0.3rem);
  left: 0;
  z-index: 40;
  min-width: 100%;
  max-width: min(28rem, 92vw);
  max-height: min(20rem, 60vh);
  overflow-y: auto;
  margin: 0;
  padding: var(--kit-space-1);
  list-style: none;
  background: var(--kit-surface-raised);
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-lg);
  box-shadow: var(--kit-shadow-lg);
}
.with-label .list {
  top: calc(100% + 0.3rem);
}
.option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--kit-space-2);
  padding: 0.45rem var(--kit-space-2);
  border-radius: var(--kit-radius-sm);
  color: var(--kit-text);
  font-size: var(--kit-text-sm);
  cursor: pointer;
  white-space: nowrap;
}
.option.active {
  background: var(--kit-surface-muted);
  color: var(--kit-heading);
}
.option[aria-selected='true'] {
  font-weight: var(--kit-weight-strong);
  color: var(--kit-heading);
}
.option-label {
  overflow: hidden;
  text-overflow: ellipsis;
}
.tick {
  color: var(--kit-accent);
}
</style>
