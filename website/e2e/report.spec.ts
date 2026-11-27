// The round report of the docs site: reads target/site-ux/measure (written by e2e/measure.spec.ts)
// and writes ux/site/rounds/<SITE_UX_ROUND>/report.md, round.json, shots/ (the completion frames at
// the first gating viewport, the worst moments and a contact sheet per theme) and a row of
// ux/site/rounds/trend.md. Run by ux/site/round.sh; not part of task check:site.

import {execFileSync} from 'node:child_process';
import {copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync} from 'node:fs';
import {test} from '@playwright/test';
import type {PageRun, Run} from './measure.spec';
import {adapter, OUT, ROOT, UX} from './site';

const ROUND = process.env.SITE_UX_ROUND ?? '';
const DIR = `${OUT}measure/`;

const vpName = (v: {width: number; height: number}) => `${v.width}x${v.height}`;

function findings(): {id: string; severity: string; status: string; title: string}[] {
  const dir = `${UX}findings`;
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => /^SUF-\d{3}\.md$/.test(f))
    .sort()
    .map((f) => {
      const head = readFileSync(`${dir}/${f}`, 'utf8').split('\n\n')[0];
      const get = (k: string) => new RegExp(`^${k}:\\s*(.*)$`, 'm').exec(head)?.[1]?.trim() ?? '';
      return {id: get('id'), severity: get('severity'), status: get('status'), title: get('title')};
    });
}

test('write the round report', async ({page}) => {
  test.skip(!/^S-\d{2}$/.test(ROUND), 'SITE_UX_ROUND=S-nn is not set');
  test.setTimeout(60_000);
  const runs: Run[] = readdirSync(DIR)
    .filter((f) => f.endsWith('.json'))
    .map((f) => JSON.parse(readFileSync(`${DIR}${f}`, 'utf8')) as Run)
    .sort((a, b) => a.scenario.localeCompare(b.scenario) || b.viewport.width - a.viewport.width || a.viewport.height - b.viewport.height || a.theme.localeCompare(b.theme));
  const pages: PageRun[] = existsSync(`${DIR}pages`)
    ? readdirSync(`${DIR}pages`)
        .filter((f) => f.endsWith('.json'))
        .map((f) => JSON.parse(readFileSync(`${DIR}pages/${f}`, 'utf8')) as PageRun)
        .sort((a, b) => a.path.localeCompare(b.path) || a.theme.localeCompare(b.theme))
    : [];
  const rdir = `${UX}rounds/${ROUND}/`;
  mkdirSync(`${rdir}shots`, {recursive: true});
  const commit = execFileSync('git', ['rev-parse', '--short', 'HEAD'], {cwd: ROOT}).toString().trim();
  const dirty = execFileSync('git', ['status', '--porcelain', '--', 'website', 'docs', 'ux/site', ':!ux/site/rounds'], {cwd: ROOT}).toString().trim()
    ? ' with uncommitted changes in website/, docs/ or ux/site/'
    : '';
  const date = new Date().toISOString().slice(0, 10);
  const want = adapter.scenarios.length * (adapter.gating_viewports.length + adapter.measured_viewports.length) * adapter.themes.length;
  if (runs.length !== want) throw new Error(`${runs.length} measured runs, the adapter names ${want}`);

  // A run completes when its target is in view and right and the bot raised no error; it is clean
  // when no moment had a layout or axe problem; it passes when both hold.
  const completes = (r: Run) => r.completion.seen === 1 && !r.completion.wrong && !r.completion.error;
  const clean = (r: Run) => r.moments.every((m) => m.problems.length === 0);
  const ok = (r: Run) => completes(r) && clean(r);
  // A problem is named without the sizes of its box, so one defect seen at several widths counts
  // once (S-02, T6): "scrolls sideways in its box: div.x (633 > 358)" is "… div.x".
  const problemKey = (p: string) =>
    p
      .replace(/\d+ node\(s\)/, 'n nodes')
      .replace(/ \(\d+ > \d+\)/g, '')
      .replace(/ \(-?\d+\.\.-?\d+ of \d+\)/g, '')
      .replace(/: \d+ > \d+$/, '');
  const distinct = (r: Run) => new Set(r.moments.flatMap((m) => m.problems.map((p) => `${m.path} ${problemKey(p)}`))).size;
  const gating = runs.filter((r) => r.gating);
  const measured = runs.filter((r) => !r.gating);

  // Distinct problems of the task runs, with where they were seen.
  const problems = new Map<string, Set<string>>();
  for (const r of runs) {
    for (const m of r.moments) {
      for (const p of m.problems) {
        const key = `${m.path}: ${problemKey(p)}`;
        problems.set(key, (problems.get(key) ?? new Set()).add(`${vpName(r.viewport)} ${r.theme}${r.gating ? '' : ' (measured)'}`));
      }
    }
  }

  // axe-core by rule over the sweep of every page, at every viewport and in every theme (S-02, T2).
  const rules = new Map<string, {impact: string; pages: Set<string>; nodes: number; themes: Set<string>; viewports: Set<string>}>();
  for (const p of pages) {
    for (const [vp, hits] of Object.entries(p.axe)) {
      for (const v of hits) {
        const e = rules.get(v.id) ?? {impact: v.impact, pages: new Set(), nodes: 0, themes: new Set(), viewports: new Set()};
        e.pages.add(p.path);
        e.nodes += v.nodes;
        e.themes.add(p.theme);
        e.viewports.add(vp);
        rules.set(v.id, e);
      }
    }
  }
  const sweptViewports = [...new Set(pages.flatMap((p) => Object.keys(p.axe)))];
  // Layout of the sweep, by viewport: pages with a layout problem.
  const sweepLayout = new Map<string, Map<string, string[]>>();
  for (const p of pages) {
    for (const [vp, ps] of Object.entries(p.layout)) {
      if (!ps.length) continue;
      const m = sweepLayout.get(vp) ?? new Map<string, string[]>();
      m.set(p.path, ps);
      sweepLayout.set(vp, m);
    }
  }
  const consoleErrors = pages.filter((p) => p.errors.length);
  const frameRuns = pages.filter((p) => (p.frames ?? []).length);

  // Shots: completion frames at the first gating viewport, the worst moments, contact sheets.
  const first = vpName(adapter.gating_viewports[0]);
  const copy = (shot: string) => {
    copyFileSync(`${DIR}shots/${shot}`, `${rdir}shots/${shot}`);
    return `shots/${shot}`;
  };
  const completions = runs.filter((r) => vpName(r.viewport) === first).map((r) => ({r, shot: copy(r.moments[r.moments.length - 1].shot)}));
  const worst = runs
    .flatMap((r) => r.moments.map((m) => ({r, m, weight: m.problems.length + (m.label === 'completion' && !completes(r) ? 10 : 0)})))
    .filter((x) => x.weight > 0)
    .sort((a, b) => b.weight - a.weight || Number(b.r.gating) - Number(a.r.gating))
    .filter((x, i, all) => all.findIndex((y) => y.r.scenario === x.r.scenario && vpName(y.r.viewport) === vpName(x.r.viewport)) === i)
    .slice(0, 8)
    .map((x) => ({...x, shot: copy(x.m.shot)}));

  for (const theme of adapter.themes) {
    const frames = runs.filter((r) => r.theme === theme);
    const cells = frames
      .map((r) => {
        const img = readFileSync(`${DIR}shots/${r.moments[r.moments.length - 1].shot}`).toString('base64');
        return `<figure><img src="data:image/jpeg;base64,${img}"><figcaption>${r.scenario} · ${vpName(r.viewport)}${r.gating ? '' : ' (measured)'}${ok(r) ? '' : ' · problems'}</figcaption></figure>`;
      })
      .join('');
    await page.setViewportSize({width: 1800, height: 1000});
    await page.setContent(
      `<style>body{margin:16px;font:13px system-ui;background:#fff;color:#111}main{display:grid;grid-template-columns:repeat(5,1fr);gap:12px}figure{margin:0}img{width:100%;height:300px;object-fit:contain;object-position:top;border:1px solid #ccc;background:#eee}figcaption{margin-top:4px}</style><h1>${ROUND} contact sheet, ${theme} theme: completion frames</h1><main>${cells}</main>`,
    );
    await page.screenshot({path: `${rdir}shots/contact-${theme}.jpg`, type: 'jpeg', quality: 75, fullPage: true});
  }

  // Metrics against the baseline and the last round.
  const baseline = existsSync(`${UX}baseline.json`) ? JSON.parse(readFileSync(`${UX}baseline.json`, 'utf8')) : {scenarios: {}};
  const n = Number(ROUND.slice(2));
  const lastId = `S-${String(n - 1).padStart(2, '0')}`;
  const last = existsSync(`${UX}rounds/${lastId}/round.json`) ? JSON.parse(readFileSync(`${UX}rounds/${lastId}/round.json`, 'utf8')) : null;
  const metricRuns = gating.filter((r) => r.theme === adapter.metric_viewport_theme);
  const keys = Object.keys(metricRuns[0]?.metrics ?? {});

  const lines: string[] = [];
  lines.push(`# ${ROUND} report`, '');
  lines.push(`Date ${date}. Site at commit ${commit}${dirty}, built into website/build. Written by ux/site/round.sh.`, '');
  lines.push('## Levels', '');
  lines.push('- This report: L2 layout contract and axe-core at every moment, L3 screen-only user, on every docs task, viewport and theme; and a sweep of every page of the sitemap in both themes (axe-core, the layout contract at every viewport, console errors, the embedded architecture frames).');
  lines.push('- Outside it: the gates run in `task check:site`; L7, the two review agents, write review-heuristics.md and review-report.md after this report.');
  lines.push('- Not run: L0 (the site has no guidance cue), L1 (no unit tests of the site components), L4 and L5 (paid levels), L6 and L8 (people).');
  lines.push("- So: no survey, no recordings of people, no probe misses. Each run's recording is its screenshots and the bot's log in round.json.", '');
  lines.push(`Docs tasks: ${adapter.scenarios.join(', ')}; anchors ${adapter.anchor_scenarios.join(' and ')}.`, '');
  lines.push('## Gates at the gating viewports', '');
  lines.push(`- ${gating.filter(ok).length} of ${gating.length} runs pass: complete and clean.`);
  lines.push(`- ${gating.filter(completes).length} of ${gating.length} runs complete: the target in view and right.`);
  lines.push(`- ${gating.filter(clean).length} of ${gating.length} runs are clean: no layout or axe problem at any moment.`, '');
  const table = (rs: Run[]) => {
    lines.push('| task | viewport | theme | completion | right | distinct problems | bot |', '|---|---|---|---|---|---|---|');
    for (const r of rs) {
      lines.push(
        `| ${r.scenario} | ${vpName(r.viewport)} | ${r.theme} | glance ${r.completion.seen} | ${r.completion.wrong ? `no: ${r.completion.wrong}` : 'yes'} | ${distinct(r)} | ${r.completion.error ? r.completion.error.replace(/\|/g, '/') : 'done'} |`,
      );
    }
  };
  table(gating);
  lines.push('', '## Measured-only viewports (not gated)', '');
  lines.push(`- ${measured.filter(ok).length} of ${measured.length} runs pass.`);
  lines.push(`- ${measured.filter(completes).length} of ${measured.length} runs complete.`);
  lines.push(`- ${measured.filter(clean).length} of ${measured.length} runs are clean.`, '');
  table(measured);
  lines.push('', '## Problems seen in the task runs', '');
  lines.push(`${problems.size} distinct problems (page and problem).`, '');
  for (const [p, where] of [...problems.entries()].sort((a, b) => b[1].size - a[1].size || a[0].localeCompare(b[0]))) lines.push(`- ${p} (${where.size}: ${[...where].sort().join(', ')})`);
  lines.push('', '## Page sweep', '');
  lines.push(`${new Set(pages.map((p) => p.path)).size} pages of the sitemap, each in ${adapter.themes.join(' and ')}.`, '');
  lines.push('### axe-core, serious and critical, by rule', '', `At ${sweptViewports.join(', ')}, in ${adapter.themes.join(' and ')}.`, '');
  if (!rules.size) lines.push('none');
  else {
    lines.push('| rule | impact | viewports | pages | nodes (sum over viewports and themes) | themes |', '|---|---|---|---|---|---|');
    for (const [id, e] of [...rules.entries()].sort((a, b) => b[1].pages.size - a[1].pages.size || a[0].localeCompare(b[0]))) {
      lines.push(`| ${id} | ${e.impact} | ${[...e.viewports].join(', ')} | ${e.pages.size}: ${[...e.pages].slice(0, 6).join(', ')}${e.pages.size > 6 ? ', …' : ''} | ${e.nodes} | ${[...e.themes].sort().join(', ')} |`);
    }
  }
  lines.push('', '### Layout, by viewport (light theme)', '');
  if (!sweepLayout.size) lines.push('none');
  for (const vp of [...adapter.gating_viewports, ...adapter.measured_viewports].map(vpName)) {
    const m = sweepLayout.get(vp);
    if (!m) continue;
    const gate = adapter.gating_viewports.map(vpName).includes(vp) ? 'gating' : 'measured';
    lines.push(`- ${vp} (${gate}): ${m.size} page(s)`);
    for (const [path, ps] of m) lines.push(`  - ${path}: ${[...new Set(ps)].slice(0, 3).join('; ')}${ps.length > 3 ? `; and ${ps.length - 3} more` : ''}`);
  }
  lines.push('', '### Embedded architecture frames', '');
  if (!frameRuns.length) lines.push('none');
  for (const p of frameRuns) lines.push(`- ${p.path} ${p.theme}: ${p.frames.join('; ')}`);
  lines.push('', '### Console errors', '');
  if (!consoleErrors.length) lines.push('none');
  for (const p of consoleErrors) lines.push(`- ${p.path} ${p.theme}: ${p.errors.slice(0, 2).join('; ')}`);
  lines.push('', `## Metrics, ${adapter.metric_viewport_theme} theme (lower is better)`, '');
  lines.push('A run that stopped before its target has no metrics here and in no sum (S-02, T5).', '');
  lines.push(`| task | ${keys.join(' | ')} |`, `|---|${keys.map(() => '---').join('|')}|`);
  for (const r of metricRuns) {
    const id = `${r.scenario}@${vpName(r.viewport)}`;
    if (r.completion.error) {
      lines.push(`| ${id} | stopped: ${r.completion.error.replace(/\|/g, '/')} |${keys.slice(1).map(() => ' |').join('')}`);
      continue;
    }
    const cells = keys.map((k) => {
      const b = baseline.scenarios[id]?.[k]?.value;
      const l = last?.metrics?.[id]?.[k];
      return `${r.metrics[k] ?? '–'}${b !== undefined ? ` (base ${b})` : ''}${l !== undefined && l !== r.metrics[k] ? ` (${lastId} ${l})` : ''}`;
    });
    lines.push(`| ${id} | ${cells.join(' | ')} |`);
  }
  lines.push('', '## Worst moments', '');
  if (!worst.length) lines.push('none');
  for (const w of worst) {
    lines.push(
      `- ${w.r.scenario} ${vpName(w.r.viewport)} ${w.r.theme}, ${w.m.label} on ${w.m.path}: ${w.m.problems.length} problem(s)${completes(w.r) ? '' : ', the run does not complete'}${w.r.gating ? '' : ' (measured only)'}. ![](${w.shot})`,
    );
  }
  lines.push('', '## Completion frames', '');
  for (const c of completions) lines.push(`- ${c.r.scenario} ${first} ${c.r.theme}: ![](${c.shot})`);
  lines.push('', `Contact sheets: ![light](shots/contact-light.jpg) ![dark](shots/contact-dark.jpg)`, '');
  const fs = findings();
  lines.push('## Findings', '', 'As of this report; the review agents file theirs after it.', '');
  lines.push(fs.length ? fs.map((f) => `- ${f.id} ${f.severity} ${f.status}: ${f.title}`).join('\n') : 'none');
  lines.push('');
  writeFileSync(`${rdir}report.md`, lines.join('\n'));

  const metrics = Object.fromEntries(metricRuns.filter((r) => !r.completion.error).map((r) => [`${r.scenario}@${vpName(r.viewport)}`, r.metrics]));
  writeFileSync(
    `${rdir}round.json`,
    `${JSON.stringify(
      {
        round: ROUND,
        date,
        commit: commit + (dirty ? '+dirty' : ''),
        gating: {runs: gating.length, pass: gating.filter(ok).length, complete: gating.filter(completes).length, clean: gating.filter(clean).length},
        measured: {runs: measured.length, pass: measured.filter(ok).length, complete: measured.filter(completes).length, clean: measured.filter(clean).length},
        problems: Object.fromEntries([...problems.entries()].map(([p, w]) => [p, [...w].sort()])),
        sweep: {
          pages: new Set(pages.map((p) => p.path)).size,
          axe: Object.fromEntries([...rules.entries()].map(([id, e]) => [id, {impact: e.impact, viewports: [...e.viewports], pages: [...e.pages], nodes: e.nodes, themes: [...e.themes].sort()}])),
          layout: Object.fromEntries([...sweepLayout.entries()].map(([vp, m]) => [vp, Object.fromEntries(m)])),
          console_errors: Object.fromEntries(consoleErrors.map((p) => [`${p.path} ${p.theme}`, p.errors])),
          frames: Object.fromEntries(frameRuns.map((p) => [`${p.path} ${p.theme}`, p.frames])),
        },
        metrics,
        runs: runs.map((r) => ({
          scenario: r.scenario,
          viewport: vpName(r.viewport),
          theme: r.theme,
          gating: r.gating,
          completion: r.completion,
          problems: r.moments.flatMap((m) => m.problems.map((p) => `${m.label} (${m.path}): ${p}`)),
          log: r.log,
        })),
      },
      null,
      2,
    )}\n`,
  );

  // trend.md: one row per round.
  const trend = `${UX}rounds/trend.md`;
  const head =
    '# Trend\n\nOne row per round of the docs site (ux/site/round.sh).\n\n| round | date | gating pass | measured pass | distinct problems | axe rules (sweep) | actions (sum) | looks (sum) | reading load max (sum) | open S1/S2/S3 |\n|---|---|---|---|---|---|---|---|---|---|\n';
  const rows = existsSync(trend) ? readFileSync(trend, 'utf8').split('\n').filter((l) => /^\| S-\d{2} /.test(l) && !l.startsWith(`| ${ROUND} `)) : [];
  const sum = (k: string) => metricRuns.filter((r) => !r.completion.error).reduce((a, r) => a + (r.metrics[k] ?? 0), 0);
  const open = (s: string) => fs.filter((f) => f.severity === s && f.status !== 'closed').length;
  rows.push(
    `| ${ROUND} | ${date} | ${gating.filter(ok).length}/${gating.length} | ${measured.filter(ok).length}/${measured.length} | ${problems.size} | ${rules.size} | ${sum('screen_only.actions')} | ${sum('screen_only.looks')} | ${sum('reading_load.max')} | ${open('S1')}/${open('S2')}/${open('S3')} |`,
  );
  writeFileSync(trend, head + rows.sort().join('\n') + '\n');
});
