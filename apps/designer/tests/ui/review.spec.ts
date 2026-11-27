import { expect, test, type Page } from "@playwright/test";
import { installMock } from "./mockTauri";

async function openProject(page: Page) {
  await installMock(page, "valid");
  await page.goto("/");
  await expect(page.locator(".project-name")).toHaveText("Minimal Project");
  await expect(page.getByText(/No diagnostics/)).toBeVisible();
}

async function selectInstance(page: Page, id: string) {
  await page.locator(".tree-node").filter({ hasText: id }).click();
  await expect(page.locator(".inspector")).toContainText(id);
}

function titleRow(page: Page) {
  return page.locator(".config-row").filter({ hasText: "title" }).first();
}

test("shows commands, before/after and full validation for a config edit", async ({
  page,
}) => {
  await openProject(page);
  await titleRow(page).locator("input").fill("Edited title");
  await titleRow(page).getByRole("button", { name: "Review" }).click();

  const review = page.getByRole("dialog");
  await expect(review).toContainText("Set app.title");
  await expect(review.locator(".review-section-head").filter({ hasText: "Commands" })).toBeVisible();
  await expect(review.locator(".review-command-list")).toContainText(
    "set_config app.title = Edited title",
  );

  const instanceChange = review
    .locator(".review-change")
    .filter({ hasText: "instance:app" });
  await expect(instanceChange).toContainText("config.title");
  const titleField = instanceChange
    .locator(".review-field-row")
    .filter({ hasText: "config.title" });
  await expect(titleField).toContainText("Minimal Project");
  await expect(titleField).toContainText("Edited title");

  await expect(review.locator(".review-validation")).toHaveText(
    "candidate passed L2–L7",
  );
  await expect(review.getByText("No diagnostics.")).toBeVisible();
});

test("groups a multi-command transaction by object kind", async ({ page }) => {
  await openProject(page);
  await selectInstance(page, "app");
  await page
    .locator(".slot-row")
    .filter({ hasText: "content" })
    .getByRole("button", { name: "Add child…" })
    .click();

  const create = page.getByRole("dialog", { name: /app · content/ });
  const rootFields = create.locator(
    "fieldset.draft-node.is-root > .dialog-fields",
  );
  await rootFields.getByLabel("Module").selectOption("ui.panel@1.0.0");
  const bodyGroup = create.getByRole("group", { name: "Slot body child" });
  const bodyFields = bodyGroup.locator(".dialog-fields");
  await bodyFields.getByLabel("Instance id").fill("card-body");
  await bodyFields.getByLabel("label").fill("Card body");
  await rootFields.getByLabel("Instance id").fill("card");
  await create.getByRole("button", { name: "Review create" }).click();

  const review = page.getByRole("dialog", {
    name: /Create card subtree .* under app\.content/,
  });
  await expect(review.locator(".review-command-list li")).toHaveCount(4);
  await expect(
    review.locator(".review-section-head").filter({ hasText: "Instances" }),
  ).toContainText("2");
  await expect(
    review.locator(".review-section-head").filter({ hasText: "Bindings" }),
  ).toContainText("2");
  await expect(
    review.locator('.review-change[data-object-id="instance:card"]'),
  ).toContainText("ui.panel@1.0.0");
  await expect(
    review.locator('.review-change[data-object-id="binding:bind-card-body"]'),
  ).toContainText("card-body");
  await expect(review.locator(".review-validation")).toHaveText(
    "candidate passed L2–L7",
  );
});

test("shows connection before/after fields", async ({ page }) => {
  await openProject(page);
  await page.getByRole("tab", { name: /Connections/ }).click();
  await page
    .locator(".connection-row")
    .filter({ hasText: "box-log-line" })
    .getByRole("button", { name: "Edit" })
    .click();

  const editor = page.getByRole("dialog", { name: "Edit connection" });
  await editor.getByLabel("To port").selectOption("audit");
  await editor.getByRole("button", { name: "Review connection" }).click();

  const review = page.getByRole("dialog", {
    name: "Update connection box-log-line",
  });
  const connectionChange = review
    .locator(".review-change")
    .filter({ hasText: "connection:box-log-line" });
  const toField = connectionChange
    .locator(".review-field-row")
    .filter({ hasText: "to" });
  await expect(toField).toContainText("logger.line");
  await expect(toField).toContainText("logger.audit");
  await expect(review.locator(".review-command-list")).toContainText(
    "update_connection",
  );
});

test("shows explicit dependency provider impact", async ({ page }) => {
  await openProject(page);
  await page.getByRole("tab", { name: /Dependencies/ }).click();
  const group = page.locator(".dependency-group").filter({ hasText: "box" });
  await group.getByRole("button", { name: "Set provider…" }).click();

  const editor = page.getByRole("dialog", { name: "box · logger" });
  await editor.getByRole("radio", { name: /logger-2/ }).check();
  await editor.getByRole("button", { name: "Review provider" }).click();

  const review = page.getByRole("dialog", {
    name: "Set box.logger provider to logger-2",
  });
  const impact = review
    .locator(".review-section")
    .filter({ hasText: "Dependency impact" });
  await expect(impact).toContainText("box.logger");
  await expect(impact).toContainText("logger → logger-2");
  await expect(impact).toContainText("current resolved");
  await expect(review.locator(".review-command-list")).toContainText(
    "set_dependency_provider",
  );
});
