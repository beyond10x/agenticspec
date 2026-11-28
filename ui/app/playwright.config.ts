import { defineConfig } from '@playwright/test';

// The fast loop of ux/adapter.json: gates (layout contract, axe-core, keyboard, the screen-only
// user) on the built app with the fixtures behind it, then the ratchet against ux/baseline.json.
// A test is killed at 60 s and the run at 120 s; the ten slowest tests are printed.
export default defineConfig({
  testDir: 'e2e',
  timeout: 60_000,
  // A round (ux/round.sh, UX_ROUND set) measures 50 runs; the fast loop has 120 s.
  globalTimeout: process.env.UX_ROUND ? 540_000 : 120_000,
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [['list'], ['../views/hosts/e2e/slowest.ts']],
  outputDir: '../../target/ux/test-results',
  use: { browserName: 'chromium', headless: true },
  projects: [
    { name: 'gates', testMatch: /(gates|keyboard|findings)\.spec\.ts/ },
    { name: 'ratchet', testMatch: /ratchet\.spec\.ts/, dependencies: ['gates'] },
    { name: 'measure', testMatch: /measure\.spec\.ts/ },
    { name: 'report', testMatch: /report\.spec\.ts/, dependencies: ['measure'] },
  ],
});
