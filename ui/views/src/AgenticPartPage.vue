<script setup lang="ts">
// The page of one part of a derived UI, drawn from its layout: the part's actions, its instance
// panel, then a frame per domain it uses. Parameterised by `source`, `architecture`, `part` and
// `instance` only; the layout is read from the source unless the host hands it over. An action
// sends `agentic-command` and nothing else.
import { computed, ref, watch } from 'vue';
import { KitButton, KitEmpty, KitError, KitLoading, KitPanel, KitStack, KitToolbar } from '@kit';
import AgenticPanel from './AgenticPanel.vue';
import { emitCommand } from './events';
import { loadLayout, pageOf, type Layout, type LayoutAction } from './layout';

const props = defineProps<{
  source?: string;
  architecture: string;
  part: string;
  instance?: number;
  layout?: Layout;
}>();

const read = ref<Layout | null>(props.layout ?? null);
const error = ref('');
const root = ref<HTMLElement | null>(null);

watch(
  () => [props.source, props.architecture, props.layout] as const,
  async ([source, architecture, given]) => {
    error.value = '';
    if (given) {
      read.value = given;
      return;
    }
    read.value = null;
    try {
      read.value = await loadLayout(source ?? '', architecture);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  },
  { immediate: true },
);

const found = computed(() => (read.value ? pageOf(read.value, props.part) : null));

function act(a: LayoutAction): void {
  emitCommand(root.value, {
    architecture: props.architecture,
    part: props.part,
    command: a.command,
    action: a.name,
    input: {},
    instance: props.instance,
  });
}
</script>

<template>
  <section ref="root" class="agentic-part-page" :data-part="part" :data-architecture="architecture">
    <KitError v-if="error" message="The layout could not be read." :detail="error" />
    <KitLoading v-else-if="!read" label="Reading the layout" :lines="3" />
    <KitEmpty v-else-if="!found" :message="`${architecture} has no part ${part}.`" />
    <KitStack v-else>
      <header class="head">
        <h2 class="part-title">{{ found.page.label }}</h2>
        <KitToolbar v-if="found.page.actions.length" label="Actions">
          <KitButton v-for="a in found.page.actions" :key="a.name" tone="primary" @click="act(a)">{{ a.label }}</KitButton>
        </KitToolbar>
      </header>
      <template v-for="(frame, i) in found.page.frames" :key="i">
        <KitPanel v-if="frame.title" :name="`frame_${i}`" :title="frame.title" collapsible :collapsed="frame.collapsed">
          <KitStack :padded="false">
            <KitPanel v-for="s in frame.sections" :key="s.name" :name="s.name" :title="s.title" collapsible>
              <AgenticPanel :view="s.widget" :source="source" :architecture="architecture" :part="s.part" :instance="instance" :args="s.args" />
            </KitPanel>
          </KitStack>
        </KitPanel>
        <template v-else>
          <KitPanel v-for="s in frame.sections" :key="s.name" :name="s.name" :title="s.title" collapsible>
            <AgenticPanel :view="s.widget" :source="source" :architecture="architecture" :part="s.part" :instance="instance" :args="s.args" />
          </KitPanel>
        </template>
      </template>
    </KitStack>
  </section>
</template>

<style scoped>
.head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--kit-space-2);
}
.part-title {
  margin: 0;
}
</style>
