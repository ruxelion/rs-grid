import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  // Screenshots de référence stockées à côté des specs
  snapshotDir: './tests/snapshots',
  // Crée les snapshots manquants (première exécution sur une nouvelle
  // plateforme) au lieu d'échouer. Les snapshots existants sont comparés
  // normalement.
  updateSnapshots: 'missing',
  fullyParallel: false,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',

  use: {
    baseURL: 'http://localhost:4173',
    // Capture trace en cas d'échec pour debug
    trace: 'on-first-retry',
    // Viewport fixe pour des screenshots reproductibles
    viewport: { width: 1280, height: 800 },
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Démarre un serveur statique sur le dist/ pré-compilé.
  // Lancer `trunk build` dans examples/basic-leptos avant les tests.
  webServer: {
    command: 'npx serve ../examples/basic-leptos/dist -p 4173 --no-clipboard',
    url: 'http://localhost:4173',
    reuseExistingServer: !process.env.CI,
    timeout: 15_000,
  },
});
