<script setup lang="ts">
// Every component of the kit, in one theme.
import { ref } from 'vue';
import {
  KitBadge,
  KitButton,
  KitClamp,
  KitCodeBox,
  KitCopy,
  KitEmpty,
  KitError,
  KitField,
  KitForm,
  KitLink,
  KitList,
  KitLoading,
  KitMetric,
  KitNav,
  KitPage,
  KitPanel,
  KitRecord,
  KitSearch,
  KitSelect,
  KitShell,
  KitSplit,
  KitStack,
  KitSwitch,
  KitTable,
  KitTextInput,
  KitThemeToggle,
  KitToaster,
  KitToolbar,
  KitZoomCanvas,
  type CodeToken,
  type FieldDef,
  type KitNavSection,
  type KitScalar,
  type Tone,
} from '@kit';

const props = defineProps<{ theme: 'light' | 'dark' }>();

const options = [
  { value: 'architectures/workstation.yaml', label: 'workstation' },
  { value: 'architectures/solo.yaml', label: 'solo' },
  { value: 'architectures/handoff.yaml', label: 'handoff' },
  { value: 'architectures/shared-board.yaml', label: 'shared-board' },
  { value: 'architectures/ladder/09-factory.yaml', label: 'ladder/09-factory' },
];
const chosen = ref<KitScalar>('architectures/workstation.yaml');
const empty = ref<KitScalar>(null);
const name = ref('fewer workers');
const dry = ref(true);
const search = ref('');
const patch = ref('set(P, count, 3) :- part{id: P, block: "agent", count: 12}.\n% halve the pool\nset(P, capacity, 8) :- part{id: P, block: "pool"}.');
const themeChoice = ref<string>(props.theme);
const tones: Tone[] = ['neutral', 'info', 'success', 'warning', 'danger'];

function highlight(text: string): CodeToken[] {
  const out: CodeToken[] = [];
  const re = /(%[^\n]*)|("(?:[^"\\]|\\.)*")|(\b\d+\b)|(\b(?:set|add|remove|connect)\b)|(\b[A-Z]\w*\b)|(:-|[(){}.,:])|([^%"\d(){}.,:A-Z]+|.)/gs;
  for (const m of text.matchAll(re)) {
    const kind = m[1] ? 'comment' : m[2] ? 'string' : m[3] ? 'number' : m[4] ? 'verb' : m[5] ? 'var' : m[6] ? 'punct' : undefined;
    out.push({ text: m[0], kind });
  }
  return out;
}

const columns: FieldDef[] = [
  { field: 'id', label: 'Part', as: 'text' },
  { field: 'block', label: 'Block', as: 'text' },
  { field: 'count', label: 'Count', as: 'number' },
  { field: 'state', label: 'State', as: 'badge', tones: { Open: 'success', Closed: 'neutral' } },
];
const rows = [
  { id: 'operator', block: 'agent', count: 1, state: 'Open' },
  { id: 'worker', block: 'agent', count: 12, state: 'Open' },
  { id: 'builds', block: 'pool', count: null, state: 'Closed' },
];
const recordFields: FieldDef[] = [
  { field: 'revision', label: 'Revision', as: 'number' },
  { field: 'passed', label: 'Check', as: 'badge', labels: { true: 'passes', false: 'refuses' }, tones: { true: 'success', false: 'danger' } },
  { field: 'digest', label: 'Digest', as: 'text' },
  { field: 'description', label: 'Description', as: 'text' },
];
const record = {
  revision: 2,
  passed: true,
  digest: '8f14e45fceea167a5a36dedd4bea2543c4b5f0d6c2a9b1e3d7f6a0b5c4d3e2f1',
  description:
    'One workstation, one owner, an operator agent acting for the owner, coding agents and a shared build pool; every worker has its own tree and sandbox, and the gateway limits the rate of model calls.',
};
const refusalFields: FieldDef[] = [
  { field: 'outcome', label: 'Outcome', as: 'badge', tones: { Refused: 'danger' } },
  { field: 'message', label: 'Result', as: 'text' },
  { field: 'diff', label: 'Changes', as: 'list' },
];
const refusal = {
  outcome: 'Refused',
  message: 'Refused at rule 2: UnknownVerb — frobnicate is no verb of agenticspec-patch/1',
  diff: ['parts.worker.count.value: 12 → 3'],
};
const navSections: KitNavSection[] = [
  {
    label: 'Layer 4: agent',
    items: [
      { label: 'Agent', href: '#agent', current: false },
      { label: 'Harness', href: '#harness', current: true },
    ],
  },
];
const valueColumns: FieldDef[] = [
  { field: 'id', label: 'Turn', as: 'text' },
  { field: 'usage', label: 'Usage', as: 'json' },
  { field: 'state', label: 'State', as: 'badge', tones: { Open: 'success' } },
];
const valueRows = [
  { id: '7f3c2a10-4b1d-4e8a-9c2f-391e34cb0000', usage: { input_tokens: 812, output_tokens: 96 }, state: { name: 'Open', summary: 'Takes turns.' } },
];
const toasts = [
  { id: 1, tone: 'success' as Tone, text: 'Applied as revision 2.' },
  { id: 2, tone: 'danger' as Tone, text: 'NoSuchSource: architectures/none.yaml' },
];
</script>

<template>
  <section class="sheet" :data-theme="theme" :data-sheet="theme" :aria-label="`ui/kit in the ${theme} theme`">
    <KitPage :title="`ui/kit · ${theme}`" lead="Every component of the kit, drawn only from its tokens.">
      <template #actions>
        <KitThemeToggle v-model="themeChoice" :themes="['light', 'dark']" />
      </template>

      <div class="frame">
        <KitShell title="agenticstudio" page-title="workstation" :theme="theme" :themes="['light', 'dark']" :global="false">
          <template #bar>
            <KitToolbar label="Filters">
              <KitSelect v-model="chosen" label="Architecture" :options="options" placeholder="Choose an architecture" />
              <KitButton tone="primary">Open draft</KitButton>
            </KitToolbar>
          </template>
          <KitStack>
            <KitPanel name="parts" title="Parts" collapsible>
              <KitTable label="Parts" :columns="columns" :rows="rows" row-key="id">
                <template #actions="{ row }">
                  <KitLink :href="`#${String(row.id)}`">Open</KitLink>
                </template>
              </KitTable>
            </KitPanel>
          </KitStack>
        </KitShell>
      </div>

      <div class="grid">
        <KitPanel name="buttons" title="Buttons">
          <div class="row">
            <KitButton tone="primary">Apply</KitButton>
            <KitButton>Cancel</KitButton>
            <KitButton tone="ghost">Tool</KitButton>
            <KitButton tone="danger">Remove</KitButton>
          </div>
          <div class="row">
            <KitButton tone="primary" size="sm">Small</KitButton>
            <KitButton tone="primary" busy>Busy</KitButton>
            <KitButton tone="primary" disabled>Disabled</KitButton>
            <KitCopy :value="record.digest" label="Copy digest" />
          </div>
        </KitPanel>

        <KitPanel name="select" title="Select">
          <div class="select-demo" :data-demo="`select-${theme}`">
            <KitSelect v-model="chosen" label="Architecture" :options="options" show-label />
            <KitSelect v-model="empty" label="Empty" :options="options" placeholder="Choose an architecture" show-label />
          </div>
        </KitPanel>

        <KitPanel name="badges" title="Badges and metrics">
          <div class="row">
            <KitBadge v-for="t in tones" :key="t" :tone="t">{{ t }}</KitBadge>
          </div>
          <div class="metrics">
            <KitMetric label="Parts" value="34" hint="in the draft" />
            <KitMetric label="Check" value="passes" tone="success" />
            <KitMetric label="Refusals" value="0" tone="danger" />
          </div>
        </KitPanel>

        <KitPanel name="form" title="Form">
          <KitForm submit-label="Apply">
            <KitField label="Name" :for="`gallery-name-${theme}`">
              <KitTextInput :id="`gallery-name-${theme}`" v-model="name" />
            </KitField>
            <KitField label="Patch" :for="`gallery-patch-${theme}`" hint="One rule per line.">
              <KitCodeBox :id="`gallery-patch-${theme}`" v-model="patch" :rows="4" :highlight="highlight" />
            </KitField>
            <KitSwitch v-model="dry" label="Dry run" />
            <template #result>
              <KitRecord :fields="refusalFields" :row="refusal" tone="danger" />
            </template>
          </KitForm>
        </KitPanel>

        <KitPanel name="record" title="Record">
          <KitRecord :fields="recordFields" :row="record">
            <template #actions>
              <KitCopy :value="record.digest" label="Copy digest" />
            </template>
          </KitRecord>
        </KitPanel>

        <KitPanel name="list" title="List and search">
          <KitSearch v-model="search" label="Find in parts" />
          <KitList label="Parts" :columns="columns" :rows="rows.filter((r) => r.id.includes(search))" row-key="id" />
        </KitPanel>

        <KitPanel name="nav" title="Navigation and values">
          <KitNav :label="`Pages, ${theme}`" :sections="navSections" />
          <KitTable label="Values" :columns="valueColumns" :rows="valueRows" row-key="id" />
        </KitPanel>

        <KitPanel name="states" title="States">
          <KitLoading />
          <KitEmpty title="No draft open" message="Choose an architecture and open a draft." />
          <KitError message="The drafts could not be read." detail="NoSuchRoute: no route /api/nothing" />
        </KitPanel>

        <KitPanel name="collapsed" title="Collapsed panel" collapsible collapsed>
          <p>Hidden until opened.</p>
        </KitPanel>

        <KitPanel name="read-empty" title="Panel of an empty read" state="empty" empty-message="The check passes." />

        <KitPanel name="code" title="Code box and clamp">
          <KitCodeBox :model-value="patch" readonly label="Patch" :highlight="highlight" />
          <KitClamp :text="record.description" />
        </KitPanel>

        <KitPanel name="toasts" title="Toasts">
          <div class="toast-frame">
            <KitToaster :toasts="toasts" />
          </div>
        </KitPanel>
      </div>

      <div class="split-frame">
        <KitSplit>
          <template #main>
            <KitZoomCanvas label="Example canvas" :content-width="640" :content-height="260">
              <svg width="640" height="260" viewBox="0 0 640 260" role="img" aria-label="Three boxes">
                <rect x="20" y="90" width="160" height="80" rx="12" class="box" />
                <rect x="240" y="90" width="160" height="80" rx="12" class="box" />
                <rect x="460" y="90" width="160" height="80" rx="12" class="box" />
                <text x="100" y="135" text-anchor="middle" class="label">owner</text>
                <text x="320" y="135" text-anchor="middle" class="label">operator</text>
                <text x="540" y="135" text-anchor="middle" class="label">worker</text>
              </svg>
            </KitZoomCanvas>
          </template>
          <template #side>
            <KitStack>
              <KitPanel name="side" title="Side panel">
                <p class="muted">The narrow pane of a split scrolls on its own.</p>
              </KitPanel>
            </KitStack>
          </template>
        </KitSplit>
      </div>
    </KitPage>
  </section>
</template>

<style scoped>
.sheet {
  background: var(--kit-canvas);
  color: var(--kit-text);
}
.frame {
  height: 300px;
  overflow: hidden;
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-card);
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 21rem), 1fr));
  gap: var(--kit-space-3);
  align-items: start;
}
.row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--kit-space-2);
  align-items: center;
}
.select-demo {
  display: flex;
  flex-direction: column;
  gap: var(--kit-space-2);
  min-height: 15rem;
}
.metrics {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--kit-space-2);
}
.toast-frame {
  position: relative;
  height: 8rem;
  transform: translateZ(0);
}
.split-frame {
  height: 320px;
  overflow: hidden;
  border: 1px solid var(--kit-hairline);
  border-radius: var(--kit-radius-card);
}
.box {
  fill: var(--kit-surface);
  stroke: var(--kit-line);
}
.label {
  fill: var(--kit-heading);
  font-family: var(--kit-font);
  font-size: 14px;
  font-weight: 600;
}
.muted {
  color: var(--kit-muted);
  font-size: var(--kit-text-sm);
}
</style>
