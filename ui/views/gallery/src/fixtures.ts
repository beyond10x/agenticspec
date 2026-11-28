// The client of the gallery: the panels' fixtures answer every read (a file per view) and play
// every channel (a script per block channel: each event at its `at`), in the browser, with no
// server. Below a scope (`/scopes/<architecture>[/<part>[/<instance>]]`) it answers as the fixture
// source of ui/views/source/source.json does (the runtime interface, ui/runtime.openapi.yaml): only
// the rows and events in scope of the placements, the placement views of the architecture, and the
// action set of the source's first principal by its grants. At the root it answers every row and
// has no action set.
import { Refused, type ActionEntry, type Client, type Row, type Value } from './generated/client';

export interface ViewFixture {
  view: string;
  rows: Row[];
}

export interface ChannelScript {
  channel: string;
  events: { at: string; event: string; payload: Value }[];
}

export interface Grant {
  grant_id: string;
  grantee_id: string;
  action_class: string;
  scope: string;
  effect: string;
  state: string;
}

/** The fixture source file, ui/views/source/source.json. */
export interface SourceFile {
  architecture_id: string;
  principals: { token: string; principal_id: string }[];
  views: Record<string, Row[]>;
  placements: Row[];
  action_classes: Record<string, string>;
  grants: Grant[];
}

const viewFiles = import.meta.glob<ViewFixture>('../../fixtures/*.json', { eager: true, import: 'default' });
const scriptFiles = import.meta.glob<ChannelScript>('../../fixtures/scripts/*.json', { eager: true, import: 'default' });
const sourceFiles = import.meta.glob<SourceFile>('../../source/source.json', { eager: true, import: 'default' });

/** Every view fixture of ui/views/fixtures. */
export const views: ViewFixture[] = Object.values(viewFiles);
/** Every channel script of ui/views/fixtures/scripts. */
export const scripts: ChannelScript[] = Object.values(scriptFiles);
/** The fixture source file. */
export const source: SourceFile = Object.values(sourceFiles)[0];

/** The views R3 reads, answered for the architecture of the scope (ui/runtime.openapi.yaml). */
export const R3_VIEWS = [
  'agentic.architecture.Placements',
  'agentic.architecture.Parts',
  'agentic.architecture.Reach',
  'agentic.architecture.Revisions',
  'agentic.architecture.Contracts',
];

/** `1s`, `250ms` in milliseconds. */
export function offset(at: string): number {
  const m = /^(\d+)(ms|s)$/.exec(at.trim());
  if (!m) return 0;
  return Number(m[1]) * (m[2] === 's' ? 1000 : 1);
}

export interface FixtureOptions {
  /** Play the channel scripts; false keeps every panel still (screenshots). */
  play?: boolean;
  /** The fixture source file the scopes are answered from; without it a scope answers as the whole. */
  source?: SourceFile;
}

interface Scope {
  architecture: string;
  part?: string;
  instance?: number;
  path: string;
}

function parseScope(path: string): Scope | null {
  const m = /^\/scopes\/([^/]+)(?:\/([^/]+)(?:\/(\d+))?)?$/.exec(path);
  if (!m) return null;
  const [architecture, part, instance] = [m[1], m[2], m[3]].map((s) => (s === undefined ? undefined : decodeURIComponent(s)));
  return {
    architecture: architecture as string,
    part,
    instance: instance === undefined ? undefined : Number(instance),
    path: [architecture, part, instance].filter((s) => s !== undefined).join('/'),
  };
}

/** A grant on `grantScope` covers `scope`: the same scope or one above it. */
export function covers(grantScope: string, scope: string): boolean {
  return scope === grantScope || scope.startsWith(grantScope + '/');
}

/** The evaluator of R5 and R4: a `Deny` grant withholds; else the first that allows. */
export function evaluate(file: SourceFile, principal: string, command: string, scope: string): string | null {
  const cls = file.action_classes[command];
  if (!cls) return null;
  const mine = file.grants.filter((g) => g.grantee_id === principal && g.state === 'Active' && g.action_class === cls && covers(g.scope, scope));
  if (mine.some((g) => g.effect === 'Deny')) return null;
  return mine.find((g) => g.effect === 'Allow' || g.effect === 'ApprovalRequired')?.grant_id ?? null;
}

function inScope(row: Value, placed: Set<string>): boolean {
  if (row === null || typeof row !== 'object' || Array.isArray(row)) return false;
  return Object.values(row).some((v) => typeof v === 'string' && placed.has(v));
}

export function fixtureClient(fixtures: ViewFixture[], channels: ChannelScript[], options: FixtureOptions = {}): Client {
  const rows = new Map(fixtures.map((f) => [f.view, f.rows]));
  // A scope reads the views the source file gives (its parts) instead of the panels' fixtures.
  const scopedRows = new Map(rows);
  for (const [view, r] of Object.entries(options.source?.views ?? {})) scopedRows.set(view, r);
  const events = new Map(channels.map((s) => [s.channel, s.events]));

  function make(scope: Scope | null): Client {
    const file = options.source;
    const placements = (file?.placements ?? []).filter(
      (p) =>
        scope !== null &&
        p.architecture_id === scope.architecture &&
        (scope.part === undefined || p.part_path === scope.part) &&
        (scope.instance === undefined || p.instance === scope.instance),
    );
    const placed = new Set(placements.map((p) => String(p.entity_id)));
    const scoped = scope !== null && file !== undefined;
    const client: Client = {
      async read(view) {
        if (scoped && (view as string) === 'agentic.architecture.Placements') return placements.map((r) => ({ ...r }));
        const found = (scoped ? scopedRows : rows).get(view);
        if (!found) throw new Refused(404, 'NoFixture', `no fixture answers ${view}`);
        let out = found;
        if (scoped && R3_VIEWS.includes(view)) {
          out = found.filter((r) => r.architecture_id === scope.architecture && (view !== 'agentic.architecture.Parts' || scope.part === undefined || r.name === scope.part));
        } else if (scoped) {
          out = found.filter((r) => inScope(r, placed));
        }
        return out.map((r) => ({ ...r }));
      },
      async command(command) {
        throw new Refused(501, 'NoBackend', `the gallery answers reads only, not ${command}`);
      },
      subscribe(channel, _params, onEvent) {
        if (options.play === false) return () => undefined;
        const timers = (events.get(channel) ?? [])
          .filter((e) => !scoped || inScope(e.payload, placed))
          .map((e) => setTimeout(() => onEvent(e.event, e.payload), offset(e.at)));
        return () => timers.forEach(clearTimeout);
      },
      async actions(): Promise<ActionEntry[] | null> {
        if (!scoped || !file.principals.length) return null;
        const principal = file.principals[0].principal_id;
        return Object.keys(file.action_classes)
          .sort()
          .map((command) => {
            const grant = evaluate(file, principal, command, scope.path);
            return grant ? { command, allowed: true, grant_id: grant } : { command, allowed: false };
          });
      },
      scope(path) {
        const s = parseScope(path);
        return s && file ? make(s) : client;
      },
    };
    return client;
  }

  return make(null);
}
