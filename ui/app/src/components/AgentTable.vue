<script setup lang="ts">
import { computed } from 'vue';
import type { AgentLane } from '../session.ts';
import { compact, int, money, modelName, duration } from '../format.ts';

const props = defineProps<{ lanes: AgentLane[]; selected?: string | null }>();
const emit = defineEmits<{ select: [string | null] }>();

const maxCost = computed(() => Math.max(0.0001, ...props.lanes.map((l) => l.agent.cost_usd)));
</script>

<template>
  <table class="data">
    <thead>
      <tr>
        <th>Agent</th>
        <th class="r">Turns</th>
        <th class="r hide-sm">Tool calls</th>
        <th class="r hide-sm">Peak context</th>
        <th class="r hide-sm">Active</th>
        <th class="r">Cost</th>
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="l in lanes"
        :key="l.agent.agent_id"
        class="row"
        :class="{ on: selected === l.agent.agent_id }"
        tabindex="0"
        @click="emit('select', selected === l.agent.agent_id ? null : l.agent.agent_id)"
        @keydown.enter="emit('select', selected === l.agent.agent_id ? null : l.agent.agent_id)"
      >
        <td>
          <div class="name">{{ l.name }}</div>
          <div class="muted small">{{ modelName(l.agent.model) }}</div>
        </td>
        <td class="r">{{ int(l.agent.turns) }}</td>
        <td class="r hide-sm">{{ int(l.agent.tool_calls) }}</td>
        <td class="r hide-sm">{{ compact(l.agent.context_max) }}</td>
        <td class="r hide-sm">{{ duration((l.last - l.first) / 1000) }}</td>
        <td class="r cost">
          <span class="bar" aria-hidden="true"><span :style="{ width: `${(l.agent.cost_usd / maxCost) * 100}%` }" /></span>
          {{ money(l.agent.cost_usd) }}
        </td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.name {
  font-weight: 500;
}
.small {
  font-size: 12px;
}
tr.on td {
  background: var(--accent-wash);
}
.cost {
  white-space: nowrap;
}
.bar {
  display: inline-block;
  width: 44px;
  height: 6px;
  margin-right: 8px;
  border-radius: 3px;
  background: var(--surface-2);
  vertical-align: middle;
  overflow: hidden;
}
.bar span {
  display: block;
  height: 100%;
  background: var(--s1);
  border-radius: 3px;
}
</style>
