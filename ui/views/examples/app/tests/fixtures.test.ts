// Unit tests of the examples' fixture client: every read of each generated app is answered, the
// level's own fixtures answer its parts, reach, graph and settings, and a missing view is refused.
import { describe, expect, it } from 'vitest';
import { READS as LARGE } from '../src/large-37/generated/client';
import { READS as WORKSTATION } from '../src/workstation/generated/client';
import { exampleClient, exampleRows, offset } from '../src/fixtures';

class Refusal extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    message: string,
  ) {
    super(message);
  }
}
const refuse = (s: number, c: string, m: string) => new Refusal(s, c, m);

describe('the fixtures of the derived examples', () => {
  for (const [name, reads] of [
    ['large-37', LARGE],
    ['workstation', WORKSTATION],
  ] as const) {
    it(`answer every read of ${name}`, () => {
      const rows = exampleRows(name);
      expect(reads.filter((v) => !rows.has(v))).toEqual([]);
    });
  }

  it('answer the level reads from the fixtures derived beside the example', async () => {
    const client = exampleClient('large-37', { play: false, refuse });
    expect(await client.read('agentic.architecture.Parts', {})).toHaveLength(37);
    const [graph] = await client.read('agentic.architecture.Graph', { architecture: 'large-37' });
    expect(typeof graph.svg).toBe('string');
    const ws = exampleClient('workstation', { play: false, refuse });
    expect((await ws.read('agentic.architecture.Parts', {})).map((r) => r.name)).toContain('worker');
  });

  it('refuse a view no fixture answers and every command', async () => {
    const client = exampleClient('workstation', { play: false, refuse });
    await expect(client.read('agentic.nothing.Here', {})).rejects.toMatchObject({ status: 404 });
    await expect(client.command('agentic.question.Answer', {})).rejects.toMatchObject({ status: 501 });
    expect(offset('250ms')).toBe(250);
    expect(offset('2s')).toBe(2000);
  });
});
