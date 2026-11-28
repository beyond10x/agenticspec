<script setup lang="ts">
// A box of code: editable (a textarea over its highlighted copy) or read-only. `highlight` cuts
// the text into tokens whose kind picks a colour of the tokens; without it the text is plain.
import { computed, ref } from 'vue';
import type { CodeToken } from '../types';

const props = withDefaults(
  defineProps<{
    id?: string;
    modelValue?: string;
    /** Names the box when no KitField label does. */
    label?: string;
    rows?: number;
    readonly?: boolean;
    placeholder?: string;
    highlight?: (text: string) => CodeToken[];
  }>(),
  { id: undefined, modelValue: '', label: undefined, rows: 8, readonly: false, placeholder: '', highlight: undefined },
);
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const tokens = computed<CodeToken[]>(() => (props.highlight ? props.highlight(props.modelValue) : [{ text: props.modelValue }]));
const mirror = ref<HTMLElement | null>(null);

function onScroll(e: Event): void {
  const t = e.target as HTMLTextAreaElement;
  if (mirror.value) {
    mirror.value.scrollTop = t.scrollTop;
    mirror.value.scrollLeft = t.scrollLeft;
  }
}
</script>

<template>
  <pre v-if="readonly" :id="id" class="kit-code-box readonly" :aria-label="label" tabindex="0"><span
    v-for="(t, i) in tokens"
    :key="i"
    :class="t.kind ? `tok-${t.kind}` : undefined"
  >{{ t.text }}</span></pre>
  <div v-else class="kit-code-box editable" :class="{ highlighted: !!highlight }" :style="{ '--rows': rows }">
    <pre v-if="highlight" ref="mirror" class="mirror" aria-hidden="true"><span
      v-for="(t, i) in tokens"
      :key="i"
      :class="t.kind ? `tok-${t.kind}` : undefined"
    >{{ t.text }}</span>{{ '\n' }}</pre>
    <textarea
      :id="id"
      class="input"
      :value="modelValue"
      :rows="rows"
      :aria-label="label"
      :placeholder="placeholder"
      spellcheck="false"
      autocapitalize="off"
      autocomplete="off"
      wrap="off"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
      @scroll="onScroll"
    />
  </div>
</template>

<style scoped>
.kit-code-box {
  margin: 0;
  font-family: var(--kit-mono);
  font-size: 0.8rem;
  line-height: 1.55;
  tab-size: 2;
}
.readonly {
  padding: var(--kit-space-2) var(--kit-space-3);
  overflow: auto;
  background: var(--kit-code-bg);
  border: 1px solid var(--kit-code-border);
  border-radius: var(--kit-radius-md);
  color: var(--kit-text);
  white-space: pre;
}
.editable {
  position: relative;
  background: var(--kit-code-bg);
  border: 1px solid var(--kit-code-border);
  border-radius: var(--kit-radius-md);
  transition:
    border-color var(--kit-motion),
    box-shadow var(--kit-motion);
}
.editable:focus-within {
  border-color: var(--kit-accent);
  box-shadow: 0 0 0 3px var(--kit-accent-soft);
}
.mirror,
.input {
  margin: 0;
  padding: var(--kit-space-2) var(--kit-space-3);
  font: inherit;
  line-height: inherit;
  white-space: pre;
  overflow-wrap: normal;
  letter-spacing: normal;
  tab-size: 2;
}
.mirror {
  position: absolute;
  inset: 0;
  overflow: hidden;
  color: var(--kit-text);
  pointer-events: none;
}
.input {
  position: relative;
  display: block;
  width: 100%;
  min-height: calc(var(--rows) * 1.55em + 2 * var(--kit-space-2));
  border: 0;
  border-radius: inherit;
  background: transparent;
  color: var(--kit-heading);
  resize: vertical;
  overflow: auto;
}
.highlighted .input {
  color: transparent;
  caret-color: var(--kit-heading);
}
.highlighted .input::selection {
  background: var(--kit-accent-soft);
  color: transparent;
}
.input::placeholder {
  color: var(--kit-muted);
}
.input:focus {
  outline: none;
}
.tok-verb {
  color: var(--kit-code-verb);
}
.tok-keyword {
  color: var(--kit-code-keyword);
}
.tok-string {
  color: var(--kit-code-string);
}
.tok-number {
  color: var(--kit-code-number);
}
.tok-var {
  color: var(--kit-code-var);
}
.tok-punct {
  color: var(--kit-muted);
}
.tok-comment {
  color: var(--kit-code-comment);
  font-style: italic;
}
</style>
