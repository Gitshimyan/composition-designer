import { expect, test, type Page } from "@playwright/test";
import { installMock } from "./mockTauri";

async function openValid(page: Page) {
  await installMock(page, "valid");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await expect(page.getByText(/No diagnostics/)).toBeVisible();
}

function titleRow(page: Page) {
  return page.locator(".config-row").filter({ hasText: "title" }).first();
}

test("opens a valid project and shows the structure", async ({ page }) => {
  await openValid(page);
  await expect(page.getByText("Minimal Project", { exact: true }).first()).toBeVisible();
  await expect(page.locator(".tree-node").filter({ hasText: "hello" })).toBeVisible();
  await expect(page.locator(".stage-counts")).toContainText("5 structural bindings");
});

test("switches the composition profile", async ({ page }) => {
  await installMock(page, "v3");
  await page.goto("/");
  const select = page.getByLabel("Composition profile");
  await expect(select).toHaveValue("default");
  await expect(select.locator("option")).toHaveCount(3);
  await select.selectOption("headless");
  await expect(select).toHaveValue("headless");
  const stored = await page.evaluate(() =>
    window.localStorage.getItem("composition-designer.last-profile"),
  );
  expect(stored).toBe("headless");
});

test("shows an invalid project with diagnostics", async ({ page }) => {
  await installMock(page, "invalid");
  await page.goto("/");
  await expect(page.getByText("Cannot resolve model")).toBeVisible();
  await expect(page.locator(".invalid-state")).toContainText("file-project-missing");
  await expect(page.locator(".diagnostic-code")).toContainText("file-project-missing");
});

test("selects an instance and shows its contract", async ({ page }) => {
  await openValid(page);
  await page.locator(".tree-node").filter({ hasText: "hello" }).click();
  await expect(page.locator(".inspector")).toContainText("hello");
  await expect(page.locator(".inspector")).toContainText("leaf.widget");
  await expect(page.locator(".config-row")).toContainText("label");
});

test("edits a property, reviews and applies the transaction", async ({ page }) => {
  await openValid(page);
  const input = titleRow(page).locator("input");
  await input.fill("Edited title");
  await titleRow(page).getByRole("button", { name: "Review" }).click();

  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText("Set app.title");
  await expect(dialog).toContainText("validated");
  await dialog.getByRole("button", { name: "Apply transaction" }).click();
  await expect(dialog).toBeHidden();
  await expect(titleRow(page).locator("input")).toHaveValue("Edited title");

  const calls = await page.evaluate(
    () => (window as unknown as { __COMPOSITION_TEST__: { applyCalls: number } })
      .__COMPOSITION_TEST__.applyCalls,
  );
  expect(calls).toBe(1);
});

test("undoes and redoes a committed edit", async ({ page }) => {
  await openValid(page);
  const input = titleRow(page).locator("input");
  const original = await input.inputValue();
  await input.fill("Undo Me");
  await titleRow(page).getByRole("button", { name: "Review" }).click();
  const dialog = page.getByRole("dialog");
  await dialog.getByRole("button", { name: "Apply transaction" }).click();
  await expect(dialog).toBeHidden();
  await expect(input).toHaveValue("Undo Me");

  const undoButton = page.getByRole("button", { name: "Undo" });
  await expect(undoButton).toBeEnabled();
  await undoButton.click();
  await expect(input).toHaveValue(original);

  const redoButton = page.getByRole("button", { name: "Redo" });
  await expect(redoButton).toBeEnabled();
  await redoButton.click();
  await expect(input).toHaveValue("Undo Me");
});

test("cancels a review without applying", async ({ page }) => {
  await openValid(page);
  await titleRow(page).locator("input").fill("Not applied");
  await titleRow(page).getByRole("button", { name: "Review" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Cancel" }).click();
  await expect(dialog).toBeHidden();

  const calls = await page.evaluate(
    () => (window as unknown as { __COMPOSITION_TEST__: { applyCalls: number } })
      .__COMPOSITION_TEST__.applyCalls,
  );
  expect(calls).toBe(0);
  // Cancel closes the review without committing; the local draft remains editable.
  await expect(titleRow(page).locator("input")).toHaveValue("Not applied");
  const committed = await page.evaluate(
    () => (window as unknown as {
      __COMPOSITION_TEST__: {
        snapshot: { model: { composition: { instances: Array<{ id: string; config: { title?: string } }> } } };
      };
    }).__COMPOSITION_TEST__.snapshot.model.composition.instances
      .find((item) => item.id === "app")?.config.title,
  );
  expect(committed).toBe("Minimal Project");
});

test("shows a rejected transaction and disables apply", async ({ page }) => {
  await installMock(page, "edit-failure");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await titleRow(page).locator("input").fill("bad value");
  await titleRow(page).getByRole("button", { name: "Review" }).click();

  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText("rejected");
  await expect(dialog.locator(".review-diagnostics")).toContainText(
    "contract-property-type-mismatch",
  );
  await expect(dialog.getByRole("button", { name: "Apply transaction" })).toBeDisabled();
});

test("blocks editing after an external change and reloads", async ({ page }) => {
  await openValid(page);
  await page.evaluate(() => {
    (window as unknown as { __COMPOSITION_TEST__: { dirty: boolean } })
      .__COMPOSITION_TEST__.dirty = true;
  });

  const banner = page.locator(".external-banner");
  await expect(banner).toBeVisible({ timeout: 6_000 });
  await expect(banner).toContainText("Files changed on disk");
  await banner.getByRole("button", { name: "Reload project" }).click();
  await expect(banner).toBeHidden();
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
});

test("rebases an open review after an external reload", async ({ page }) => {
  await openValid(page);
  const input = titleRow(page).locator("input");
  await input.fill("Rebased title");
  await titleRow(page).getByRole("button", { name: "Review" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();

  await page.evaluate(() => {
    (window as unknown as { __COMPOSITION_TEST__: { dirty: boolean } })
      .__COMPOSITION_TEST__.dirty = true;
  });
  const banner = page.locator(".external-banner");
  await expect(banner).toBeVisible({ timeout: 6_000 });
  await dialog.getByRole("button", { name: "Reload and revalidate" }).click();
  await expect(banner).toBeHidden();
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Apply transaction" }).click();
  await expect(dialog).toBeHidden();
  await expect(input).toHaveValue("Rebased title");
});

test("saves layout without changing the source digest", async ({ page }) => {
  await openValid(page);
  const digestBefore = await page.locator(".status-digest").getAttribute("title");
  await page.getByRole("button", { name: "Reset layout" }).click();
  await expect(page.getByText("layout saved")).toBeVisible();

  await expect
    .poll(async () =>
      page.evaluate(
        () => (window as unknown as { __COMPOSITION_TEST__: { saveLayoutCalls: number } })
          .__COMPOSITION_TEST__.saveLayoutCalls,
      ),
    )
    .toBeGreaterThan(0);

  const digestAfter = await page.locator(".status-digest").getAttribute("title");
  expect(digestAfter).toBe(digestBefore);
});
