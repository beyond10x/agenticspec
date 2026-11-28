// Unit tests of the studio's host code: the patch tokens, the graph card's facts, the hook after
// OpenDraft, and the graph slot's selection.
import { describe, expect, it, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { nextTick, reactive } from 'vue';
import { tokens } from '../src/slots/patch';
import { asGraph, facts, linksOf, type GraphRow } from '../src/slots/graph';
import { hooks } from '../src/hooks';
import DraftGraph from '../src/slots/DraftGraph.vue';

const graph: GraphRow = {
  draft_id: 'd1',
  revision: 2,
  name: 'workstation',
  summary: '',
  svg: '<svg class="graph" width="200" height="100"><g class="edge" data-from="a" data-to="b"></g><g class="node f-agent" data-part="a"><rect class="card-bg" role="button" tabindex="0" aria-pressed="false"></rect></g><g class="node" data-part="b"><rect class="card-bg" role="button" tabindex="0" aria-pressed="false"></rect></g></svg>',
  width: 200,
  height: 100,
  parts: [
    { id: 'a', block: 'agent', family: 'agent', count: '12', acts_for: 'b', properties: [['harness', 'coding']] },
    { id: 'b', block: 'principal', family: 'principal', border: true },
  ],
  links: [{ from: 'a', to: 'b', carries: 'question' }],
};

describe('patch tokens', () => {
  it('marks verbs, variables, strings, numbers and comments and loses no character', () => {
    const text = 'set(P, count, 3) :- part{id: P, block: "agent"}.\n% fewer\nfrobnicate(P).';
    const t = tokens(text);
    expect(t.map((x) => x.text).join('')).toBe(text);
    const kinds = (k: string) => t.filter((x) => x.kind === k).map((x) => x.text);
    expect(kinds('verb')).toEqual(['set']);
    expect(kinds('keyword')).toEqual(['part', 'frobnicate']);
    expect(kinds('var')).toEqual(['P', 'P', 'P']);
    expect(kinds('string')).toEqual(['"agent"']);
    expect(kinds('number')).toEqual(['3']);
    expect(kinds('comment')).toEqual(['% fewer']);
  });

  it('keeps an unclosed string and odd characters', () => {
    const text = 'add(part{id: "x ~ @';
    expect(tokens(text).map((x) => x.text).join('')).toBe(text);
  });
});

describe('graph', () => {
  it('reads a row only when it is a graph', () => {
    expect(asGraph(graph)).toBe(graph);
    expect(asGraph({ svg: 1 })).toBeNull();
    expect(asGraph(null)).toBeNull();
  });

  it('a part card shows its facts and links', () => {
    expect(facts(graph.parts[0])).toEqual([
      ['block', 'agent'],
      ['count', '12'],
      ['acts for', 'b'],
      ['harness', 'coding'],
    ]);
    expect(facts(graph.parts[1])).toContainEqual(['border', 'a human reaches it']);
    expect(linksOf(graph, 'b')).toHaveLength(1);
  });

  it('clicking a part selects it, shows its card and lights its links', async () => {
    const read = reactive({ state: 'ready', rows: [graph], row: graph, error: '', reload: () => {} });
    const w = mount(DraftGraph, { props: { read: read as never }, attachTo: document.body });
    await nextTick();
    await nextTick();
    const a = w.get('[data-part="a"] rect');
    await a.trigger('click');
    expect(a.attributes('aria-pressed')).toBe('true');
    const card = w.get('[data-selected-part]');
    expect(card.attributes('data-part')).toBe('a');
    expect(card.text()).toContain('harness');
    expect(w.get('.edge').classes()).toContain('hot');
    await a.trigger('keydown', { key: 'Escape' });
    expect(w.find('[data-selected-part]').exists()).toBe(false);
    w.unmount();
  });
});

describe('hooks', () => {
  it('an opened draft is shown by its page', () => {
    const navigate = vi.fn();
    hooks['studio.draft.OpenDraft']?.({ draft_id: 'd1' }, { navigate, toast: vi.fn() });
    expect(navigate).toHaveBeenCalledWith('draft', { draft_id: 'd1' });
  });
});
