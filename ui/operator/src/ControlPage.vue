<script setup lang="ts">
// A page across parts of the control-plane UI, drawn from the layout's `control` key: Inbox,
// Activity, Operations and Architecture are frames of tables (ControlTable), Metrics is the table
// of the computed KPIs with the simulator's prediction beside each, and Events is the level's
// channel as a log, newest first.
import { computed } from 'vue';
import { KitEmpty, KitPanel, KitStack, KitTable, type FieldDef } from '@kit';
import type { Client, LayoutControl, LayoutControlPage } from '../../views/src/index';
import ControlTable from './ControlTable.vue';
import { predicted, type LoggedEvent } from './control';

const props = defineProps<{
  client: Client;
  architecture: string;
  control: LayoutControl;
  page: LayoutControlPage;
  refresh: number;
  events: LoggedEvent[];
}>();

const metricColumns: FieldDef[] = [
  { field: 'label', label: 'KPI', as: 'text' },
  { field: 'kpi', label: 'Id', as: 'text' },
  { field: 'predicted', label: 'Predicted by the simulator', as: 'text' },
];
const metricRows = computed(() =>
  props.control.metrics.map((m) => ({ ...m, predicted: predicted(m.prediction) })),
);

const eventColumns: FieldDef[] = [
  { field: 'id', label: 'Cursor', as: 'text' },
  { field: 'event', label: 'Event', as: 'badge' },
  { field: 'data', label: 'Payload', as: 'text' },
];
</script>

<template>
  <section class="control-page" :data-page="page.page">
    <h2 class="page-title">{{ page.label }}</h2>
    <KitStack v-if="page.kind === 'metrics'">
      <KitPanel name="metrics" title="Computed KPIs">
        <KitTable label="Computed KPIs" :columns="metricColumns" :rows="metricRows" row-key="kpi" empty-message="The metrics ingress computes no KPI." />
      </KitPanel>
    </KitStack>
    <KitStack v-else-if="page.kind === 'events'">
      <KitPanel name="events" :title="`Channel ${control.channel}`">
        <KitTable label="Events" :columns="eventColumns" :rows="events" row-key="key" empty-message="No event has arrived yet." />
      </KitPanel>
    </KitStack>
    <KitEmpty v-else-if="!page.frames.length" :message="`Nothing of ${architecture} belongs on this page.`" />
    <KitStack v-else>
      <template v-for="(frame, i) in page.frames" :key="i">
        <KitPanel v-if="frame.title" :name="`frame_${i}`" :title="frame.title" collapsible :collapsed="frame.collapsed">
          <KitStack :padded="false">
            <KitPanel v-for="s in frame.sections" :key="s.name" :name="s.name" :title="s.title" collapsible>
              <ControlTable :client="client" :architecture="architecture" :section="s" :refresh="refresh" />
            </KitPanel>
          </KitStack>
        </KitPanel>
        <template v-else>
          <KitPanel v-for="s in frame.sections" :key="s.name" :name="s.name" :title="s.title" collapsible>
            <ControlTable :client="client" :architecture="architecture" :section="s" :refresh="refresh" />
          </KitPanel>
        </template>
      </template>
    </KitStack>
  </section>
</template>

<style scoped>
.page-title {
  margin: 0 0 var(--kit-space-2);
}
</style>
