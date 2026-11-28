import { defineConfig } from '@playwright/test';

// The browser test of the ui/kit gallery: axe-core, horizontal overflow, the select's keyboard,
// and (with SCREENS=1) the screenshots in target/screens. A test is killed at 60 s and the run at
// 120 s; the slowest files are printed.
export default defineConfig({
  testDir: '.',
  testMatch: /.*\.spec\.ts/,
  timeout: 60_000,
  globalTimeout: 120_000,
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: [['list']],
  reportSlowTests: { max: 10, threshold: 0 },
  outputDir: '../../../target/ui-kit/test-results',
  globalSetup: './global-setup.ts',
  use: { browserName: 'chromium', headless: true },
});
