<script setup lang="ts">
// Host slot `PatchEditor`: the rules of a patch, edited with their syntax marked.
import { KitCodeBox } from '@kit';
import type { PatchEditorProps } from '../generated/hosts';
import { tokens } from './patch';

defineProps<PatchEditorProps>();
const emit = defineEmits<{ 'update:modelValue': [value: string] }>();
</script>

<template>
  <KitCodeBox
    :id="id"
    :model-value="modelValue"
    :rows="7"
    :highlight="tokens"
    placeholder='set(P, count, 3) :- part{id: P, block: "agent"}.'
    data-slot="patch-editor"
    @update:model-value="emit('update:modelValue', $event)"
  />
</template>
