import { defineConfig } from '@playwright/test';

// The browser test of the derived examples: the 37-part example at 1366x768 (the parts table on
// the first screen, no sideways overflow, every part in two steps, the reach table filtered by
// part, the graph with pan, zoom and fit, nothing serious or critical for axe-core) and the
// workstation's labels, frames and actions. A test is killed at 60 s and the run at 120 s.
export default defineConfig({
  testDir: '.',
  testMatch: /.*\.spec\.ts/,
  timeout: 60_000,
  globalTimeout: 120_000,
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [['list'], ['./slowest.ts']],
  outputDir: '../../../../../target/views/examples-test-results',
  globalSetup: './global-setup.ts',
  use: { browserName: 'chromium', headless: true },
});
