import { defineConfig } from '@playwright/test';

// The browser test of the operator UI `agenticspec ui build` writes, over one static server and
// fixture adapter started once for the run. A test is killed at 60 s and the run at 120 s; the
// slowest tests are printed.
export default defineConfig({
  testDir: '.',
  testMatch: /.*\.spec\.ts/,
  timeout: 60_000,
  globalTimeout: 120_000,
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [['list'], ['../../views/hosts/e2e/slowest.ts']],
  outputDir: '../../../target/operator/test-results',
  globalSetup: './global-setup.ts',
  use: { browserName: 'chromium', headless: true },
});
