import { defineConfig } from '@playwright/test';

// The browser test of the custom elements of ui/views in a plain page and a React page over one
// fixture server: the part page's panels show the fixture's rows, the host's --kit-accent
// applies, and a link and an action send their DOM events without changing the URL. A test is
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
  outputDir: '../../../../target/views/hosts-test-results',
  globalSetup: './global-setup.ts',
  use: { browserName: 'chromium', headless: true },
});
