import {execFileSync} from 'node:child_process';
import {defineConfig} from '@playwright/test';

// The fast loop of ux/site/adapter.json on the built site (website/build), served by
// `docusaurus serve` on a free port of 127.0.0.1: the gates (layout contract, axe-core, keyboard,
// the screen-only user, the page sweep), then the ratchet against ux/site/baseline.json.
// A test is killed at 60 s and the run at 120 s; the ten slowest tests are printed.
// A round (ux/site/round.sh, SITE_UX_ROUND set) runs the projects measure and report instead.

// The workers load this file again; the port is chosen once, in the runner, and passed on in the
// environment, so every worker reaches the same server.
if (!process.env.SITE_UX_PORT) {
  process.env.SITE_UX_PORT = execFileSync(process.execPath, [
    '-e',
    "const s=require('net').createServer();s.listen(0,'127.0.0.1',()=>{process.stdout.write(String(s.address().port));s.close()})",
  ])
    .toString()
    .trim();
}
const port = Number(process.env.SITE_UX_PORT);
if (port === 8471 || port === 8472) throw new Error(`port ${port} serves the human; pick another SITE_UX_PORT`);
const origin = `http://127.0.0.1:${port}`;

// The bound of the whole run: 120 s for the gate (projects gates and ratchet); 14 min for a round,
// the projects measure and report, which play every task at five viewports and sweep every page
// (about 7 min at 2 workers in S-01), whether SITE_UX_ROUND is set or the project is named alone
// (S-02, T9). Each test is still killed at 60 s.
const projectArgs = process.argv.flatMap((a, i, all) => (a.startsWith('--project=') ? [a.slice(10)] : a === '--project' ? [all[i + 1] ?? ''] : []));
const round = !!process.env.SITE_UX_ROUND || projectArgs.some((p) => p === 'measure' || p === 'report');

export default defineConfig({
  testDir: 'e2e',
  timeout: 60_000,
  globalTimeout: round ? 840_000 : 120_000,
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [['list'], ['./e2e/slowest.ts']],
  reportSlowTests: null,
  outputDir: '../target/site-ux/test-results',
  use: {browserName: 'chromium', headless: true, baseURL: `${origin}/agenticspec/`},
  webServer: {
    command: `npx docusaurus serve --dir build --host 127.0.0.1 --port ${port} --no-open`,
    url: `${origin}/agenticspec/`,
    reuseExistingServer: false,
    timeout: 30_000,
    stdout: 'ignore',
    stderr: 'pipe',
  },
  projects: [
    {name: 'gates', testMatch: /(gates|keyboard|pages|findings)\.spec\.ts/},
    {name: 'ratchet', testMatch: /ratchet\.spec\.ts/, dependencies: ['gates']},
    {name: 'measure', testMatch: /measure\.spec\.ts/},
    {name: 'report', testMatch: /report\.spec\.ts/, dependencies: ['measure']},
  ],
});
