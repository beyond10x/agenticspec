// The row of `studio.DraftGraph` (agenticspec-render's graph of the draft's current revision) and
// what the selection card shows of a part.

export interface GraphPart {
  id: string;
  block: string;
  family: string;
  kind?: string | null;
  count?: string | null;
  per?: string | null;
  acts_for?: string | null;
  border?: boolean;
  subtitle?: string;
  properties?: [string, string][];
}

export interface GraphLink {
  from: string;
  to: string;
  carries: string;
  over?: string | null;
}

export interface GraphRow {
  draft_id: string;
  revision: number;
  name: string;
  summary: string;
  svg: string;
  width: number;
  height: number;
  parts: GraphPart[];
  links: GraphLink[];
}

/** The row when it is a graph, else null. */
export function asGraph(row: unknown): GraphRow | null {
  const r = row as Partial<GraphRow> | null | undefined;
  return r && typeof r.svg === 'string' && typeof r.width === 'number' && Array.isArray(r.parts) ? (r as GraphRow) : null;
}

/** The facts of a part, in the order the card shows them. */
export function facts(p: GraphPart): [string, string][] {
  const out: [string, string][] = [['block', p.block]];
  if (p.kind) out.push(['kind', p.kind]);
  if (p.count) out.push(['count', p.count]);
  if (p.per) out.push(['one per', p.per]);
  if (p.acts_for) out.push(['acts for', p.acts_for]);
  if (p.border) out.push(['border', 'a human reaches it']);
  for (const [k, v] of p.properties ?? []) out.push([k, v]);
  return out;
}

/** The links that start, end or pass at a part. */
export function linksOf(g: GraphRow, id: string): GraphLink[] {
  return g.links.filter((l) => l.from === id || l.to === id || l.over === id);
}
