import { defineConfig } from '@playwright/test';

// The browser test of the gallery of the block panels: every panel shows its fixture rows, axe-core
// finds nothing serious or critical and nothing runs past the viewport sideways, at 1366x768 and
// 390x844, in both themes; with SCREENS=1 it writes screenshots into target/screens. A test is
// killed at 60 s and the run at 120 s; the slowest tests are printed.
export default defineConfig({
  testDir: '.',
  testMatch: /.*\.spec\.ts/,
  timeout: 60_000,
  globalTimeout: 120_000,
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [['list'], ['./slowest.ts']],
  outputDir: '../../../../target/views/test-results',
  globalSetup: './global-setup.ts',
  use: { browserName: 'chromium', headless: true },
});
