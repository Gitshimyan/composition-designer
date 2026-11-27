import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/ui",
  outputDir: "test-results",
  fullyParallel: false,
  workers: 1,
  timeout: 30_000,
  expect: { timeout: 6_000 },
  reporter: [["list"], ["html", { open: "never", outputFolder: "playwright-report" }]],
  use: {
    baseURL: "http://127.0.0.1:4173",
    channel: "msedge",
    headless: true,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  webServer: {
    command: "npm run preview -- --host 127.0.0.1 --port 4173 --strictPort",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
