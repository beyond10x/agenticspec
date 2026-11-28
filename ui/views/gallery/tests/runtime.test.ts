// The runtime interface in the browser (ui/runtime.openapi.yaml): the gallery's fixture client
// answers a scope as the fixture source does, and the action set of a scope decides which actions
// a widget shows. A generated widget gates each command action with `useActions(client)` of its
// scope and `may.allows(<command>)` (held by the golden case `actions` of crates/agenticspec-ui);
// the probe below does the same with the generated runtime.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createApp, defineComponent, h, nextTick } from 'vue';
import {
  at,
  compareValues,
  contains,
  groupRows,
  install,
  member,
  metricValue,
  same,
  sortRows,
  truthy,
  useActions,
  useWidget,
} from '../src/generated/runtime';
import { createClient, type Client } from '../src/generated/client';
import { R3_VIEWS, fixtureClient, scripts, source, views } from '../src/fixtures';
import doc from '../../../runtime.openapi.yaml?raw';

const A = source.architecture_id;
const ANSWER = 'agentic.question.Answer';

const mounted: { unmount(): void }[] = [];

afterEach(() => {
  while (mounted.length) mounted.pop()?.unmount();
  vi.unstubAllGlobals();
});

async function settle(): Promise<void> {
  for (let i = 0; i < 5; i++) {
    await Promise.resolve();
    await nextTick();
  }
}

/** A part page's widget as generated: the client of its scope, its action set, a row action. */
const Probe = defineComponent({
  props: { part: { type: String, required: true }, instance: { type: Number, required: false } },
  setup(props) {
    const client = useWidget({ architecture: A, part: props.part, instance: props.instance });
    const may = useActions(client);
    return () => h('div', [may.allows(ANSWER) ? h('button', { 'data-action': 'answer' }, 'Answer') : null]);
  },
});

function mount(props: { part: string; instance?: number }, client: Client): HTMLElement {
  const app = createApp({ render: () => h(Probe, props) });
  install(app, { app: 't', title: 't', home: 'p', routes: [], client, hosts: {}, hooks: {}, shell: {}, theme: null });
  const el = document.createElement('div');
  document.body.append(el);
  app.mount(el);
  mounted.push(app);
  return el;
}

const fixtures = () => fixtureClient(views, scripts, { play: false, source });

describe('scope', () => {
  it('R1 at team.lead answers exactly the rows a placement to the part reaches', async () => {
    const lead = fixtures().scope(`/scopes/${A}/team.lead`);
    const rows = await lead.read('agentic.harness.Turns', {});
    expect(rows.map((r) => r.turn_id)).toEqual(['00000000-0000-4000-8000-391e34cb0000']);
    const worker = fixtures().scope(`/scopes/${A}/team.worker/2`);
    expect((await worker.read('agentic.harness.Turns', {})).map((r) => r.turn_id)).toEqual(['00000000-0000-4000-8000-391e34cb0001']);
    expect((await fixtures().read('agentic.harness.Turns', {})).length).toBe(3);
  });

  it('R3 answers the placement views of the architecture, and its list is the document\'s', async () => {
    const lead = fixtures().scope(`/scopes/${A}/team.lead`);
    for (const view of R3_VIEWS) expect(Array.isArray(await lead.read(view as never, {} as never)), view).toBe(true);
    expect((await lead.read('agentic.architecture.Parts', {})).map((r) => r.name)).toEqual(['team.lead']);
    const block = doc.split('r3_views:\n')[1].split(/\n(?! {4}- )/)[0];
    expect(block.split('\n').map((l) => l.replace(/^ {4}- /, '').trim())).toEqual(R3_VIEWS);
  });
});

describe('action set', () => {
  it('shows Answer on team.lead, where a grant allows it, and hides it on team.worker', async () => {
    const client = fixtures();
    const lead = mount({ part: 'team.lead' }, client);
    const worker = mount({ part: 'team.worker', instance: 2 }, client);
    await settle();
    expect(lead.querySelector('[data-action="answer"]')).not.toBeNull();
    expect(worker.querySelector('[data-action="answer"]')).toBeNull();
    const set = await client.scope(`/scopes/${A}/team.lead`).actions();
    expect(set?.find((e) => e.command === ANSWER)).toEqual({ command: ANSWER, allowed: true, grant_id: '00000000-0000-4000-8000-69f9b46e0100' });
  });

  it('shows every action when the source has no action set (404), and none while it loads', async () => {
    let answer: (r: Response) => void = () => undefined;
    const fetcher = vi.fn(
      (url: string) =>
        new Promise<Response>((resolve) => {
          if (url.endsWith('/api/actions')) answer = resolve;
          else resolve(new Response(JSON.stringify({ rows: [] }), { status: 200 }));
        }),
    );
    vi.stubGlobal('fetch', fetcher);
    const el = mount({ part: 'team.lead' }, createClient('https://runtime.example'));
    await settle();
    expect(fetcher).toHaveBeenCalledWith(`https://runtime.example/scopes/${A}/team.lead/api/actions`, { credentials: 'include' });
    expect(el.querySelector('[data-action="answer"]')).toBeNull();
    answer(new Response(JSON.stringify({ error: 'NoSuchPath' }), { status: 404 }));
    await settle();
    expect(el.querySelector('[data-action="answer"]')).not.toBeNull();
  });

  it('two widgets of one scope read its action set once', async () => {
    const client = fixtures();
    const scoped = client.scope(`/scopes/${A}/team.lead`);
    const spy = vi.spyOn(scoped, 'actions');
    vi.spyOn(client, 'scope').mockReturnValue(scoped);
    mount({ part: 'team.lead' }, client);
    mount({ part: 'team.lead' }, client);
    await settle();
    expect(spy).toHaveBeenCalledTimes(1);
  });
});

// What a section does with the rows it read, as released ess-ui/1 defines it (ESS 0.99.2): the
// generator compiles `reads.filter` into calls of `same`, `member`, `contains`, `truthy` and `at`
// (held by `constructs` and the golden case `rows` of crates/agenticspec-ui), and draws `sort`,
// `group_by` and a metric with `sortRows`, `groupRows` and `metricValue`.
describe('row predicate', () => {
  it('counts null, false, zero, NaN, empty text and an empty list as false, the text false as true', () => {
    for (const v of [null, undefined, false, 0, NaN, '', []]) expect(truthy(v)).toBe(false);
    for (const v of [true, 1, -1, 'false', 'x', [0], {}]) expect(truthy(v)).toBe(true);
  });

  it('compares by text, lists by their members in any order and maps by their fields', () => {
    expect(same('3', 3)).toBe(true);
    expect(same(3.0, '3')).toBe(true);
    expect(same(null, '')).toBe(true);
    expect(same(undefined, null)).toBe(true);
    expect(same(true, 'true')).toBe(true);
    expect(same('open', 'Open')).toBe(false);
    expect(same([1, 2], [2, 1])).toBe(true);
    expect(same([1, 2], [1, 2, 2])).toBe(false);
    expect(same({ a: 1, b: [2] }, { b: [2], a: 1 })).toBe(true);
    expect(same({ a: 1 }, '[object Object]')).toBe(false);
  });

  it('finds a value in a literal list, words in a text in any case, and walks a path through lists', () => {
    expect(member('normal', ['low', 'normal'])).toBe(true);
    expect(member(3, ['3'])).toBe(true);
    expect(member('high', [])).toBe(false);
    expect(contains('Close the Ticket', 'ticket close')).toBe(true);
    expect(contains('Close', 'open')).toBe(false);
    expect(contains('anything', '')).toBe(true);
    expect(at({ owner: { name: 'ada' } }, ['owner', 'name'])).toBe('ada');
    expect(at({ tags: [{ n: 'a' }, { n: 'b' }] }, ['tags', 'n'])).toEqual(['a', 'b']);
    expect(at({ tags: [1, 2, 3] }, ['tags', 'length'])).toBe(3);
    expect(at({ owner: 'ada' }, ['owner', 'name'])).toBeUndefined();
  });

  it('keeps the rows the compiled predicate of the golden case keeps', () => {
    const rows = [
      { id: 1, state: 'open', owner: 'ada' },
      { id: 2, state: 'open', owner: 'bob' },
      { id: 3, state: 'done', owner: 'ada' },
    ];
    const keep = (owner: string) =>
      rows
        .filter((row) => same(row['state'], 'open') && (same(row['owner'], owner) || same(owner, null)))
        .map((r) => r.id);
    expect(keep('ada')).toEqual([1]);
    expect(keep('')).toEqual([1, 2]);
  });
});

describe('sort and groups', () => {
  const rows = [
    { id: 'a', n: 10, p: 'low' },
    { id: 'b', n: 'x', p: 'high' },
    { id: 'c', n: 2, p: 'normal' },
    { id: 'd', n: '7', p: 'high' },
    { id: 'e', p: 'urgent' },
  ];

  it('orders numbers by value before every other value, and text by its text', () => {
    expect(sortRows(rows, 'n', 'asc', false).map((r) => r.id)).toEqual(['c', 'a', 'd', 'b', 'e']);
    expect(sortRows(rows, 'n', 'desc', false).map((r) => r.id)).toEqual(['e', 'b', 'd', 'a', 'c']);
  });

  it('reads text that is a number as one under a number column', () => {
    expect(sortRows(rows, 'n', 'asc', true).map((r) => r.id)).toEqual(['c', 'd', 'a', 'b', 'e']);
    expect(compareValues('10', '9', true)).toBe(1);
    expect(compareValues('10', '9', false)).toBe(-1);
  });

  it('keeps the order of rows that compare equal and leaves the input as it was', () => {
    const before = rows.map((r) => r.id);
    expect(sortRows(rows, 'p', 'asc', false).map((r) => r.id)).toEqual(['b', 'd', 'a', 'c', 'e']);
    expect(rows.map((r) => r.id)).toEqual(before);
  });

  it('groups under group_order first, then each other value as it first appears', () => {
    const groups = groupRows(rows, 'p', ['high', 'normal', 'low']);
    expect(groups.map((g) => [g.name, g.rows.map((r) => r.id)])).toEqual([
      ['high', ['b', 'd']],
      ['normal', ['c']],
      ['low', ['a']],
      ['urgent', ['e']],
    ]);
  });

  it('shows an ordered group no row falls under only when asked', () => {
    expect(groupRows(rows, 'p', ['none']).map((g) => g.name)).not.toContain('none');
    expect(groupRows(rows, 'p', ['none'], true).map((g) => g.name)[0]).toBe('none');
  });
});

describe('metric', () => {
  const ready = (rows: unknown[]) => ({ state: 'ready' as const, rows });

  it('counts, sums, takes the least, the most and the mean of a field', () => {
    const rows = [{ m: 2 }, { m: 4 }, { m: 'x' }];
    expect(metricValue(ready(rows), 'count', null, null)).toBe(3);
    expect(metricValue(ready(rows), 'sum', 'm', null)).toBe(6);
    expect(metricValue(ready(rows), 'min', 'm', null)).toBe(2);
    expect(metricValue(ready(rows), 'max', 'm', null)).toBe(4);
    expect(metricValue(ready(rows), 'avg', 'm', null)).toBe(3);
    expect(metricValue({ state: 'empty', rows: [] }, 'sum', 'm', null)).toBe(0);
    expect(metricValue({ state: 'empty', rows: [] }, 'avg', 'm', null)).toBe('–');
  });

  it('takes the first row: its match field, else its value, else its first number', () => {
    expect(metricValue(ready([{ k: 'a', v: 5, value: 9 }]), null, null, 'v')).toBe(5);
    expect(metricValue(ready([{ k: 'a', v: 5, value: 9 }]), null, null, null)).toBe(9);
    expect(metricValue(ready([{ k: 'a', v: 5 }]), null, null, null)).toBe(5);
  });

  it('says it loads, and that it failed', () => {
    expect(metricValue({ state: 'loading', rows: [] }, 'count', null, null)).toBe('…');
    expect(metricValue({ state: 'failed', rows: [] }, 'count', null, null)).toBe('Error');
  });
});
