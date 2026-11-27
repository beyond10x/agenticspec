/**
 * The data file of the agentic ladder, website/static/ladder/ladder.json, which
 * `agenticspec-sim ladder` writes from ladder/ladder.yaml and the rung architectures
 * (task ladder:generate). Never edited by hand.
 */
import raw from '@site/static/ladder/ladder.json';

export interface Stage {
  id: string;
  name: string;
  marker: string;
  trust_need: string;
  unit_of_work: string;
  rungs: number[];
}

export interface Barrier {
  id: string;
  name: string;
  between: [string, string];
  decision: string;
  before_rung: number;
}

export interface Part {
  id: string;
  block: string;
  family: string;
  label: string;
  count: number;
  trust: string | null;
}

export interface Link {
  from: string;
  to: string;
  carries: string;
}

export interface Simulation {
  tasks_done: number;
  tasks_correct: number;
  false_completions_shipped: number;
  owner_wait_share: number;
  cost_usd: number;
}

export interface View {
  col0: number;
  col1: number;
  row0: number;
  row1: number;
}

export interface Rung {
  id: string;
  n: number;
  stage: string;
  title: string;
  changed: string;
  risk: string | null;
  architecture: string;
  name: string;
  summary: string;
  parts: Part[];
  links: Link[];
  added_parts: string[];
  removed_parts: string[];
  changed_parts: string[];
  added_links: string[];
  removed_links: string[];
  trust_added: string[];
  agents_at_work: number;
  trust_parts: number;
  view: View;
  simulation?: Simulation;
}

export interface Band {
  name: string;
  row0: number;
  row1: number;
}

export interface LadderData {
  format: string;
  title: string;
  summary: string;
  rule: string;
  stages: Stage[];
  barriers: Barrier[];
  rungs: Rung[];
  layout: {
    cols: number;
    rows: number;
    bands: Band[];
    parts: Record<string, {col: number; row: number}>;
  };
  simulation?: {
    profile: string;
    horizon: string;
    seeds: number;
    seed: number;
    tasks_per_hour_per_worker: number;
  };
}

export const ladder = raw as unknown as LadderData;

/** One step of the animation: a rung, or the barrier before the first rung of a stage. */
export type Step = {kind: 'rung'; rung: number} | {kind: 'barrier'; barrier: number};

export const steps: Step[] = ladder.rungs.flatMap((_, i) => {
  const out: Step[] = [];
  ladder.barriers.forEach((b, k) => {
    if (b.before_rung === i) out.push({kind: 'barrier', barrier: k});
  });
  out.push({kind: 'rung', rung: i});
  return out;
});

/** The rung a step draws: a barrier draws the rung before it. */
export function rungOf(step: Step): number {
  return step.kind === 'rung' ? step.rung : Math.max(0, ladder.barriers[step.barrier].before_rung - 1);
}

export function hashOf(step: Step): string {
  return step.kind === 'rung' ? `rung-${step.rung + 1}` : `barrier-${step.barrier + 1}`;
}

export function stepOfHash(hash: string): number | null {
  const m = /^#(rung|barrier)-(\d+)$/.exec(hash);
  if (!m) return null;
  const n = Number(m[2]) - 1;
  const i = steps.findIndex((s) =>
    m[1] === 'rung' ? s.kind === 'rung' && s.rung === n : s.kind === 'barrier' && s.barrier === n,
  );
  return i < 0 ? null : i;
}

export function stageOf(id: string): Stage {
  return ladder.stages.find((s) => s.id === id) ?? ladder.stages[0];
}

const GITHUB = 'https://github.com/beyond10x/agenticspec/blob/main/';

/** The repository path of a file named relative to ladder/ladder.yaml. */
export function repositoryPath(relative: string): string {
  const out: string[] = ['ladder'];
  for (const part of relative.split('/')) {
    if (part === '..') out.pop();
    else if (part !== '.' && part !== '') out.push(part);
  }
  return out.join('/');
}

export function sourceUrl(relative: string): string {
  return GITHUB + repositoryPath(relative);
}
