// The ratchet against ux/site/baseline.json (all keys lower is better) and the findings rule of
// ux/site/findings. Runs after the gates, which wrote target/site-ux/metrics. Fails on a value
// worse than the baseline beyond its tolerance, on an improvement not taken into the baseline, on a
// missing value, and on a baseline made worse than on main without a decision id.
// SITE_UX_RATCHET=write takes the metrics into the baseline (an improvement, or a new scenario).

import {execFileSync} from 'node:child_process';
import {existsSync, readdirSync, readFileSync, writeFileSync} from 'node:fs';
import {expect, test} from '@playwright/test';
import type {Metrics} from './bot';
import {adapter, OUT, ROOT, UX} from './site';

const BASELINE = `${UX}baseline.json`;
const ABOUT =
  'Lower is better. Written by SITE_UX_RATCHET=write from target/site-ux/metrics (website/e2e/ratchet.spec.ts); one entry per docs task and gating viewport, light theme.';

interface Entry {
  value: number;
  tolerance: number;
  decision?: string;
}
interface Baseline {
  about: string;
  tolerances: Record<string, string>;
  scenarios: Record<string, Record<string, Entry>>;
}

const TOLERANCES = {
  counts: 'exact (0), for actions, looks, steers and glances',
  steps: '5 % of the value, for time to act in steps and reading load in words',
  layout_shift: '0.01',
};

function tolerance(key: string, value: number): number {
  if (key.startsWith('layout_shift')) return 0.01;
  if (key.startsWith('reading_load') || key.includes('time_to_act')) return Math.round(value * 0.05 * 100) / 100;
  return 0;
}

function measured(): Record<string, Metrics> {
  const out: Record<string, Metrics> = {};
  for (const name of adapter.scenarios) {
    for (const vp of adapter.gating_viewports) {
      const id = `${name}@${vp.width}x${vp.height}`;
      const f = `${OUT}metrics/${id}.json`;
      if (existsSync(f)) out[id] = JSON.parse(readFileSync(f, 'utf8')).metrics;
    }
  }
  return out;
}

function onMain(): Baseline | null {
  for (const ref of ['origin/main', 'main']) {
    try {
      return JSON.parse(execFileSync('git', ['show', `${ref}:ux/site/baseline.json`], {cwd: ROOT, stdio: ['ignore', 'pipe', 'ignore']}).toString());
    } catch {
      // not on this ref
    }
  }
  return null;
}

test('the metrics hold the baseline', () => {
  const now = measured();
  const want = adapter.scenarios.length * adapter.gating_viewports.length;
  const base: Baseline = existsSync(BASELINE) ? JSON.parse(readFileSync(BASELINE, 'utf8')) : {about: '', tolerances: TOLERANCES, scenarios: {}};
  if (process.env.SITE_UX_RATCHET === 'write') {
    for (const [id, m] of Object.entries(now)) {
      const old = base.scenarios[id] ?? {};
      base.scenarios[id] = Object.fromEntries(
        Object.entries(m)
          .filter((kv): kv is [string, number] => kv[1] !== null)
          .map(([k, v]) => [k, {value: v, tolerance: tolerance(k, v), ...(old[k]?.decision ? {decision: old[k].decision} : {})}]),
      );
    }
    // Lines after the standard text (what changed what is measured) stay.
    base.about = base.about?.startsWith(ABOUT) ? base.about : ABOUT;
    base.tolerances = TOLERANCES;
    writeFileSync(BASELINE, `${JSON.stringify(base, null, 2)}\n`);
  }
  const problems: string[] = [];
  if (Object.keys(now).length !== want) problems.push(`metrics of ${Object.keys(now).length} of ${want} task runs`);
  for (const [id, m] of Object.entries(now)) {
    const b = base.scenarios[id];
    if (!b) {
      problems.push(`${id}: not in the baseline`);
      continue;
    }
    for (const [k, v] of Object.entries(m)) {
      const e = b[k];
      if (!e) problems.push(`${id} ${k}: no baseline value`);
      else if (v === null) problems.push(`${id} ${k}: no value, the run stopped before its target`);
      else if (v > e.value + e.tolerance) problems.push(`${id} ${k}: ${v}, worse than ${e.value} (tolerance ${e.tolerance})`);
      else if (v < e.value - e.tolerance) problems.push(`${id} ${k}: ${v}, better than ${e.value}: take it with SITE_UX_RATCHET=write`);
    }
    for (const k of Object.keys(b)) if (!(k in m)) problems.push(`${id} ${k}: missing value`);
  }
  expect(problems).toEqual([]);
});

test('the baseline is no worse than on main without a decision id', () => {
  const main = onMain();
  test.skip(!main || !existsSync(BASELINE), 'no site baseline on main yet');
  const base: Baseline = JSON.parse(readFileSync(BASELINE, 'utf8'));
  const problems: string[] = [];
  for (const [id, keys] of Object.entries(main!.scenarios)) {
    for (const [k, e] of Object.entries(keys)) {
      const cur = base.scenarios[id]?.[k];
      if (!cur) problems.push(`${id} ${k}: dropped from the baseline`);
      else if (cur.value > e.value + e.tolerance && !cur.decision) problems.push(`${id} ${k}: ${e.value} to ${cur.value} without a decision id`);
    }
  }
  expect(problems).toEqual([]);
});

const FIELDS = [
  'id',
  'title',
  'class',
  'severity',
  'metric',
  'status',
  'found_by',
  'found_on',
  'evidence_recording',
  'evidence_screenshot',
  'evidence_lines',
  'regression_asset',
];

test('every site finding has its header, and every closed S1 or S2 finding a regression asset', () => {
  const dir = `${UX}findings`;
  const files = existsSync(dir) ? readdirSync(dir).filter((f) => /^SUF-\d{3}\.md$/.test(f)) : [];
  const problems: string[] = [];
  for (const f of files) {
    const head = readFileSync(`${dir}/${f}`, 'utf8').split('\n\n')[0];
    const fields = Object.fromEntries(
      head
        .split('\n')
        .map((l) => /^([a-z_]+):\s*(.*)$/.exec(l))
        .filter((m): m is RegExpExecArray => !!m)
        .map((m) => [m[1], m[2].trim()]),
    );
    for (const k of FIELDS) if (!(k in fields)) problems.push(`${f}: no ${k}`);
    if (fields.id !== f.replace(/\.md$/, '')) problems.push(`${f}: id ${fields.id}`);
    if (fields.status === 'closed' && (fields.severity === 'S1' || fields.severity === 'S2')) {
      const asset = (fields.regression_asset ?? '').split(/[\s#]/)[0];
      if (!asset || asset === 'none') problems.push(`${f}: closed ${fields.severity} without a regression asset`);
      else if (!existsSync(`${ROOT}${asset}`)) problems.push(`${f}: regression asset ${asset} does not exist`);
    }
  }
  expect(problems).toEqual([]);
});
