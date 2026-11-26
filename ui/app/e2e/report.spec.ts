// The round report: reads target/ux/measure (written by e2e/measure.spec.ts) and writes
// ux/rounds/<UX_ROUND>/report.md, round.json, shots/ (the completion frames at the first gating
// viewport, the worst moments and a contact sheet per theme) and a row of ux/rounds/trend.md.
// Run by ux/round.sh; not part of task check.

import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { test } from '@playwright/test';
import type { Run } from './measure.spec.ts';
import { adapter, OUT, UX } from './scenarios.ts';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const ROUND = process.env.UX_ROUND ?? '';
const DIR = `${OUT}measure/`;

const vpName = (v: { width: number; height: number }) => `${v.width}x${v.height}`;

function findings(): { id: string; severity: string; status: string; title: string }[] {
  const dir = `${UX}findings`;
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => /^UXF-\d{3}\.md$/.test(f))
    .sort()
    .map((f) => {
      const head = readFileSync(`${dir}/${f}`, 'utf8').split('\n\n')[0];
      const get = (k: string) => new RegExp(`^${k}:\\s*(.*)$`, 'm').exec(head)?.[1]?.trim() ?? '';
      return { id: get('id'), severity: get('severity'), status: get('status'), title: get('title') };
    });
}

test('write the round report', async ({ page }) => {
  test.skip(!/^R-\d{2}$/.test(ROUND), 'UX_ROUND=R-nn is not set');
  test.setTimeout(60_000);
  const runs: Run[] = readdirSync(DIR)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(`${DIR}${f}`, 'utf8')) as Run)
    .sort((a, b) => a.scenario.localeCompare(b.scenario) || b.viewport.width - a.viewport.width || a.theme.localeCompare(b.theme));
  const rdir = `${UX}rounds/${ROUND}/`;
  mkdirSync(`${rdir}shots`, { recursive: true });
  const commit = execFileSync('git', ['rev-parse', '--short', 'HEAD'], { cwd: ROOT }).toString().trim();
  const dirty = execFileSync('git', ['status', '--porcelain', '--', 'ui/app/src'], { cwd: ROOT }).toString().trim() ? ' with uncommitted changes in ui/app/src' : '';
  const date = new Date().toISOString().slice(0, 10);

  const ok = (r: Run) => r.completion.seen === 1 && r.completion.right && r.moments.every((m) => m.problems.length === 0);
  const gating = runs.filter((r) => r.gating);
  const measured = runs.filter((r) => !r.gating);

  // Distinct problems, with where they were seen.
  const problems = new Map<string, Set<string>>();
  for (const r of runs) {
    for (const m of r.moments) {
      for (const p of m.problems) {
        const key = p.replace(/\d+ node\(s\)/, 'n nodes');
        problems.set(key, (problems.get(key) ?? new Set()).add(`${vpName(r.viewport)} ${r.theme}${r.gating ? '' : ' (measured)'}`));
      }
    }
  }

  // Shots: completion frames at the first gating viewport, the worst moments, contact sheets.
  const first = vpName(adapter.gating_viewports[0]);
  const copy = (shot: string) => {
    copyFileSync(`${DIR}shots/${shot}`, `${rdir}shots/${shot}`);
    return `shots/${shot}`;
  };
  const completions = runs.filter((r) => vpName(r.viewport) === first).map((r) => ({ r, shot: copy(r.moments[r.moments.length - 1].shot) }));
  const worst = runs
    .flatMap((r) => r.moments.map((m) => ({ r, m, weight: m.problems.length + (m.label === 'completion' && !ok(r) ? 10 : 0) })))
    .filter((x) => x.weight > 0)
    .sort((a, b) => b.weight - a.weight || Number(b.r.gating) - Number(a.r.gating))
    .filter((x, i, all) => all.findIndex((y) => y.r.scenario === x.r.scenario && vpName(y.r.viewport) === vpName(x.r.viewport)) === i)
    .slice(0, 6)
    .map((x) => ({ ...x, shot: copy(x.m.shot) }));

  for (const theme of adapter.themes) {
    const frames = runs.filter((r) => r.theme === theme);
    const cells = frames
      .map((r) => {
        const img = readFileSync(`${DIR}shots/${r.moments[r.moments.length - 1].shot}`).toString('base64');
        return `<figure><img src="data:image/jpeg;base64,${img}"><figcaption>${r.scenario} · ${vpName(r.viewport)}${r.gating ? '' : ' (measured)'}${ok(r) ? '' : ' · problems'}</figcaption></figure>`;
      })
      .join('');
    await page.setViewportSize({ width: 1800, height: 1000 });
    await page.setContent(
      `<style>body{margin:16px;font:13px system-ui;background:#fff;color:#111}main{display:grid;grid-template-columns:repeat(5,1fr);gap:12px}figure{margin:0}img{width:100%;height:200px;object-fit:contain;object-position:top;border:1px solid #ccc;background:#eee}figcaption{margin-top:4px}</style><h1>${ROUND} contact sheet, ${theme} theme: completion frames</h1><main>${cells}</main>`,
    );
    await page.screenshot({ path: `${rdir}shots/contact-${theme}.jpg`, type: 'jpeg', quality: 75, fullPage: true });
  }

  // Metrics against the baseline and the last round.
  const baseline = existsSync(`${UX}baseline.json`) ? JSON.parse(readFileSync(`${UX}baseline.json`, 'utf8')) : { scenarios: {} };
  const n = Number(ROUND.slice(2));
  const lastId = `R-${String(n - 1).padStart(2, '0')}`;
  const last = existsSync(`${UX}rounds/${lastId}/round.json`) ? JSON.parse(readFileSync(`${UX}rounds/${lastId}/round.json`, 'utf8')) : null;
  const metricRuns = gating.filter((r) => r.theme === adapter.metric_viewport_theme);
  const keys = Object.keys(metricRuns[0]?.metrics ?? {});

  const lines: string[] = [];
  lines.push(`# ${ROUND} report`, '');
  lines.push(`Date ${date}. App at commit ${commit}${dirty}. Fixtures: ui/fixtures (synthetic). Written by ux/round.sh.`, '');
  lines.push('## Levels', '');
  lines.push('- Run: L1 unit (vitest), L2 layout contract and axe-core, L3 screen-only user, L7 two review agents.');
  lines.push('- Not run: L0 (the UI has no guidance cue), L4 and L5 (paid levels), L6 and L8 (people).');
  lines.push('- So: no survey, no recordings of people, no probe misses. Each run\'s recording is its screenshots and the bot\'s log in round.json.', '');
  lines.push('## Gates at the gating viewports', '');
  lines.push(`${gating.filter(ok).length} of ${gating.length} runs pass every gate (completion in view and right, no layout or axe problem at any moment).`, '');
  lines.push('| scenario | viewport | theme | completion | right | problems |', '|---|---|---|---|---|---|');
  for (const r of gating) {
    lines.push(`| ${r.scenario} | ${vpName(r.viewport)} | ${r.theme} | glance ${r.completion.seen} | ${r.completion.right ? 'yes' : 'no'} | ${r.moments.reduce((a, m) => a + m.problems.length, 0)} |`);
  }
  lines.push('', '## Measured-only viewports', '');
  lines.push(`${measured.filter(ok).length} of ${measured.length} runs pass every gate. Not gated.`, '');
  lines.push('| scenario | viewport | theme | completion | right | problems |', '|---|---|---|---|---|---|');
  for (const r of measured) {
    lines.push(`| ${r.scenario} | ${vpName(r.viewport)} | ${r.theme} | glance ${r.completion.seen} | ${r.completion.right ? 'yes' : 'no'} | ${r.moments.reduce((a, m) => a + m.problems.length, 0)} |`);
  }
  lines.push('', '## Problems seen', '');
  if (!problems.size) lines.push('none');
  for (const [p, where] of [...problems.entries()].sort((a, b) => b[1].size - a[1].size)) lines.push(`- ${p} (${where.size}: ${[...where].sort().join(', ')})`);
  lines.push('', `## Metrics, ${adapter.metric_viewport_theme} theme (lower is better)`, '');
  lines.push(`| scenario | ${keys.join(' | ')} |`, `|---|${keys.map(() => '---').join('|')}|`);
  for (const r of metricRuns) {
    const id = `${r.scenario}@${vpName(r.viewport)}`;
    const cells = keys.map((k) => {
      const b = baseline.scenarios[id]?.[k]?.value;
      const l = last?.metrics?.[id]?.[k];
      return `${r.metrics[k]}${b !== undefined ? ` (base ${b})` : ''}${l !== undefined && l !== r.metrics[k] ? ` (${lastId} ${l})` : ''}`;
    });
    lines.push(`| ${id} | ${cells.join(' | ')} |`);
  }
  lines.push('', '## Worst moments', '');
  if (!worst.length) lines.push('none');
  for (const w of worst) lines.push(`- ${w.r.scenario} ${vpName(w.r.viewport)} ${w.r.theme}, ${w.m.label}: ${w.m.problems.length} problem(s)${ok(w.r) ? '' : ', run fails a gate'}. ![](${w.shot})`);
  lines.push('', '## Completion frames', '');
  for (const c of completions) lines.push(`- ${c.r.scenario} ${first} ${c.r.theme}: ![](${c.shot})`);
  lines.push('', `Contact sheets: ![light](shots/contact-light.jpg) ![dark](shots/contact-dark.jpg)`, '');
  const fs = findings();
  lines.push('## Findings', '');
  lines.push(fs.length ? fs.map((f) => `- ${f.id} ${f.severity} ${f.status}: ${f.title}`).join('\n') : 'none');
  lines.push('');
  writeFileSync(`${rdir}report.md`, lines.join('\n'));

  const metrics = Object.fromEntries(metricRuns.map((r) => [`${r.scenario}@${vpName(r.viewport)}`, r.metrics]));
  writeFileSync(
    `${rdir}round.json`,
    `${JSON.stringify(
      {
        round: ROUND,
        date,
        commit: commit + (dirty ? '+dirty' : ''),
        gating: { runs: gating.length, pass: gating.filter(ok).length },
        measured: { runs: measured.length, pass: measured.filter(ok).length },
        problems: Object.fromEntries([...problems.entries()].map(([p, w]) => [p, [...w].sort()])),
        metrics,
        runs: runs.map((r) => ({ scenario: r.scenario, viewport: vpName(r.viewport), theme: r.theme, gating: r.gating, completion: r.completion, problems: r.moments.flatMap((m) => m.problems.map((p) => `${m.label}: ${p}`)), log: r.log })),
      },
      null,
      2,
    )}\n`,
  );

  // trend.md: one row per round.
  const trend = `${UX}rounds/trend.md`;
  const head = '# Trend\n\nOne row per round (ux/round.sh).\n\n| round | date | gating pass | measured pass | distinct problems | actions (sum) | looks (sum) | reading load max (sum) | open S1/S2/S3 |\n|---|---|---|---|---|---|---|---|---|\n';
  const rows = existsSync(trend) ? readFileSync(trend, 'utf8').split('\n').filter((l) => /^\| R-\d{2} /.test(l) && !l.startsWith(`| ${ROUND} `)) : [];
  const sum = (k: string) => metricRuns.reduce((a, r) => a + (r.metrics[k] ?? 0), 0);
  const open = (s: string) => fs.filter((f) => f.severity === s && f.status !== 'closed').length;
  rows.push(
    `| ${ROUND} | ${date} | ${gating.filter(ok).length}/${gating.length} | ${measured.filter(ok).length}/${measured.length} | ${problems.size} | ${sum('screen_only.actions')} | ${sum('screen_only.looks')} | ${sum('reading_load.max')} | ${open('S1')}/${open('S2')}/${open('S3')} |`,
  );
  writeFileSync(trend, head + rows.sort().join('\n') + '\n');
});
