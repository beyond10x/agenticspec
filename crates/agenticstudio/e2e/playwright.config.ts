import { defineConfig } from '@playwright/test';

// The browser test of the agenticstudio page. global-setup.ts starts one `agenticstudio serve`
// for the whole run, on a free port of 127.0.0.1, from the binary task check:studio built. The
// tests run in order on one draft. A test is killed at 60 s and the run at 120 s; the ten slowest
// tests are printed.
export default defineConfig({
  testDir: '.',
  testMatch: /.*\.spec\.ts/,
  timeout: 60_000,
  globalTimeout: 120_000,
  fullyParallel: false,
  workers: 1,
  forbidOnly: true,
  retries: 0,
  reporter: [['list'], ['../../../ui/views/hosts/e2e/slowest.ts']],
  outputDir: '../../../target/studio/test-results',
  globalSetup: './global-setup.ts',
  use: { browserName: 'chromium', headless: true },
});
