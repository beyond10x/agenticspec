<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from 'vue';
import { query, subscribe, upsert, type LiveState } from '../client.ts';
import type { OverviewFiguresRow, SessionListRow } from '../generated/types.ts';
import { bucketsOf, change, figures, slice, SLICES, ms, window, type Figure, type Slice } from '../stats.ts';
import { compact, duration, int, money, percent, plural } from '../format.ts';
import StatTile from '../components/StatTile.vue';
import SessionTable from '../components/SessionTable.vue';

const emit = defineEmits<{ live: [LiveState] }>();

// The list is read a page at a time from the server, newest first; a search reads the matches
// among the newest 1,000 sessions at once. The figures are the server's sums per bucket.
const PAGE = 100;
const SEARCH_LIMIT = 1000;

const sliceId = ref<Slice>((new URLSearchParams(location.search).get('slice') as Slice) || '72h');
const search = ref(new URLSearchParams(location.search).get('q') ?? '');
const searching = computed(() => search.value.trim() !== '');
const rows = shallowRef<SessionListRow[]>([]);
const buckets = shallowRef<OverviewFiguresRow[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const now = ref(Date.now());
const more = ref(false);

let controller: AbortController | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;

function listParams(offset = 0) {
  const q = search.value.trim();
  return {
    since: new Date(now.value - slice(sliceId.value).ms).toISOString(),
    search: q || undefined,
    offset: offset || undefined,
    limit: q ? SEARCH_LIMIT : offset + PAGE,
  };
}

async function load() {
  controller?.abort();
  controller = new AbortController();
  const signal = controller.signal;
  now.value = Date.now();
  const w = window(sliceId.value, now.value);
  try {
    const [list, figs] = await Promise.all([
      query('session-list', listParams(), { signal }),
      query('figures', { since: new Date(w.since).toISOString(), bucket_seconds: w.bucketSeconds }, { signal }),
    ]);
    rows.value = list;
    buckets.value = figs;
    more.value = !search.value.trim() && list.length >= PAGE;
    error.value = null;
  } catch (e) {
    if ((e as Error).name !== 'AbortError') error.value = (e as Error).message;
  } finally {
    loading.value = false;
  }
}

async function loadMore() {
  const offset = rows.value.length;
  try {
    const page = await query('session-list', listParams(offset));
    rows.value = [...rows.value, ...page.filter((r) => !rows.value.some((x) => x.session_id === r.session_id))];
    more.value = page.length >= PAGE;
  } catch (e) {
    error.value = (e as Error).message;
  }
}

function keepUrl() {
  const q = new URLSearchParams();
  if (sliceId.value !== '72h') q.set('slice', sliceId.value);
  if (search.value.trim()) q.set('q', search.value.trim());
  const s = q.toString();
  history.replaceState(null, '', s ? `/?${s}` : '/');
}

watch(sliceId, () => {
  keepUrl();
  void load();
});
watch(search, () => {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    keepUrl();
    void load();
  }, 180);
});

let close: (() => void) | null = null;
let tick: ReturnType<typeof setInterval> | null = null;
let refresh: ReturnType<typeof setTimeout> | null = null;
onMounted(() => {
  void load();
  close = subscribe(
    'sessions',
    {},
    {
      'agenticspec.view.SessionUpdated': (p) => {
        now.value = Date.now();
        const q = search.value.trim().toLowerCase();
        if (!q || p.names.includes(q)) rows.value = upsert(rows.value, p as SessionListRow, 'session_id');
        // The figures are sums the server keeps; read them again once the changes settle.
        if (refresh) clearTimeout(refresh);
        refresh = setTimeout(() => {
          const w = window(sliceId.value, now.value);
          void query('figures', { since: new Date(w.since).toISOString(), bucket_seconds: w.bucketSeconds }).then(
            (f) => (buckets.value = f),
            () => {},
          );
        }, 2000);
      },
    },
    (st) => emit('live', st),
  );
  tick = setInterval(() => (now.value = Date.now()), 30_000);
});
onUnmounted(() => {
  close?.();
  controller?.abort();
  if (tick) clearInterval(tick);
  if (refresh) clearTimeout(refresh);
});

const current = computed(() => {
  const from = now.value - slice(sliceId.value).ms;
  return rows.value.filter((r) => ms(r.last_at) >= from).sort((a, b) => ms(b.last_at) - ms(a.last_at));
});

// While a search narrows the list, the figures are summed from the matching rows.
const tiles = computed(() =>
  figures(searching.value ? bucketsOf(current.value, sliceId.value, now.value) : buckets.value, sliceId.value, now.value),
);

function show(f: Figure): string {
  if (f.value === null) return '–';
  switch (f.kind) {
    case 'count':
      return int(f.value);
    case 'money':
      return money(f.value);
    case 'duration':
      return duration(f.value);
    case 'share':
      return percent(f.value);
    default:
      return f.value >= 100 ? compact(f.value) : f.value.toFixed(f.value >= 10 ? 0 : 1);
  }
}

const periodName = computed(() => `previous ${slice(sliceId.value).label}`);
const totals = computed(() => {
  const t = tiles.value;
  const count = t.find((f) => f.id === 'sessions')?.value ?? 0;
  const cost = (t.find((f) => f.id === 'cost')?.value ?? 0) * count;
  return { count, cost };
});
</script>

<template>
  <section>
    <div class="intro">
      <div>
        <h1>Sessions</h1>
        <p class="secondary sub">
          <template v-if="!loading">
            <template v-if="searching">
              {{ plural(totals.count, 'session') }} matching in the last {{ slice(sliceId).label }} ·
              {{ money(totals.cost) }} at API list price
            </template>
            <template v-else>
              {{ plural(totals.count, 'session') }} in the last {{ slice(sliceId).label }} ·
              {{ money(totals.cost) }} at API list price
            </template>
          </template>
          <template v-else>Reading the store…</template>
        </p>
      </div>
      <div class="filters">
        <label class="search">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <circle cx="7" cy="7" r="5.25" stroke="currentColor" stroke-width="1.5" />
            <path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
          <input v-model="search" type="search" placeholder="Search sessions" aria-label="Search sessions" />
        </label>
        <div class="segmented" role="group" aria-label="Time slice">
          <button v-for="s in SLICES" :key="s.id" type="button" :aria-pressed="sliceId === s.id" @click="sliceId = s.id">
            {{ s.label }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="error" class="card error">{{ error }}</div>

    <div class="tiles" data-ux-region="tiles">
      <StatTile
        v-for="f in tiles"
        :key="f.id"
        :label="f.label"
        :value="show(f)"
        :note="f.note"
        :change="change(f.value, f.previous)"
        :higher-is-better="f.higherIsBetter"
        :period="periodName"
        :trend="f.trend"
      />
    </div>

    <div class="card section-gap" data-ux-region="sessions">
      <div class="card-head">
        <h2>Most recent</h2>
        <span class="hint">{{ more ? `${int(current.length)} of ${int(totals.count)}` : plural(current.length, 'session') }}</span>
      </div>
      <SessionTable :rows="current" :now="now" :loading="loading" />
      <div v-if="more" class="more">
        <button type="button" class="chip" @click="loadMore">Show {{ PAGE }} more</button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.intro {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 16px 24px;
  flex-wrap: wrap;
  margin-bottom: 20px;
}
.sub {
  margin: 4px 0 0;
  font-size: 13.5px;
}
.filters {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.tiles {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
  gap: 12px;
}
.more {
  display: flex;
  justify-content: center;
  padding: 12px;
  border-top: 1px solid var(--grid);
}
.error {
  padding: 14px 18px;
  margin-bottom: 16px;
  color: var(--bad-text);
}
@media (max-width: 720px) {
  .tiles {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }
  .filters,
  .search,
  .search input {
    width: 100%;
  }
  .segmented {
    width: 100%;
  }
  .segmented button {
    flex: 1;
  }
}
</style>
