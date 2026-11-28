<script setup lang="ts">
// The pages of the operator app, all drawn from the layout: `#/` is the layout's home, and
// `#/<page, its dots as slashes>` is the overview of a level or the page of a part. A part page
// shows the actions the action set (R5) of its scope allows (all of them when the adapter has no
// R5) and sends an action as a command (R4) at that scope.
import { computed, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { KitPanel, KitShell, KitStack, KitTable, type FieldDef, type KitNavSection, type KitToast, type Tone } from '@kit';
import { AgenticOverview, AgenticPartPage, type Client, type CommandDetail, type LayoutLevel, type LayoutPage, type NavigateDetail } from '../../views/src/index';
import type { Data } from './data';

const props = defineProps<{ data: Data; client: Client }>();
const layout = computed(() => props.data.layout);
const architecture = computed(() => props.data.config.architecture);

const hash = ref(location.hash);
const onHash = () => (hash.value = location.hash);
window.addEventListener('hashchange', onHash);
onBeforeUnmount(() => window.removeEventListener('hashchange', onHash));

const href = (page: string) => (page === layout.value.home ? '#/' : `#/${page.split('.').join('/')}`);
const name = computed(() => hash.value.replace(/^#\/?/, '').split('/').filter(Boolean).join('.') || layout.value.home);

type Shown = { kind: 'overview'; level: LayoutLevel } | { kind: 'part'; level: LayoutLevel; page: LayoutPage } | { kind: 'missing' };
const shown = computed<Shown>(() => {
  for (const level of layout.value.levels) {
    if (level.overview === name.value) return { kind: 'overview', level };
    const page = level.pages.find((p) => p.page === name.value);
    if (page) return { kind: 'part', level, page };
  }
  return { kind: 'missing' };
});
const pageTitle = computed(() => {
  const s = shown.value;
  return s.kind === 'part' ? s.page.label : s.kind === 'overview' ? s.level.name : 'Not found';
});

const nav = computed<KitNavSection[]>(() =>
  layout.value.levels.map((level) => ({
    label: level.name,
    items: [
      { label: 'Overview', href: href(level.overview), current: name.value === level.overview },
      ...level.parts
        .filter((p) => p.page !== level.overview)
        .map((p) => ({ label: p.label, href: href(p.page), current: name.value === p.page })),
    ],
  })),
);

const prefersDark = typeof matchMedia === 'function' && matchMedia('(prefers-color-scheme: dark)').matches;
const theme = ref(document.documentElement.dataset.theme || (prefersDark ? 'dark' : 'light'));

const toasts = reactive<KitToast[]>([]);
let next = 1;
function toast(tone: Tone, text: string): void {
  toasts.push({ id: next++, tone, text });
}
function dismiss(id: number): void {
  const i = toasts.findIndex((t) => t.id === id);
  if (i >= 0) toasts.splice(i, 1);
}

const scopeOf = (part: string) => `/scopes/${[architecture.value, part].map(encodeURIComponent).join('/')}`;

// The action set of the part shown: null while it is read or when the adapter has none.
const allowed = ref<string[] | null>(null);
watch(
  () => (shown.value.kind === 'part' ? shown.value.page.part : ''),
  async (part) => {
    allowed.value = null;
    if (!part) return;
    try {
      const set = await props.client.scope(scopeOf(part)).actions();
      if (set && part === (shown.value.kind === 'part' ? shown.value.page.part : '')) allowed.value = set.filter((a) => a.allowed).map((a) => a.command);
    } catch {
      allowed.value = null;
    }
  },
  { immediate: true },
);

function navigate(e: Event): void {
  const d = (e as CustomEvent<NavigateDetail>).detail;
  e.preventDefault();
  location.hash = href(d.page);
}

async function command(e: Event): Promise<void> {
  const d = (e as CustomEvent<CommandDetail>).detail;
  const s = shown.value;
  const label = (s.kind === 'part' ? s.page.actions.find((a) => a.name === d.action)?.label : undefined) ?? d.command;
  try {
    const answer = await props.client.scope(scopeOf(d.part)).command(d.command as never, d.input as never);
    if (answer.ok) toast('success', `${label}: done`);
    else toast('warning', `${label}: refused`);
  } catch (err) {
    toast('danger', `${label}: ${err instanceof Error ? err.message : String(err)}`);
  }
}

const portColumns: FieldDef[] = [
  { field: 'part', label: 'Part', as: 'text' },
  { field: 'port', label: 'Port', as: 'text' },
  { field: 'kind', label: 'Kind', as: 'badge' },
  { field: 'protocol', label: 'Protocol', as: 'text' },
  { field: 'instances', label: 'Instances', as: 'number' },
];
const portRows = computed(() => {
  const kinds = new Map((props.data.ports?.ports ?? []).map((p) => [p.name, p.kind]));
  return (props.data.contract?.ports ?? []).map((p) => ({ ...p, kind: kinds.get(p.port) ?? '', key: `${p.part} ${p.port}` }));
});
</script>

<template>
  <KitShell
    :title="data.config.title"
    :page-title="pageTitle"
    :theme="theme"
    :themes="['light', 'dark']"
    :nav="nav"
    :toasts="toasts"
    @update:theme="theme = $event"
    @dismiss="dismiss"
  >
    <div class="operator" @agentic-navigate="navigate" @agentic-command="command">
      <KitStack v-if="shown.kind === 'overview'">
        <AgenticOverview :key="shown.level.path" :architecture="architecture" :level="shown.level.path" :layout="layout" />
        <KitPanel v-if="shown.level.path === '' && portRows.length" name="contract" title="Contract" collapsible>
          <KitTable label="Contract" :columns="portColumns" :rows="portRows" row-key="key" />
        </KitPanel>
      </KitStack>
      <AgenticPartPage
        v-else-if="shown.kind === 'part'"
        :key="shown.page.page"
        :architecture="architecture"
        :part="shown.page.part"
        :layout="layout"
        :allowed="allowed"
      />
      <KitStack v-else>
        <p class="missing">No page is named {{ name }}. <a :href="href(layout.home)">Open the home page</a>.</p>
      </KitStack>
    </div>
  </KitShell>
</template>

<style scoped>
.missing a {
  color: var(--kit-accent);
}
</style>
