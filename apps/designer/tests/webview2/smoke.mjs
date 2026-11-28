import { spawn, spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const here = dirname(fileURLToPath(import.meta.url));
const appRoot = resolve(here, "../..");
const repoRoot = resolve(appRoot, "../..");
const executable = join(
  appRoot,
  "src-tauri",
  "target",
  "debug",
  process.platform === "win32"
    ? "composition-designer.exe"
    : "composition-designer",
);
const project = join(appRoot, ".tmp", `webview2-smoke-${process.pid}`);
const screenshot = join(appRoot, ".tmp", "webview2-smoke.png");
const reportPath = join(appRoot, ".tmp", "webview2-smoke.json");

function fail(message) {
  console.error(message);
  process.exit(2);
}

if (!existsSync(executable)) {
  fail(`desktop executable is missing: ${executable}; run npm run desktop`);
}

rmSync(project, { recursive: true, force: true });
cpSync(join(repoRoot, "examples", "minimal-project"), project, {
  recursive: true,
});
mkdirSync(join(appRoot, ".tmp"), { recursive: true });

const port = await new Promise((resolvePort, reject) => {
  const server = createServer();
  server.once("error", reject);
  server.listen(0, "127.0.0.1", () => {
    const address = server.address();
    const selected = typeof address === "object" && address ? address.port : 0;
    server.close(() => resolvePort(selected));
  });
});

const env = {
  ...process.env,
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port} --remote-allow-origins=*`,
};
const child = spawn(executable, ["--project", project], {
  cwd: appRoot,
  env,
  stdio: "ignore",
  windowsHide: false,
});

function stopApp() {
  if (child.exitCode !== null) return;
  if (process.platform === "win32") {
    spawnSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], {
      stdio: "ignore",
    });
  } else {
    child.kill("SIGTERM");
  }
}

async function waitForCdp(url, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // WebView2 is still starting.
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 250));
  }
  throw new Error(`WebView2 CDP endpoint did not start: ${url}`);
}

let browser;
try {
  await waitForCdp(`http://127.0.0.1:${port}/json/version`, 30_000);
  browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
  const context = browser.contexts()[0];
  if (!context) throw new Error("WebView2 CDP exposed no browser context");
  let page = context.pages().find((candidate) => !candidate.isClosed());
  if (!page) page = await context.waitForEvent("page", { timeout: 30_000 });

  await page.waitForSelector(".project-name", { timeout: 30_000 });
  const projectName = await page.locator(".project-name").textContent();
  if (projectName?.trim() !== "Minimal Project") {
    throw new Error(`unexpected project name: ${projectName}`);
  }

  const titleRow = page.locator(".config-row").filter({ hasText: "title" }).first();
  const titleInput = titleRow.locator("input");
  await titleInput.waitFor({ state: "visible", timeout: 15_000 });
  await titleInput.fill("WebView2 Smoke");
  await titleRow.getByRole("button", { name: "Review" }).click();
  const dialog = page.getByRole("dialog");
  await dialog.waitFor({ state: "visible", timeout: 15_000 });
  await dialog.getByRole("button", { name: "Apply transaction" }).click();
  await dialog.waitFor({ state: "hidden", timeout: 15_000 });
  if ((await titleInput.inputValue()) !== "WebView2 Smoke") {
    throw new Error("applied config did not appear in the inspector");
  }

  await page.screenshot({ path: screenshot, fullPage: false });
  const report = {
    ok: true,
    projectName: projectName?.trim(),
    title: await titleInput.inputValue(),
    cdpPort: port,
    screenshot,
  };
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  console.log(JSON.stringify(report));
} catch (error) {
  writeFileSync(
    reportPath,
    `${JSON.stringify({ ok: false, error: String(error), cdpPort: port }, null, 2)}\n`,
  );
  console.error(error);
  process.exitCode = 1;
} finally {
  if (browser) await browser.close().catch(() => undefined);
  stopApp();
  rmSync(project, { recursive: true, force: true });
}
