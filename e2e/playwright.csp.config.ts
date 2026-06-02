import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  testMatch: '**/csp.spec.ts',
  snapshotDir: './tests/snapshots',
  fullyParallel: false,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI
    ? [['github'], ['json', { outputFile: 'reports/csp-results.json' }]]
    : 'list',

  use: {
    baseURL: 'http://localhost:4174',
    trace: 'on-first-retry',
    viewport: { width: 1280, height: 800 },
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  webServer: {
    command: 'node csp-server.js',
    url: 'http://localhost:4174',
    reuseExistingServer: !process.env.CI,
    timeout: 15_000,
  },
});
