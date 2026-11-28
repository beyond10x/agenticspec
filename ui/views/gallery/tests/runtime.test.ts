// The runtime interface in the browser (ui/runtime.openapi.yaml): the gallery's fixture client
// answers a scope as the fixture source does, and the action set of a scope decides which actions
// a widget shows. A generated widget gates each command action with `useActions(client)` of its
// scope and `may.allows(<command>)` (held by the golden case `actions` of crates/agenticspec-ui);
// the probe below does the same with the generated runtime.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createApp, defineComponent, h, nextTick } from 'vue';
import { install, useActions, useWidget } from '../src/generated/runtime';
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
