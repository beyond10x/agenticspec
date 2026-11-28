// The client of the derived examples: the block panels' fixtures (ui/views/fixtures) answer the
// panels' reads, and the fixtures `agenticspec ui derive` wrote beside each example
// (ui/views/examples/<name>.fixtures) answer the level's reads - its parts, its reach, its graph and
// its settings - and play its channel. In the browser, with no server.

export type Value = string | number | boolean | null | Value[] | { [key: string]: Value };
export type Row = Record<string, Value>;

interface ViewFixture {
  view?: string;
  rows: Row[];
}

interface ChannelScript {
  channel: string;
  events: { at: string; event: string; payload: Value }[];
}

/** The client every generated app takes, by its shape. */
export interface FixtureClient {
  read(view: string, params: object): Promise<Row[]>;
  command(command: string, input: object): Promise<never>;
  subscribe(channel: string, params: Record<string, Value>, onEvent: (event: string, data: Value) => void): () => void;
  scope(path: string): FixtureClient;
}

const panelFiles = import.meta.glob<ViewFixture>('../../../fixtures/*.json', { eager: true, import: 'default' });
const levelFiles = import.meta.glob<ViewFixture>('../../*.fixtures/*.json', { eager: true, import: 'default' });
const scriptFiles = import.meta.glob<ChannelScript>('../../*.fixtures/scripts/*.json', { eager: true, import: 'default' });

/** The view each fixture of a level answers, by its file name. */
export const LEVEL_VIEWS: Record<string, string> = {
  'parts.json': 'agentic.architecture.Parts',
  'reach.json': 'agentic.architecture.Reach',
  'graph.json': 'agentic.architecture.Graph',
  'settings.json': 'agentic.architecture.Settings',
};

/** The rows of every view the example `name` reads. */
export function exampleRows(name: string): Map<string, Row[]> {
  const rows = new Map<string, Row[]>();
  for (const f of Object.values(panelFiles)) if (f.view) rows.set(f.view, f.rows);
  for (const [path, f] of Object.entries(levelFiles)) {
    const [dir, file] = path.split('/').slice(-2);
    const view = LEVEL_VIEWS[file];
    if (dir === `${name}.fixtures` && view) rows.set(view, f.rows);
  }
  return rows;
}

function exampleScripts(name: string): Map<string, ChannelScript['events']> {
  const out = new Map<string, ChannelScript['events']>();
  for (const [path, s] of Object.entries(scriptFiles)) {
    if (path.split('/').slice(-3)[0] === `${name}.fixtures`) out.set(s.channel, s.events);
  }
  return out;
}

/** `1s`, `250ms` in milliseconds. */
export function offset(at: string): number {
  const m = /^(\d+)(ms|s)$/.exec(at.trim());
  return m ? Number(m[1]) * (m[2] === 's' ? 1000 : 1) : 0;
}

export interface FixtureOptions {
  /** Play the channel's script; false keeps every panel still. */
  play?: boolean;
  /** The refusal of the generated client, so the runtime knows it. */
  refuse: (status: number, code: string, message: string) => Error;
}

export function exampleClient(name: string, options: FixtureOptions): FixtureClient {
  const rows = exampleRows(name);
  const events = exampleScripts(name);
  const client: FixtureClient = {
    async read(view) {
      const found = rows.get(view);
      if (!found) throw options.refuse(404, 'NoFixture', `no fixture answers ${view}`);
      return found.map((r) => ({ ...r }));
    },
    async command(command) {
      throw options.refuse(501, 'NoBackend', `the examples answer reads only, not ${command}`);
    },
    subscribe(channel, _params, onEvent) {
      if (options.play === false) return () => undefined;
      const timers = (events.get(channel) ?? []).map((e) => setTimeout(() => onEvent(e.event, e.payload), offset(e.at)));
      return () => timers.forEach(clearTimeout);
    },
    scope() {
      return client;
    },
  };
  return client;
}
