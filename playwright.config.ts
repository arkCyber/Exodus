import { defineConfig, devices } from '@playwright/test';

/** When set, tests hit an existing `tauri dev` / Vite server (no webServer spawn). */
const tauriE2e = !!process.env.TAURI_E2E;
/** Dedicated Playwright Vite port — avoids clashing with `tauri dev` on :1421. */
const e2ePort = process.env.PLAYWRIGHT_E2E_PORT ?? '1431';
const baseURL =
  process.env.PLAYWRIGHT_BASE_URL ??
  (tauriE2e ? 'http://localhost:1421' : `http://127.0.0.1:${e2ePort}`);

export default defineConfig({
  testDir: './e2e',
  fullyParallel: !tauriE2e,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 1,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: tauriE2e
    ? undefined
    : {
        command: 'pnpm dev:e2e',
        url: baseURL,
        /** Only reuse when explicitly requested (e.g. debugging a running E2E server). */
        reuseExistingServer: !!process.env.PLAYWRIGHT_REUSE_SERVER,
        timeout: 120_000,
      },
});
