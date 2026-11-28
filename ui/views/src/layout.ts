// The layout of a derived UI (`agenticspec-ui-layout/1`), which `agenticspec ui derive` writes
// beside its document as `<name>.layout.json`: per level, its part pages, each a list of frames of
// panels with their arguments. A part page and an overview are drawn from it; a server publishes
// it at `<source>/layouts/<architecture>.json`.

export interface LayoutAction {
  name: string;
  command: string;
  label: string;
  bind: Record<string, string>;
}

export interface LayoutSection {
  name: string;
  title: string;
  part: string;
  widget: string;
  args: Record<string, unknown>;
  row_actions: LayoutAction[];
}

export interface LayoutFrame {
  title: string | null;
  collapsed: boolean;
  sections: LayoutSection[];
}

export interface LayoutPage {
  page: string;
  part: string;
  label: string;
  block: string;
  actions: LayoutAction[];
  frames: LayoutFrame[];
}

export interface LayoutPart {
  name: string;
  path: string;
  label: string;
  block: string;
  count: number;
  /** The part page, or the overview of a nested level. */
  page: string;
}

export interface LayoutLevel {
  name: string;
  path: string;
  parts: LayoutPart[];
  overview: string;
  channel: string | null;
  instances: number;
  pages: LayoutPage[];
}

export interface Layout {
  format: 'agenticspec-ui-layout/1';
  architecture: string;
  home: string;
  levels: LayoutLevel[];
  /** The pages across parts of a control-plane UI (`agenticspec ui derive --control-plane`). */
  control?: LayoutControl;
}

/** A column of a table of the control plane. */
export interface LayoutColumn {
  field: string;
  as: 'text' | 'number' | 'badge' | 'toggle' | 'list' | 'timestamp' | 'date' | 'time' | 'percent' | 'json';
  label: string;
}

/** A section of a page across parts: a view at the architecture's scope, or at a part's. */
export interface LayoutControlSection {
  name: string;
  title: string;
  widget: string;
  /** The part the rows are scoped to; null for the whole architecture. */
  part: string | null;
  args: Record<string, unknown>;
  key: string;
  columns: LayoutColumn[];
  /** The field the rows are ordered by, newest first. */
  order: string | null;
  row_actions: LayoutAction[];
}

export interface LayoutControlFrame {
  title: string | null;
  collapsed: boolean;
  sections: LayoutControlSection[];
}

export interface LayoutControlPage {
  page: string;
  label: string;
  kind: 'inbox' | 'activity' | 'operations' | 'metrics' | 'architecture' | 'events';
  frames: LayoutControlFrame[];
}

/** A tile of the overview: the open rows of an Inbox view, or the rows per state of a view. */
export interface LayoutTile {
  label: string;
  page: string;
  count: 'open' | 'states';
  field: string | null;
  section: LayoutControlSection;
}

/** A KPI of the Metrics page and the simulator's prediction of it. */
export interface LayoutMetric {
  kpi: string;
  label: string;
  prediction: number | null;
}

export interface LayoutControl {
  /** The channel of the top level, which the Events page shows. */
  channel: string;
  events: string[];
  pages: LayoutControlPage[];
  tiles: LayoutTile[];
  metrics: LayoutMetric[];
}

/** Reads the layout of `architecture` from the server at `source`. */
export async function loadLayout(source: string, architecture: string): Promise<Layout> {
  const url = `${source.replace(/\/+$/, '')}/layouts/${encodeURIComponent(architecture)}.json`;
  const r = await fetch(url, { credentials: 'include' });
  if (!r.ok) throw new Error(`${url}: ${r.status}`);
  const layout = (await r.json()) as Layout;
  if (layout.format !== 'agenticspec-ui-layout/1') throw new Error(`${url}: not an agenticspec-ui-layout/1 document`);
  return layout;
}

/** The page of `part` (a part path, `team.coder` for a nested one) in `layout`. */
export function pageOf(layout: Layout, part: string): { level: LayoutLevel; page: LayoutPage } | null {
  for (const level of layout.levels) {
    const page = level.pages.find((p) => p.part === part);
    if (page) return { level, page };
  }
  return null;
}
