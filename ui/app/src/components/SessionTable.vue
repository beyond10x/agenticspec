<script setup lang="ts">
import type { SessionListRow } from '../generated/types.ts';
import { ago, dayClock, duration, int, money, percent, projectName } from '../format.ts';
import { go } from '../router.ts';
import { ms, wallSeconds } from '../stats.ts';

defineProps<{ rows: SessionListRow[]; now: number; loading?: boolean }>();

function open(r: SessionListRow) {
  go({ page: 'session', sessionId: r.session_id });
}
</script>

<template>
  <div class="wrap">
    <table class="data">
      <thead>
        <tr>
          <th>Session</th>
          <th>Last active</th>
          <th class="r hide-sm">Wall time</th>
          <th class="r hide-sm">Agents</th>
          <th class="r hide-sm">Tool calls</th>
          <th class="r hide-sm">Cache hit share</th>
          <th class="r hide-sm">Waiting</th>
          <th class="r">Cost</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="r in rows"
          :key="r.session_id"
          class="row"
          tabindex="0"
          @click="open(r)"
          @keydown.enter="open(r)"
          @keydown.space.prevent="open(r)"
        >
          <td class="session">
            <div class="title">{{ r.title || 'Untitled session' }}</div>
            <div class="project muted" :title="r.project">{{ projectName(r.project) }}</div>
          </td>
          <td class="when">
            <div class="num">{{ ago(ms(r.last_at), now) }}</div>
            <div class="muted small num">{{ dayClock(ms(r.last_at)) }}</div>
          </td>
          <td class="r hide-sm">{{ duration(wallSeconds(r)) }}</td>
          <td class="r hide-sm">{{ int(r.agents) }}</td>
          <td class="r hide-sm">
            {{ int(r.tool_calls) }}
            <span v-if="r.refused_tool_calls" class="refused" :title="`${r.refused_tool_calls} refused`">
              · {{ r.refused_tool_calls }} refused
            </span>
          </td>
          <td class="r hide-sm">{{ percent(r.cache_hit_share) }}</td>
          <td class="r hide-sm">{{ percent(r.waiting_share) }}</td>
          <td class="r cost">{{ money(r.cost_usd) }}</td>
        </tr>
      </tbody>
    </table>
    <div v-if="!rows.length" class="empty">
      {{ loading ? 'Reading the store…' : 'No session in this slice matches.' }}
    </div>
  </div>
</template>

<style scoped>
.wrap {
  margin-top: 6px;
}
.session {
  max-width: 420px;
}
.title {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.project {
  font-size: 12.5px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.when {
  white-space: nowrap;
}
.small {
  font-size: 12px;
}
.refused {
  color: var(--bad-text);
  font-size: 12px;
}
.cost {
  white-space: nowrap;
  font-weight: 500;
}
@media (max-width: 720px) {
  .session {
    max-width: 190px;
  }
}
</style>
