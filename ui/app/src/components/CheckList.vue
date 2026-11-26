<script setup lang="ts">
import { computed } from 'vue';
import type { SessionChecksRow } from '../generated/types.ts';
import { verdictStatus } from '../legend.ts';
import { int } from '../format.ts';

const props = defineProps<{ checks: SessionChecksRow[] }>();

const order = { Gap: 0, Unk: 1, Ok: 2 } as const;
const sorted = computed(() => props.checks.slice().sort((a, b) => order[a.verdict] - order[b.verdict] || a.check.localeCompare(b.check)));

function title(check: string): string {
  const s = check.replace(/-/g, ' ');
  return s.charAt(0).toUpperCase() + s.slice(1);
}
</script>

<template>
  <ul class="checks">
    <li v-for="c in sorted" :key="c.check_id">
      <span class="badge" :style="{ background: verdictStatus(c.verdict, c.severity).color }" aria-hidden="true">
        {{ verdictStatus(c.verdict, c.severity).mark }}
      </span>
      <div class="body">
        <div class="head">
          <span class="name">{{ title(c.check) }}</span>
          <span class="status secondary">
            {{ verdictStatus(c.verdict, c.severity).text }}<template v-if="c.verdict !== 'Ok'">
              · {{ int(c.rows) }}<template v-if="c.possible"> ({{ int(c.possible) }} possible)</template></template>
          </span>
        </div>
        <div class="about muted">{{ c.about }}</div>
      </div>
    </li>
    <li v-if="!checks.length" class="muted none">No check verdicts yet; they follow the session within 30 s.</li>
  </ul>
</template>

<style scoped>
.checks {
  list-style: none;
  margin: 0;
  padding: 0;
}
li {
  display: flex;
  gap: 12px;
  padding: 11px 0;
  border-bottom: 1px solid var(--grid);
}
li:last-child {
  border-bottom: 0;
}
.badge {
  flex: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  color: #fff;
  margin-top: 1px;
}
.body {
  min-width: 0;
  flex: 1;
}
.head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.name {
  font-weight: 500;
}
.about {
  font-size: 12.5px;
  margin-top: 2px;
}
.none {
  display: block;
  font-size: 13px;
}
</style>
