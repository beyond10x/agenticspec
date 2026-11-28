// The client of the gallery: the panels' fixtures answer every read (a file per view) and play
// every channel (a script per block channel: each event at its `at`), in the browser, with no
// server. A scope answers as the whole, since a fixture holds the rows of one scope.
import { Refused, type Client, type Row, type Value } from './generated/client';

export interface ViewFixture {
  view: string;
  rows: Row[];
}

export interface ChannelScript {
  channel: string;
  events: { at: string; event: string; payload: Value }[];
}

const viewFiles = import.meta.glob<ViewFixture>('../../fixtures/*.json', { eager: true, import: 'default' });
const scriptFiles = import.meta.glob<ChannelScript>('../../fixtures/scripts/*.json', { eager: true, import: 'default' });

/** Every view fixture of ui/views/fixtures. */
export const views: ViewFixture[] = Object.values(viewFiles);
/** Every channel script of ui/views/fixtures/scripts. */
export const scripts: ChannelScript[] = Object.values(scriptFiles);

/** `1s`, `250ms` in milliseconds. */
export function offset(at: string): number {
  const m = /^(\d+)(ms|s)$/.exec(at.trim());
  if (!m) return 0;
  return Number(m[1]) * (m[2] === 's' ? 1000 : 1);
}

export interface FixtureOptions {
  /** Play the channel scripts; false keeps every panel still (screenshots). */
  play?: boolean;
}

export function fixtureClient(fixtures: ViewFixture[], channels: ChannelScript[], options: FixtureOptions = {}): Client {
  const rows = new Map(fixtures.map((f) => [f.view, f.rows]));
  const events = new Map(channels.map((s) => [s.channel, s.events]));
  const client: Client = {
    async read(view) {
      const found = rows.get(view);
      if (!found) throw new Refused(404, 'NoFixture', `no fixture answers ${view}`);
      return found.map((r) => ({ ...r }));
    },
    async command(command) {
      throw new Refused(501, 'NoBackend', `the gallery answers reads only, not ${command}`);
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
