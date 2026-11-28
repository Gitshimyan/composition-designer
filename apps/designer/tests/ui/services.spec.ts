import { expect, test, type Page } from "@playwright/test";
import { installMock } from "./mockTauri";

async function openV2(page: Page) {
  await installMock(page, "v2");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("V2 Services");
}

async function openServices(page: Page) {
  await page.getByRole("tab", { name: /Services/ }).click();
  await expect(page.locator(".relation-toolbar")).toContainText("service instances");
}

test("lists services and shows scope plus interfaces", async ({ page }) => {
  await openV2(page);
  await openServices(page);

  await expect(page.locator(".relation-view")).toContainText(
    "service instances",
  );
  const service = page
    .locator(".dependency-group")
    .filter({ hasText: "session" })
    .first();
  await expect(service).toContainText("example.session");
  await expect(service).toContainText("example.session.api@1.0.0");
  await expect(service).toContainText("used by app");

  await service.locator(".dependency-group-head").click();
  await expect(page.locator(".inspector")).toContainText("Scope");
  await expect(page.locator(".inspector")).toContainText("service");
  await expect(page.locator(".inspector")).toContainText(
    "example.session.api@1.0.0",
  );
});

test("dependency view shows interface targets", async ({ page }) => {
  await openV2(page);
  await page.getByRole("tab", { name: /Dependencies/ }).click();
  const row = page
    .locator(".dependency-row")
    .filter({ hasText: "session" })
    .first();
  await expect(row).toContainText("interface example.session.api@1.0.0");
  await expect(row).toContainText("provider:");
  await expect(row).toContainText("session");
});

test("creates a service through review and apply", async ({ page }) => {
  await openV2(page);
  await openServices(page);
  await page.getByRole("button", { name: "New service" }).click();

  const dialog = page.getByRole("dialog");
  await expect(dialog).toContainText("Create service");
  await dialog.getByLabel("Instance id").fill("audit");
  await expect(dialog).toContainText("example.session.api@1.0.0");
  await dialog.getByRole("button", { name: "Review service" }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("Create service audit");
  await review.getByRole("button", { name: "Apply transaction" }).click();
  await expect(review).toBeHidden();

  await expect(page.locator(".relation-view")).toContainText("2 service instances");
  const audit = page
    .locator(".dependency-group")
    .filter({ hasText: "audit" })
    .first();
  await expect(audit).toContainText("service");
});

test("v1 projects explain that services need schema v2", async ({ page }) => {
  await installMock(page, "valid");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await page.getByRole("tab", { name: /Services/ }).click();
  await expect(page.getByRole("button", { name: "New service" })).toBeDisabled();
  await expect(page.locator(".relation-empty")).toContainText("schema v1");
});
