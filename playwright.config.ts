import { defineConfig } from "@playwright/test";

/**
 * Pillar 2 — MAIRON Playwright suite (deterministic, hermetic).
 *
 * `playwright.config.ts` — Chromium-only, `workers: 2`, fully deterministic:
 * no network, no Rust kernel, no flaky clocks. The suite runs against:
 *   * the standalone kernel artifacts (`/war-room`, `/neural-hud`,
 *     `/locales/fa.patch.json`) served byte-exact from the repo tree;
 *   * a deterministic contract surface for `/api/v1/*` (stable JSON shapes);
 *   * the IMPERIUM tab-lifecycle reference implementation of the mini-app
 *     (all tabs stay mounted in the DOM and toggle via `hidden` +
 *     `style.display` — switching never destroys state).
 *
 * Set `MAIRON_BASE_URL` to run the same assertions against a live kernel
 * instead of the hermetic fixture (the webServer is skipped then).
 */
export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: 0,
  workers: 2,
  reporter: [["list"]],
  use: {
    baseURL: process.env.MAIRON_BASE_URL ?? "http://127.0.0.1:8517",
    headless: true,
    viewport: { width: 390, height: 844 },
    actionTimeout: 5_000,
    navigationTimeout: 15_000,
  },
  webServer: process.env.MAIRON_BASE_URL
    ? undefined
    : {
        command: "node tests/e2e/fixtures/hermetic-server.mjs",
        url: "http://127.0.0.1:8517/healthz",
        reuseExistingServer: true,
        timeout: 20_000,
      },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
});
